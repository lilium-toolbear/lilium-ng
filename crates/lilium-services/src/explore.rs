// Python parity source: dzmm_archive@18fdefbc0b6979178d7f1eb4ce0624ec4a60a2f2 core/explore.py
// Ports ExploreFetcher orchestration. Per the migration SOP, orchestration lives
// in lilium-services.
//
// Divergence: the Python fetcher fires background tweet media downloads. This
// port saves tweets (and their media_urls) but does not download the media
// files to disk yet — tweet-specific attachment paths + a tweet media downloader
// are a remaining gap. Tweets are still upserted with their remote `media_urls`.
use crate::{account, explore_content, user};
use lilium_api_client::http::DzmmApi;
use lilium_models::dzmm::{book, card, chapter, checkpoint, gallery, tweet};
use sea_orm::ConnectionTrait;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::instrument;

/// Configuration for [`ExploreFetcher`]. Mirrors Python `ExploreFetchConfig`.
#[derive(Debug, Clone)]
pub struct ExploreFetchConfig {
    pub sort: String,
    pub max_pages: Option<u32>,
    pub initial_offset: u64,
    pub page_size: u64,
    pub content_types: Vec<String>,
    pub user_info_cache_hours: i64,
}

impl Default for ExploreFetchConfig {
    fn default() -> Self {
        Self {
            sort: "recent".to_string(),
            max_pages: None,
            initial_offset: 0,
            page_size: 100,
            content_types: vec![
                "cards".into(),
                "novels".into(),
                "tweets".into(),
                "checkpoints".into(),
                "galleries".into(),
                "gamefy".into(),
            ],
            user_info_cache_hours: 1,
        }
    }
}

/// Statistics returned by [`ExploreFetcher::fetch_and_process`]. Mirrors Python
/// `ExploreFetchStats`.
#[derive(Debug, Clone, Default)]
pub struct ExploreFetchStats {
    pub tweets_saved: usize,
    pub tweets_updated: usize,
    pub images_downloaded: usize,
    pub other_content_skipped: usize,
    pub cards_saved: usize,
    pub galleries_saved: usize,
    pub checkpoints_saved: usize,
    pub books_saved: usize,
    pub chapters_saved: usize,
    pub errors: usize,
    pub pages_fetched: u32,
    pub stopped_early: bool,
}

/// Fetches and stores explore-feed content. Mirrors Python
/// `core.explore.ExploreFetcher`.
pub struct ExploreFetcher<'a> {
    auth: &'a DzmmApi,
    config: ExploreFetchConfig,
    backfill: bool,
    #[allow(dead_code)]
    data_path: PathBuf,
    /// Cache for prefetched book details (book_id -> detailed JSON).
    /// Mirrors Python's `item["_detailed_book"]` pattern.
    book_details_cache: HashMap<String, serde_json::Value>,
}

impl<'a> ExploreFetcher<'a> {
    pub fn new(
        auth: &'a DzmmApi,
        data_path: PathBuf,
        config: ExploreFetchConfig,
        backfill: bool,
    ) -> Self {
        Self {
            auth,
            config,
            backfill,
            data_path,
            book_details_cache: HashMap::new(),
        }
    }

    #[instrument(level = "debug", skip(self, db), fields(sort = %self.config.sort))]
    pub async fn fetch_and_process<C>(&mut self, db: &C) -> crate::Result<ExploreFetchStats>
    where
        C: ConnectionTrait,
    {
        let mut stats = ExploreFetchStats::default();
        let mut offset = self.config.initial_offset;
        let mut page: u32 = 1;

        loop {
            if let Some(max) = self.config.max_pages
                && page > max
            {
                break;
            }

            let results = match self.fetch_page(offset).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Error fetching page {page}: {e}");
                    stats.errors += 1;
                    break;
                }
            };
            if results.is_empty() {
                break;
            }
            stats.pages_fetched = page;
            tracing::info!(
                "📄 Page {page}: fetched {} items (offset={offset})",
                results.len()
            );

            let (should_stop, user_ids) = self.process_items(db, &results, &mut stats).await?;

            // Update public user profiles for newly-seen authors.
            if !user_ids.is_empty() {
                let pairs: Vec<(String, String)> = user_ids
                    .iter()
                    .map(|uid| (uid.clone(), String::new()))
                    .collect();
                let _ = user::batch_fetch_and_update_with_auth(
                    db,
                    self.auth,
                    &pairs,
                    self.config.user_info_cache_hours,
                )
                .await;
            }

            if should_stop {
                stats.stopped_early = true;
                break;
            }
            offset += self.config.page_size;
            page += 1;
        }
        Ok(stats)
    }

    async fn fetch_page(&self, offset: u64) -> crate::Result<Vec<serde_json::Value>> {
        let types_param = self.config.content_types.join(",");
        let data = self
            .auth
            .fetch_explore_feed(
                &types_param,
                Some(offset),
                Some(&self.config.sort),
                Some(self.config.page_size),
            )
            .await
            .map_err(|e| lilium_common::LiliumError::service("EXPLORE_API_ERROR", e.to_string()))?;
        Ok(data
            .get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
    }

    async fn process_items<C>(
        &mut self,
        db: &C,
        items: &[serde_json::Value],
        stats: &mut ExploreFetchStats,
    ) -> crate::Result<(bool, Vec<String>)>
    where
        C: ConnectionTrait,
    {
        let mut user_ids: Vec<String> = Vec::new();

        if !self.backfill {
            self.prefetch_book_details(items, stats).await;
        }

        for item in items {
            let Some(content_type) = item.get("type").and_then(|v| v.as_str()) else {
                continue;
            };

            if self.backfill && self.is_known_content(db, item, content_type).await? {
                return Ok((true, user_ids));
            }
            if self.backfill && content_type == "book" {
                self.prefetch_book_detail(item, stats).await;
            }

            if let Some(uid) = self.save_content(db, item, content_type, stats).await? {
                user_ids.push(uid);
            }
        }
        Ok((false, user_ids))
    }

    async fn prefetch_book_details(
        &mut self,
        items: &[serde_json::Value],
        stats: &mut ExploreFetchStats,
    ) {
        for item in items {
            self.prefetch_book_detail(item, stats).await;
        }
    }

    async fn prefetch_book_detail(
        &mut self,
        item: &serde_json::Value,
        stats: &mut ExploreFetchStats,
    ) {
        if item.get("type").and_then(|v| v.as_str()) != Some("book") {
            return;
        }
        let payload = merge_item_payload(item);
        let Some(book_id) = payload.get("id").and_then(|v| v.as_str()) else {
            return;
        };
        match self.auth.fetch_novel_book(book_id).await {
            Ok(detailed) if detailed.is_object() => {
                // Cache the detailed book info for later use in save_content.
                // Mirrors Python's `item["_detailed_book"] = detailed_book`.
                self.book_details_cache
                    .insert(book_id.to_string(), detailed);
                tracing::debug!(book_id, "Prefetched book details");
            }
            Ok(_) => {}
            Err(e) => {
                stats.errors += 1;
                tracing::warn!(book_id, "Could not fetch novel details: {e}");
            }
        }
    }

    async fn is_known_content<C>(
        &self,
        db: &C,
        item: &serde_json::Value,
        content_type: &str,
    ) -> crate::Result<bool>
    where
        C: ConnectionTrait,
    {
        let data = item.get("data").unwrap_or(&serde_json::Value::Null);
        let Some(id) = data.get("id").and_then(|v| v.as_str()) else {
            return Ok(false);
        };
        let known = match content_type {
            "tweet" => explore_content::get_tweet(db, id).await?.is_some(),
            "card" | "gamefy" => match data.get("id").and_then(|v| v.as_i64()) {
                Some(n) => explore_content::get_card(db, n as i32).await?.is_some(),
                None => false,
            },
            "gallery" => explore_content::get_gallery(db, id).await?.is_some(),
            "checkpoint" => explore_content::get_checkpoint(db, id).await?.is_some(),
            "book" => explore_content::get_book(db, id).await?.is_some(),
            "chapter" => explore_content::get_chapter(db, id).await?.is_some(),
            _ => false,
        };
        if known {
            tracing::info!("✓ Reached known {content_type} {id}, stopping fetch");
        }
        Ok(known)
    }

    /// Save one content item. Returns the authoring user_id for profile sync.
    async fn save_content<C>(
        &mut self,
        db: &C,
        item: &serde_json::Value,
        content_type: &str,
        stats: &mut ExploreFetchStats,
    ) -> crate::Result<Option<String>>
    where
        C: ConnectionTrait,
    {
        let payload = merge_item_payload(item);
        if !payload.is_object() {
            stats.other_content_skipped += 1;
            return Ok(None);
        }

        match content_type {
            "tweet" => {
                let Some(tweet_model) = tweet::Model::from_api(&payload) else {
                    stats.errors += 1;
                    return Ok(None);
                };
                let id = tweet_model.tweet_id.clone();
                let uid = tweet_model.user_id.clone();
                let existed = explore_content::get_tweet(db, &id).await?.is_some();
                let _ = explore_content::upsert_tweet(db, tweet_model).await?;
                if existed {
                    stats.tweets_updated += 1;
                } else {
                    stats.tweets_saved += 1;
                }
                // Media download is a remaining gap; media_urls are stored.
                Ok(uid)
            }
            "card" | "gamefy" => {
                let Some(card_model) = card::Model::from_api(&payload) else {
                    stats.errors += 1;
                    return Ok(None);
                };
                let uid = card_model.user_id.clone();
                let _ = explore_content::upsert_card(db, card_model).await?;
                stats.cards_saved += 1;
                Ok(uid)
            }
            "gallery" => {
                let Some(gallery_model) = gallery::Model::from_api(&payload) else {
                    stats.errors += 1;
                    return Ok(None);
                };
                let uid = gallery_model.user_id.clone();
                let _ = explore_content::upsert_gallery(db, gallery_model).await?;
                stats.galleries_saved += 1;
                Ok(uid)
            }
            "checkpoint" => {
                let Some(checkpoint_model) = checkpoint::Model::from_api(&payload) else {
                    stats.errors += 1;
                    return Ok(None);
                };
                let uid = checkpoint_model.user_id.clone();
                let _ = explore_content::upsert_checkpoint(db, checkpoint_model).await?;
                stats.checkpoints_saved += 1;
                Ok(uid)
            }
            "book" => {
                // Merge detailed book info from cache if available.
                // Mirrors Python's `item.get("_detailed_book")` pattern.
                let book_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let book_payload = if let Some(detailed) = self.book_details_cache.remove(book_id) {
                    merge_payloads(&payload, &detailed)
                } else {
                    payload.clone()
                };
                let Some(book_model) = book::Model::from_api(&book_payload) else {
                    stats.errors += 1;
                    return Ok(None);
                };
                let uid = book_model.user_id.clone();
                let author = book_model.author.clone();
                let book_id = book_model.book_id.clone();
                let _ = explore_content::upsert_book(db, book_model).await?;
                stats.books_saved += 1;

                // Persist chapters embedded in the book payload.
                if let Some(chapters) = book_payload.get("chapters").and_then(|v| v.as_array()) {
                    for chapter_data in chapters {
                        if !chapter_data.is_object() || chapter_data.get("id").is_none() {
                            continue;
                        }
                        let mut chap_payload = chapter_data.clone();
                        if let Some(obj) = chap_payload.as_object_mut() {
                            obj.entry("bookId".to_string())
                                .or_insert(serde_json::json!(book_id));
                            if let Some(uid) = &uid {
                                obj.entry("userId".to_string())
                                    .or_insert(serde_json::json!(uid));
                            }
                            if let Some(author) = &author {
                                obj.entry("author".to_string()).or_insert(author.clone());
                            }
                        }
                        if let Some(chapter_model) = chapter::Model::from_api(&chap_payload) {
                            let _ = explore_content::upsert_chapter(db, chapter_model).await?;
                            stats.chapters_saved += 1;
                        }
                    }
                }
                Ok(uid)
            }
            "chapter" => {
                let Some(chapter_model) = chapter::Model::from_api(&payload) else {
                    stats.errors += 1;
                    return Ok(None);
                };
                let uid = payload
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned);
                let _ = explore_content::upsert_chapter(db, chapter_model).await?;
                stats.chapters_saved += 1;
                Ok(uid)
            }
            other => {
                tracing::warn!("Unknown content type '{other}', skipping");
                stats.other_content_skipped += 1;
                Ok(None)
            }
        }
    }
}

/// Combine tRPC wrapper metadata with the inner `data` payload. Mirrors Python
/// `ExploreFetcher._merge_item_payload`.
fn merge_item_payload(item: &serde_json::Value) -> serde_json::Value {
    let mut payload = serde_json::Map::new();
    if let Some(obj) = item.as_object() {
        for (k, v) in obj {
            if k != "data" {
                payload.insert(k.clone(), v.clone());
            }
        }
    }
    if let Some(data) = item.get("data").filter(|v| v.is_object())
        && let Some(data_obj) = data.as_object()
    {
        for (k, v) in data_obj {
            payload.insert(k.clone(), v.clone());
        }
    }
    serde_json::Value::Object(payload)
}

/// Merge two JSON objects, with `detailed` fields taking precedence.
/// Mirrors Python's `{**book_payload, **detailed_book}` pattern.
fn merge_payloads(payload: &serde_json::Value, detailed: &serde_json::Value) -> serde_json::Value {
    let mut merged = payload.clone();
    if let (Some(merged_obj), Some(detailed_obj)) = (merged.as_object_mut(), detailed.as_object()) {
        for (k, v) in detailed_obj {
            merged_obj.insert(k.clone(), v.clone());
        }
    }
    merged
}

/// Resolve the next available enabled account + auth client for the explore
/// CLI. Mirrors Python `AccountService.get_next_available_account`.
pub async fn next_auth_client<C>(db: &C) -> crate::Result<Option<DzmmApi>>
where
    C: ConnectionTrait,
{
    let Some(account) = account::get_next_available_account(db).await? else {
        return Ok(None);
    };
    Ok(Some(account::create_auth_client(account)?))
}
