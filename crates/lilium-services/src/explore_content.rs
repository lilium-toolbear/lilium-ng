// Python parity source: dzmm_archive@18fdefbc0b6979178d7f1eb4ce0624ec4a60a2f2 services/explore_content_service.py
// Python parity source: dzmm_archive@18fdefbc0b6979178d7f1eb4ce0624ec4a60a2f2 services/tweet_service.py
//
// Divergence: Python uses `model_dump(exclude_unset=True)` for partial updates,
// only updating fields that were explicitly set. Rust uses `reset_all()` which
// updates all non-PK fields. This is acceptable because the Rust models are
// always constructed with all fields from `from_api`, so all fields are "set".
//
// Upsert + get_by_id for the six explore-content entities. Each upsert is a
// check-then-insert/update (mirrors the Python `model_dump(exclude_unset=True)`
// field copy by overwriting all non-PK fields on update).
use crate::Result;
use lilium_models::dzmm::{book, card, chapter, checkpoint, gallery, tweet};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};
use tracing::instrument;

// ---------------- tweet ----------------

#[instrument(level = "debug", skip(db), fields(tweet_id = %tweet_id))]
pub async fn get_tweet<C>(db: &C, tweet_id: &str) -> Result<Option<tweet::Model>>
where
    C: ConnectionTrait,
{
    Ok(tweet::Entity::find_by_id(tweet_id.to_owned())
        .one(db)
        .await?)
}

/// Insert a new tweet. Mirrors Python `TweetService.create_tweet`.
#[instrument(level = "debug", skip(db, model), fields(tweet_id = %model.tweet_id))]
pub async fn create_tweet<C>(db: &C, model: tweet::Model) -> Result<tweet::Model>
where
    C: ConnectionTrait,
{
    let active: tweet::ActiveModel = model.into();
    Ok(active.insert(db).await?)
}

/// Insert or update a tweet. Mirrors Python `TweetService.upsert_tweet`.
/// Returns `true` if a new tweet was created.
#[instrument(level = "debug", skip(db, model), fields(tweet_id = %model.tweet_id))]
pub async fn upsert_tweet<C>(db: &C, model: tweet::Model) -> Result<bool>
where
    C: ConnectionTrait,
{
    if get_tweet(db, &model.tweet_id).await?.is_some() {
        let active: tweet::ActiveModel = model.into();
        active.reset_all().update(db).await?;
        Ok(false)
    } else {
        let active: tweet::ActiveModel = model.into();
        active.insert(db).await?;
        Ok(true)
    }
}

#[instrument(level = "debug", skip(db, paths), fields(tweet_id = %tweet_id, path_count = paths.len()))]
pub async fn set_tweet_local_media_paths<C>(
    db: &C,
    tweet_id: &str,
    paths: Vec<String>,
) -> Result<()>
where
    C: ConnectionTrait,
{
    let Some(existing) = get_tweet(db, tweet_id).await? else {
        return Ok(());
    };
    if existing.local_media_paths.as_ref() == Some(&paths) {
        return Ok(());
    }
    let mut active: tweet::ActiveModel = existing.into();
    active.local_media_paths = Set(Some(paths));
    active.updated_at = Set(Some(chrono::Utc::now()));
    active.update(db).await?;
    Ok(())
}

// ---------------- card ----------------

#[instrument(level = "debug", skip(db), fields(card_id = card_id))]
pub async fn get_card<C>(db: &C, card_id: i32) -> Result<Option<card::Model>>
where
    C: ConnectionTrait,
{
    Ok(card::Entity::find_by_id(card_id).one(db).await?)
}

#[instrument(level = "debug", skip(db, model), fields(card_id = model.card_id))]
pub async fn upsert_card<C>(db: &C, model: card::Model) -> Result<bool>
where
    C: ConnectionTrait,
{
    if get_card(db, model.card_id).await?.is_some() {
        let active: card::ActiveModel = model.into();
        active.reset_all().update(db).await?;
        Ok(false)
    } else {
        let active: card::ActiveModel = model.into();
        active.insert(db).await?;
        Ok(true)
    }
}

// ---------------- gallery ----------------

#[instrument(level = "debug", skip(db), fields(gallery_id = %gallery_id))]
pub async fn get_gallery<C>(db: &C, gallery_id: &str) -> Result<Option<gallery::Model>>
where
    C: ConnectionTrait,
{
    Ok(gallery::Entity::find_by_id(gallery_id.to_owned())
        .one(db)
        .await?)
}

#[instrument(level = "debug", skip(db, model), fields(gallery_id = %model.gallery_id))]
pub async fn upsert_gallery<C>(db: &C, model: gallery::Model) -> Result<bool>
where
    C: ConnectionTrait,
{
    if get_gallery(db, &model.gallery_id).await?.is_some() {
        let active: gallery::ActiveModel = model.into();
        active.reset_all().update(db).await?;
        Ok(false)
    } else {
        let active: gallery::ActiveModel = model.into();
        active.insert(db).await?;
        Ok(true)
    }
}

// ---------------- checkpoint ----------------

#[instrument(level = "debug", skip(db), fields(checkpoint_id = %checkpoint_id))]
pub async fn get_checkpoint<C>(db: &C, checkpoint_id: &str) -> Result<Option<checkpoint::Model>>
where
    C: ConnectionTrait,
{
    Ok(checkpoint::Entity::find_by_id(checkpoint_id.to_owned())
        .one(db)
        .await?)
}

#[instrument(level = "debug", skip(db, model), fields(checkpoint_id = %model.checkpoint_id))]
pub async fn upsert_checkpoint<C>(db: &C, model: checkpoint::Model) -> Result<bool>
where
    C: ConnectionTrait,
{
    if get_checkpoint(db, &model.checkpoint_id).await?.is_some() {
        let active: checkpoint::ActiveModel = model.into();
        active.reset_all().update(db).await?;
        Ok(false)
    } else {
        let active: checkpoint::ActiveModel = model.into();
        active.insert(db).await?;
        Ok(true)
    }
}

// ---------------- book ----------------

#[instrument(level = "debug", skip(db), fields(book_id = %book_id))]
pub async fn get_book<C>(db: &C, book_id: &str) -> Result<Option<book::Model>>
where
    C: ConnectionTrait,
{
    Ok(book::Entity::find_by_id(book_id.to_owned()).one(db).await?)
}

#[instrument(level = "debug", skip(db, model), fields(book_id = %model.book_id))]
pub async fn upsert_book<C>(db: &C, model: book::Model) -> Result<bool>
where
    C: ConnectionTrait,
{
    if get_book(db, &model.book_id).await?.is_some() {
        let active: book::ActiveModel = model.into();
        active.reset_all().update(db).await?;
        Ok(false)
    } else {
        let active: book::ActiveModel = model.into();
        active.insert(db).await?;
        Ok(true)
    }
}

// ---------------- chapter ----------------

#[instrument(level = "debug", skip(db), fields(chapter_id = %chapter_id))]
pub async fn get_chapter<C>(db: &C, chapter_id: &str) -> Result<Option<chapter::Model>>
where
    C: ConnectionTrait,
{
    Ok(chapter::Entity::find_by_id(chapter_id.to_owned())
        .one(db)
        .await?)
}

#[instrument(level = "debug", skip(db, model), fields(chapter_id = %model.chapter_id))]
pub async fn upsert_chapter<C>(db: &C, model: chapter::Model) -> Result<bool>
where
    C: ConnectionTrait,
{
    if get_chapter(db, &model.chapter_id).await?.is_some() {
        let active: chapter::ActiveModel = model.into();
        active.reset_all().update(db).await?;
        Ok(false)
    } else {
        let active: chapter::ActiveModel = model.into();
        active.insert(db).await?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lilium_test_fixtures::FixtureProfile;
    use serde_json::json;

    #[tokio::test]
    async fn tweet_upsert_inserts_then_updates() {
        let test_db = lilium_test_fixtures::TestDb::acquire(FixtureProfile::Empty)
            .await
            .expect("acquire explore db");
        let db = test_db.database().orm();
        let data = json!({
            "id": "tw-1",
            "created_at": "2024-11-16T04:27:00.123Z",
            "content": "hello",
            "userId": "u1",
            "likesCount": 3,
        });
        let tweet = tweet::Model::from_api(&data).expect("parse");
        let created = upsert_tweet(db, tweet).await.expect("insert");
        assert!(created);
        let data2 = json!({
            "id": "tw-1",
            "created_at": "2024-11-16T04:27:00.123Z",
            "content": "updated",
            "userId": "u1",
            "likesCount": 5,
        });
        let tweet2 = tweet::Model::from_api(&data2).expect("parse");
        let created2 = upsert_tweet(db, tweet2).await.expect("update");
        assert!(!created2);
        let row = get_tweet(db, "tw-1").await.unwrap().unwrap();
        assert_eq!(row.content.as_deref(), Some("updated"));
        assert_eq!(row.likes_count, 5);
    }

    #[tokio::test]
    async fn card_upsert_inserts_then_updates() {
        let test_db = lilium_test_fixtures::TestDb::acquire(FixtureProfile::Empty)
            .await
            .expect("acquire explore db");
        let db = test_db.database().orm();
        let data = json!({"id": 42, "name": "card1", "userId": "u1", "isGamefy": true});
        let card = card::Model::from_api(&data).expect("parse");
        assert!(upsert_card(db, card).await.unwrap());
        let data2 = json!({"id": 42, "name": "card1-updated", "userId": "u1", "isGamefy": true});
        let card2 = card::Model::from_api(&data2).expect("parse");
        assert!(!upsert_card(db, card2).await.unwrap());
        let row = get_card(db, 42).await.unwrap().unwrap();
        assert_eq!(row.name.as_deref(), Some("card1-updated"));
        assert!(row.is_gamefy);
    }

    #[tokio::test]
    async fn book_and_chapter_upsert() {
        let test_db = lilium_test_fixtures::TestDb::acquire(FixtureProfile::Empty)
            .await
            .expect("acquire explore db");
        let db = test_db.database().orm();
        let book_data = json!({
            "id": "book-1",
            "title": "My Novel",
            "userId": "u-author",
            "createdAt": "2024-01-01T00:00:00Z",
            "chapterCount": 1,
        });
        let book = book::Model::from_api(&book_data).expect("parse");
        assert!(upsert_book(db, book).await.unwrap());
        let chap_data = json!({
            "id": "chap-1",
            "bookId": "book-1",
            "userId": "u-author",
            "title": "Chapter 1",
            "createdAt": "2024-01-02T00:00:00Z",
        });
        let chap = chapter::Model::from_api(&chap_data).expect("parse");
        assert!(upsert_chapter(db, chap).await.unwrap());
        let row = get_book(db, "book-1").await.unwrap().unwrap();
        assert_eq!(row.title.as_deref(), Some("My Novel"));
        let chap_row = get_chapter(db, "chap-1").await.unwrap().unwrap();
        assert_eq!(chap_row.title.as_deref(), Some("Chapter 1"));
    }
}
