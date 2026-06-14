use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use lilium_common::LiliumError;
use lilium_database::DbSessionContext;

use lilium_models::dzmm::message::Message;
use tracing::instrument;

const SELECT_ENRICHED: &str = r#"m.message_id, m.room_id, m.sent_at, m.sent_by, m.content_type, m.content_text,
    m.content_tsv::text, m.attachment_url, m.attachment_file, m.sticker_id, m.alt_text,
    m.metadata, m.raw_data, m.source, m.created_at, m.updated_at,
    m.is_deleted, m.deleted_at, m.deleted_by, m.is_recalled, m.is_edited,
    m.history, m.reference_message_id, m.reference_data,
    u.full_name AS user_display_name, u.avatar_url AS user_avatar_url,
    r.title AS room_title"#;

const SELECT_BASE: &str = r#"m.message_id, m.room_id, m.sent_at, m.sent_by, m.content_type, m.content_text,
    m.content_tsv::text, m.attachment_url, m.attachment_file, m.sticker_id, m.alt_text,
    m.metadata, m.raw_data, m.source, m.created_at, m.updated_at,
    m.is_deleted, m.deleted_at, m.deleted_by, m.is_recalled, m.is_edited,
    m.history, m.reference_message_id, m.reference_data,
    NULL::varchar AS user_display_name, NULL::varchar AS user_avatar_url,
    NULL::varchar AS room_title"#;

const SELECT_MESSAGE_COLS: &str = r#"message_id, room_id, sent_at, sent_by, content_type, content_text,
    content_tsv::text, attachment_url, attachment_file, sticker_id, alt_text,
    metadata, raw_data, source, created_at, updated_at,
    is_deleted, deleted_at, deleted_by, is_recalled, is_edited,
    history, reference_message_id, reference_data"#;

pub struct MessageService<'a> {
    session: DbSessionContext<'a>,
}

#[derive(Debug, Clone)]
enum BindVal {
    Text(String),
    Timestamp(DateTime<Utc>),
    Bool(bool),
    TextArray(Vec<String>),
}

#[derive(Debug, Clone, Default)]
struct QueryParts {
    conditions: String,
    joins: String,
    params: Vec<BindVal>,
}

fn build_query_parts(filters: &MessageFilters, include_visible_only: bool) -> QueryParts {
    let mut p = QueryParts::default();
    let mut n: u32 = 0;

    let mut needs_user_join = false;
    let mut needs_room_join = false;

    macro_rules! next_n {
        () => {{
            n += 1;
            n
        }};
    }

    if let Some(ref v) = filters.room_id {
        p.conditions
            .push_str(&format!(" AND m.room_id = ${}", next_n!()));
        p.params.push(BindVal::Text(v.clone()));
    }

    if let Some(ref ids) = filters.room_ids {
        if !ids.is_empty() {
            p.conditions
                .push_str(&format!(" AND m.room_id = ANY(${})", next_n!()));
            p.params.push(BindVal::TextArray(ids.clone()));
        }
    }

    if let Some(ref v) = filters.account_id {
        needs_room_join = true;
        p.joins
            .push_str(" LEFT JOIN rooms r ON m.room_id = r.room_id");
        p.conditions.push_str(&format!(
            " AND r.account_ids @> ARRAY[${}]::varchar[]",
            next_n!()
        ));
        p.params.push(BindVal::Text(v.clone()));
    } else if filters.user_or_account_id.is_some() {
        let Some(id) = filters.user_or_account_id.as_ref() else {
            unreachable!("checked is_some above")
        };
        needs_room_join = true;
        p.joins
            .push_str(" LEFT JOIN rooms r ON m.room_id = r.room_id");
        p.conditions.push_str(&format!(
            " AND (m.sent_by = ${next} OR r.account_ids @> ARRAY[${next}]::varchar[])",
            next = next_n!()
        ));
        p.params.push(BindVal::Text(id.clone()));
    }

    if let Some(ref v) = filters.user_id {
        p.conditions
            .push_str(&format!(" AND m.sent_by = ${}", next_n!()));
        p.params.push(BindVal::Text(v.clone()));
    }

    if let Some(ref types) = filters.content_types {
        if !types.is_empty() {
            p.conditions
                .push_str(&format!(" AND m.content_type = ANY(${})", next_n!()));
            p.params.push(BindVal::TextArray(types.clone()));
        }
    }

    if let Some(ref types) = filters.message_types {
        let mut type_conds: Vec<String> = Vec::new();
        if types.iter().any(|t| t == "deleted") {
            let nn = next_n!();
            type_conds.push(format!("m.is_deleted = ${}", nn));
            p.params.push(BindVal::Bool(true));
        }
        if types.iter().any(|t| t == "recalled") {
            let nn = next_n!();
            type_conds.push(format!("m.is_recalled = ${}", nn));
            p.params.push(BindVal::Bool(true));
        }
        if types.iter().any(|t| t == "edited") {
            let nn = next_n!();
            type_conds.push(format!("m.is_edited = ${}", nn));
            p.params.push(BindVal::Bool(true));
        }
        if !type_conds.is_empty() {
            p.conditions
                .push_str(&format!(" AND ({})", type_conds.join(" OR ")));
        }
    }

    if let Some(ref q) = filters.search_query {
        let is_uuid = q.contains('-') && q.len() >= 36 && q.matches('-').count() >= 4;
        if is_uuid {
            p.conditions
                .push_str(&format!(" AND m.message_id = ${}", next_n!()));
            p.params.push(BindVal::Text(q.clone()));
        } else {
            p.conditions.push_str(&format!(
                " AND m.content_tsv @@ plainto_tsquery('zhparser', ${})",
                next_n!()
            ));
            p.params.push(BindVal::Text(q.clone()));
        }
    }

    if let Some(ref name) = filters.sender_name {
        needs_user_join = true;
        if !p.joins.contains("LEFT JOIN users") {
            p.joins
                .push_str(" LEFT JOIN users u ON m.sent_by = u.user_id");
        }
        p.conditions.push_str(&format!(
            " AND u.name_tsv @@ plainto_tsquery('zhparser', ${})",
            next_n!()
        ));
        p.params.push(BindVal::Text(name.clone()));
    }

    if let Some(ref v) = filters.start_time {
        p.conditions
            .push_str(&format!(" AND m.sent_at >= ${}", next_n!()));
        p.params.push(BindVal::Timestamp(*v));
    }

    if let Some(ref v) = filters.end_time {
        p.conditions
            .push_str(&format!(" AND m.sent_at <= ${}", next_n!()));
        p.params.push(BindVal::Timestamp(*v));
    }

    if let Some(v) = filters.has_attachment {
        if v {
            p.conditions.push_str(" AND m.attachment_file IS NOT NULL");
        } else {
            p.conditions.push_str(" AND m.attachment_file IS NULL");
        }
    }

    if let Some(v) = filters.has_reference {
        if v {
            p.conditions
                .push_str(" AND m.reference_message_id IS NOT NULL");
        } else {
            p.conditions.push_str(" AND m.reference_message_id IS NULL");
        }
    }

    if let Some(ref v) = filters.source {
        p.conditions
            .push_str(&format!(" AND m.source = ${}", next_n!()));
        p.params.push(BindVal::Text(v.clone()));
    }

    if let Some(ref v) = filters.created_after {
        p.conditions
            .push_str(&format!(" AND m.created_at > ${}", next_n!()));
        p.params.push(BindVal::Timestamp(*v));
    }

    if filters.gps_only.unwrap_or(false) {
        p.joins
            .push_str(" INNER JOIN image_gps g ON m.message_id = g.message_id");
    }

    if include_visible_only {
        p.conditions
            .push_str(" AND m.is_deleted = false AND m.is_recalled = false");
    }

    if needs_user_join && !p.joins.contains("LEFT JOIN users") {
        p.joins
            .push_str(" LEFT JOIN users u ON m.sent_by = u.user_id");
    }
    if needs_room_join && !p.joins.contains("LEFT JOIN rooms") {
        p.joins
            .push_str(" LEFT JOIN rooms r ON m.room_id = r.room_id");
    }

    p
}

fn apply_binds<'q>(
    mut q: sqlx::query::QueryAs<'q, sqlx::Postgres, EnrichedMessage, sqlx::postgres::PgArguments>,
    params: &[BindVal],
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, EnrichedMessage, sqlx::postgres::PgArguments> {
    for p in params {
        q = match p {
            BindVal::Text(v) => q.bind(v.clone()),
            BindVal::Timestamp(v) => q.bind(*v),
            BindVal::Bool(v) => q.bind(*v),
            BindVal::TextArray(v) => q.bind(v.clone()),
        };
    }
    q
}

#[derive(Debug, Clone, Default)]
pub struct MessageFilters {
    pub room_id: Option<String>,
    pub room_ids: Option<Vec<String>>,
    pub account_id: Option<String>,
    pub user_or_account_id: Option<String>,
    pub user_id: Option<String>,
    pub content_types: Option<Vec<String>>,
    pub message_types: Option<Vec<String>>,
    pub search_query: Option<String>,
    pub sender_name: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub has_attachment: Option<bool>,
    pub has_reference: Option<bool>,
    pub source: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub gps_only: Option<bool>,
    pub visible_only: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub limit: i64,
    pub per_page: i64,
    pub cursor: Option<String>,
    pub reverse: bool,
    pub page: Option<i64>,
    pub sort_by: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrichedMessage {
    pub message_id: String,
    pub room_id: String,
    pub sent_at: DateTime<Utc>,
    pub sent_by: String,
    pub content_type: String,
    pub content_text: Option<String>,
    pub content_tsv: Option<String>,
    pub attachment_url: Option<String>,
    pub attachment_file: Option<String>,
    pub sticker_id: Option<String>,
    pub alt_text: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub raw_data: serde_json::Value,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<String>,
    pub is_recalled: bool,
    pub is_edited: bool,
    pub history: Option<serde_json::Value>,
    pub reference_message_id: Option<String>,
    pub reference_data: Option<serde_json::Value>,
    pub user_display_name: Option<String>,
    pub user_avatar_url: Option<String>,
    pub room_title: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone)]
pub struct MessageContextResult {
    pub messages: Vec<EnrichedMessage>,
    pub before_count: i64,
    pub after_count: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageCounts {
    pub total_messages: i64,
    pub deleted_messages: i64,
    pub recalled_messages: i64,
    pub edited_messages: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageStats {
    pub total: i64,
    pub deleted: i64,
    pub recalled: i64,
    pub edited: i64,
    pub by_content_type: HashMap<String, i64>,
    pub by_room: Option<HashMap<String, i64>>,
}

#[derive(Debug)]
enum Cursor {
    TwoPart(DateTime<Utc>, String),
    ThreePart(DateTime<Utc>, DateTime<Utc>, String),
}

fn decode_cursor(cursor: &str) -> std::result::Result<Cursor, LiliumError> {
    let parts: Vec<&str> = cursor.splitn(3, '|').collect();
    match parts.len() {
        2 => {
            let sent_at: DateTime<Utc> = parts[0].parse().map_err(|_| {
                LiliumError::domain_service_with_code(
                    "MESSAGE_INVALID_CURSOR",
                    "Invalid cursor timestamp",
                )
            })?;
            Ok(Cursor::TwoPart(sent_at, parts[1].to_string()))
        }
        3 => {
            let sent_at: DateTime<Utc> = parts[0].parse().map_err(|_| {
                LiliumError::domain_service_with_code(
                    "MESSAGE_INVALID_CURSOR",
                    "Invalid cursor timestamp",
                )
            })?;
            let created_at: DateTime<Utc> = parts[1].parse().map_err(|_| {
                LiliumError::domain_service_with_code(
                    "MESSAGE_INVALID_CURSOR",
                    "Invalid cursor created_at",
                )
            })?;
            Ok(Cursor::ThreePart(sent_at, created_at, parts[2].to_string()))
        }
        _ => Err(LiliumError::domain_service_with_code(
            "MESSAGE_INVALID_CURSOR",
            "Invalid cursor format: expected 2 or 3 parts",
        )),
    }
}

fn encode_cursor_two(sent_at: DateTime<Utc>, message_id: &str) -> String {
    format!("{}|{}", sent_at.to_rfc3339(), message_id)
}

fn encode_cursor_three(
    sent_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    message_id: &str,
) -> String {
    format!(
        "{}|{}|{}",
        sent_at.to_rfc3339(),
        created_at.to_rfc3339(),
        message_id
    )
}

impl<'a> MessageService<'a> {
    #[instrument(skip(session))]
    pub fn new(session: DbSessionContext<'a>) -> Self {
        Self { session }
    }

    #[instrument(skip(self, filters, pagination), fields(enriched))]
    pub async fn get_messages(
        &mut self,
        filters: &MessageFilters,
        pagination: &PaginationParams,
        enriched: bool,
    ) -> crate::Result<PaginatedResult<EnrichedMessage>> {
        let per_page = if pagination.per_page > 0 {
            pagination.per_page
        } else {
            pagination.limit
        }
        .min(200);

        let query_parts = build_query_parts(filters, filters.visible_only.unwrap_or(false));

        let select_clause = if enriched {
            SELECT_ENRICHED
        } else {
            SELECT_BASE
        };
        let enriched_joins = if enriched {
            if !query_parts.joins.contains("LEFT JOIN users") {
                " LEFT JOIN users u ON m.sent_by = u.user_id"
            } else {
                ""
            }
            .to_string()
                + if !query_parts.joins.contains("LEFT JOIN rooms") {
                    " LEFT JOIN rooms r ON m.room_id = r.room_id"
                } else {
                    ""
                }
        } else {
            String::new()
        };

        let sort_column = if pagination.sort_by.as_deref() == Some("created_at") {
            "m.created_at"
        } else {
            "m.sent_at"
        };

        let mut sql = format!(
            "SELECT {} FROM messages m{}{} WHERE 1=1{}",
            select_clause, query_parts.joins, enriched_joins, query_parts.conditions
        );

        let mut param_idx = if query_parts.params.is_empty() {
            1
        } else {
            query_parts.params.len() as u32 + 1
        };

        let use_three_part = if pagination.sort_by.as_deref() != Some("created_at") {
            if let Some(ref cursor) = pagination.cursor {
                let parts: Vec<&str> = cursor.split('|').collect();
                parts.len() == 3
            } else {
                false
            }
        } else {
            false
        };

        if let Some(ref cursor) = pagination.cursor {
            let decoded = decode_cursor(cursor)?;

            match (&decoded, use_three_part) {
                (Cursor::ThreePart(..), true) => {
                    let p1 = param_idx;
                    param_idx += 1;
                    let p2 = param_idx;
                    param_idx += 1;
                    let p3 = param_idx;

                    let cmp = if pagination.reverse { ">" } else { "<" };
                    sql.push_str(&format!(
                        " AND ((m.sent_at, m.created_at, m.message_id) {cmp} (${p1}, ${p2}, ${p3}))"
                    ));
                }
                _ => {
                    let p1 = param_idx;
                    param_idx += 1;
                    let p2 = param_idx;
                    param_idx += 1;
                    let p3 = param_idx;

                    if pagination.reverse {
                        sql.push_str(&format!(
                            " AND (m.{sort} > ${p1} OR (m.{sort} = ${p2} AND m.message_id > ${p3}))",
                            sort = sort_column
                        ));
                    } else {
                        sql.push_str(&format!(
                            " AND (m.{sort} < ${p1} OR (m.{sort} = ${p2} AND m.message_id < ${p3}))",
                            sort = sort_column
                        ));
                    }
                }
            }
        }

        if pagination.page.unwrap_or(1) > 1 && pagination.cursor.is_none() {
            param_idx += 1;
            sql.push_str(&format!(" OFFSET ${}", param_idx));
        }

        if use_three_part {
            if pagination.reverse {
                sql.push_str(" ORDER BY m.sent_at ASC, m.created_at ASC, m.message_id ASC");
            } else {
                sql.push_str(" ORDER BY m.sent_at DESC, m.created_at DESC, m.message_id DESC");
            }
        } else if pagination.reverse {
            sql.push_str(&format!(" ORDER BY {} ASC, m.message_id ASC", sort_column));
        } else {
            sql.push_str(&format!(
                " ORDER BY {} DESC, m.message_id DESC",
                sort_column
            ));
        }

        let limit_param = param_idx;
        sql.push_str(&format!(" LIMIT ${}", limit_param));

        let mut q = sqlx::query_as::<_, EnrichedMessage>(&sql);
        q = apply_binds(q, &query_parts.params);

        if let Some(ref cursor) = pagination.cursor {
            let decoded = decode_cursor(cursor)?;
            match (&decoded, use_three_part) {
                (Cursor::ThreePart(sa, ca, mid), true) => {
                    q = q.bind(*sa).bind(*ca).bind(mid.clone());
                    q = q.bind(*sa).bind(*ca).bind(mid.clone());
                }
                (Cursor::TwoPart(sa, mid), _) => {
                    q = q.bind(*sa).bind(mid.clone()).bind(mid.clone());
                }
                _ => {
                    let (sa, mid) = match decoded {
                        Cursor::TwoPart(s, m) => (s, m),
                        Cursor::ThreePart(s, _, m) => (s, m),
                    };
                    q = q.bind(sa).bind(mid.clone()).bind(mid.clone());
                }
            }
        }

        if pagination.page.unwrap_or(1) > 1 && pagination.cursor.is_none() {
            let pp = pagination.page.unwrap();
            let offset = (pp - 1) * pagination.per_page;
            q = q.bind(offset);
        }

        q = q.bind(per_page + 1);
        let mut messages = q.fetch_all(self.session.as_mut()).await?;

        let has_more = messages.len() as i64 > per_page;
        if has_more {
            messages.pop();
        }

        let next_cursor = if has_more {
            let last = messages.last().unwrap();
            if use_three_part {
                Some(encode_cursor_three(
                    last.sent_at,
                    last.created_at,
                    &last.message_id,
                ))
            } else {
                Some(encode_cursor_two(last.sent_at, &last.message_id))
            }
        } else {
            None
        };

        Ok(PaginatedResult {
            data: messages,
            next_cursor,
            has_more,
        })
    }

    #[instrument(skip(self), fields(message_id = %message_id, enriched))]
    pub async fn get_by_id(
        &mut self,
        message_id: &str,
        enriched: bool,
    ) -> Result<Option<EnrichedMessage>> {
        let select_clause = if enriched {
            SELECT_ENRICHED
        } else {
            SELECT_BASE
        };
        let join_clause = if enriched {
            " LEFT JOIN users u ON m.sent_by = u.user_id LEFT JOIN rooms r ON m.room_id = r.room_id"
        } else {
            ""
        };

        sqlx::query_as::<_, EnrichedMessage>(&format!(
            "SELECT {} FROM messages m{} WHERE m.message_id = $1 ORDER BY m.sent_at DESC LIMIT 1",
            select_clause, join_clause,
        ))
        .bind(message_id)
        .fetch_optional(self.session.as_mut())
        .await
        .map_err(anyhow::Error::from)
    }

    #[instrument(skip(self), fields(message_id = %message_id, sent_at = %sent_at, enriched))]
    pub async fn get_by_id_at(
        &mut self,
        message_id: &str,
        sent_at: DateTime<Utc>,
        enriched: bool,
    ) -> Result<Option<EnrichedMessage>> {
        let select_clause = if enriched {
            SELECT_ENRICHED
        } else {
            SELECT_BASE
        };
        let join_clause = if enriched {
            " LEFT JOIN users u ON m.sent_by = u.user_id LEFT JOIN rooms r ON m.room_id = r.room_id"
        } else {
            ""
        };

        sqlx::query_as::<_, EnrichedMessage>(&format!(
            "SELECT {} FROM messages m{} WHERE m.message_id = $1 AND m.sent_at = $2",
            select_clause, join_clause,
        ))
        .bind(message_id)
        .bind(sent_at)
        .fetch_optional(self.session.as_mut())
        .await
        .map_err(anyhow::Error::from)
    }

    #[instrument(skip(self), fields(message_id = %message_id, before_count, after_count))]
    pub async fn get_context(
        &mut self,
        message_id: &str,
        before_count: i64,
        after_count: i64,
    ) -> Result<Option<MessageContextResult>> {
        let anchor = sqlx::query_as::<_, Message>(&format!(
            "SELECT {} FROM messages WHERE message_id = $1 ORDER BY sent_at DESC LIMIT 1",
            SELECT_MESSAGE_COLS
        ))
        .bind(message_id)
        .fetch_optional(self.session.as_mut())
        .await?;

        let anchor = match anchor {
            Some(m) => m,
            None => return Ok(None),
        };

        let before_limit = before_count.min(50);
        let after_limit = after_count.min(50);

        let before = sqlx::query_as::<_, EnrichedMessage>(&format!(
            "SELECT {} FROM messages m{} WHERE m.room_id = $1 AND (m.sent_at < $2 OR (m.sent_at = $2 AND m.message_id < $3)) ORDER BY m.sent_at DESC, m.message_id DESC LIMIT $4",
            SELECT_ENRICHED,
            " LEFT JOIN users u ON m.sent_by = u.user_id LEFT JOIN rooms r ON m.room_id = r.room_id",
        ))
        .bind(&anchor.room_id)
        .bind(anchor.sent_at)
        .bind(&anchor.message_id)
        .bind(before_limit)
        .fetch_all(self.session.as_mut())
        .await?;

        let after = sqlx::query_as::<_, EnrichedMessage>(&format!(
            "SELECT {} FROM messages m{} WHERE m.room_id = $1 AND (m.sent_at > $2 OR (m.sent_at = $2 AND m.message_id > $3)) ORDER BY m.sent_at ASC, m.message_id ASC LIMIT $4",
            SELECT_ENRICHED,
            " LEFT JOIN users u ON m.sent_by = u.user_id LEFT JOIN rooms r ON m.room_id = r.room_id",
        ))
        .bind(&anchor.room_id)
        .bind(anchor.sent_at)
        .bind(&anchor.message_id)
        .bind(after_limit)
        .fetch_all(self.session.as_mut())
        .await?;

        let anchor_enriched = self.enrich_single(&anchor).await?;
        let before_count_actual = before.len() as i64;
        let after_count_actual = after.len() as i64;

        let mut messages: Vec<EnrichedMessage> = Vec::new();
        for msg in before.into_iter().rev() {
            messages.push(msg);
        }
        messages.push(anchor_enriched);
        messages.extend(after);

        Ok(Some(MessageContextResult {
            messages,
            before_count: before_count_actual,
            after_count: after_count_actual,
        }))
    }

    #[instrument(skip(self), fields(message_id = %message_id, count))]
    pub async fn get_before(
        &mut self,
        message_id: &str,
        count: i64,
    ) -> Result<Vec<EnrichedMessage>> {
        let target = self.get_by_id(message_id, false).await?;
        let target = match target {
            Some(m) => m,
            None => return Ok(Vec::new()),
        };

        let limit = count.min(50);

        let mut messages = sqlx::query_as::<_, EnrichedMessage>(&format!(
            "SELECT {} FROM messages m{} WHERE m.room_id = $1 AND (m.sent_at < $2 OR (m.sent_at = $2 AND m.message_id < $3)) ORDER BY m.sent_at DESC, m.message_id DESC LIMIT $4",
            SELECT_ENRICHED,
            " LEFT JOIN users u ON m.sent_by = u.user_id LEFT JOIN rooms r ON m.room_id = r.room_id",
        ))
        .bind(&target.room_id)
        .bind(target.sent_at)
        .bind(&target.message_id)
        .bind(limit)
        .fetch_all(self.session.as_mut())
        .await?;

        messages.reverse();
        Ok(messages)
    }

    #[instrument(skip(self), fields(message_id = %message_id, count))]
    pub async fn get_after(
        &mut self,
        message_id: &str,
        count: i64,
    ) -> Result<Vec<EnrichedMessage>> {
        let target = self.get_by_id(message_id, false).await?;
        let target = match target {
            Some(m) => m,
            None => return Ok(Vec::new()),
        };

        let limit = count.min(50);

        sqlx::query_as::<_, EnrichedMessage>(&format!(
            "SELECT {} FROM messages m{} WHERE m.room_id = $1 AND (m.sent_at > $2 OR (m.sent_at = $2 AND m.message_id > $3)) ORDER BY m.sent_at ASC, m.message_id ASC LIMIT $4",
            SELECT_ENRICHED,
            " LEFT JOIN users u ON m.sent_by = u.user_id LEFT JOIN rooms r ON m.room_id = r.room_id",
        ))
        .bind(&target.room_id)
        .bind(target.sent_at)
        .bind(&target.message_id)
        .bind(limit)
        .fetch_all(self.session.as_mut())
        .await
        .map_err(anyhow::Error::from)
    }

    async fn enrich_single(&mut self, message: &Message) -> Result<EnrichedMessage> {
        let user_data = sqlx::query_as::<_, (Option<String>, Option<String>)>(
            "SELECT full_name, avatar_url FROM users WHERE user_id = $1",
        )
        .bind(&message.sent_by)
        .fetch_optional(self.session.as_mut())
        .await?;

        let room_data =
            sqlx::query_as::<_, (Option<String>,)>("SELECT title FROM rooms WHERE room_id = $1")
                .bind(&message.room_id)
                .fetch_optional(self.session.as_mut())
                .await?;

        let (user_display_name, user_avatar_url) = user_data.unwrap_or((None, None));
        let room_title = room_data.and_then(|(t,)| t);

        Ok(EnrichedMessage {
            message_id: message.message_id.clone(),
            room_id: message.room_id.clone(),
            sent_at: message.sent_at,
            sent_by: message.sent_by.clone(),
            content_type: message.content_type.clone(),
            content_text: message.content_text.clone(),
            content_tsv: message.content_tsv.clone(),
            attachment_url: message.attachment_url.clone(),
            attachment_file: message.attachment_file.clone(),
            sticker_id: message.sticker_id.clone(),
            alt_text: message.alt_text.clone(),
            metadata: message.metadata.clone(),
            raw_data: message.raw_data.clone(),
            source: message.source.clone(),
            created_at: message.created_at,
            updated_at: message.updated_at,
            is_deleted: message.is_deleted,
            deleted_at: message.deleted_at,
            deleted_by: message.deleted_by.clone(),
            is_recalled: message.is_recalled,
            is_edited: message.is_edited,
            history: message.history.clone(),
            reference_message_id: message.reference_message_id.clone(),
            reference_data: message.reference_data.clone(),
            user_display_name,
            user_avatar_url,
            room_title,
        })
    }

    #[instrument(skip(self, messages), fields(message_count = messages.len()))]
    pub async fn enrich_batch(&mut self, messages: &[Message]) -> Result<Vec<EnrichedMessage>> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let user_ids: Vec<String> = messages
            .iter()
            .map(|m| m.sent_by.clone())
            .filter(|id| !id.is_empty())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let room_ids: Vec<String> = messages
            .iter()
            .map(|m| m.room_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let users_map: HashMap<String, (Option<String>, Option<String>)> = if !user_ids.is_empty() {
            let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
                "SELECT user_id, full_name, avatar_url FROM users WHERE user_id = ANY($1)",
            )
            .bind(&user_ids)
            .fetch_all(self.session.as_mut())
            .await?;
            rows.into_iter()
                .map(|(id, name, avatar)| (id, (name, avatar)))
                .collect()
        } else {
            HashMap::new()
        };

        let rooms_map: HashMap<String, String> = if !room_ids.is_empty() {
            let rows = sqlx::query_as::<_, (String, String)>(
                "SELECT room_id, title FROM rooms WHERE room_id = ANY($1)",
            )
            .bind(&room_ids)
            .fetch_all(self.session.as_mut())
            .await?;
            rows.into_iter().collect()
        } else {
            HashMap::new()
        };

        let enriched = messages
            .iter()
            .map(|msg| {
                let (user_display_name, user_avatar_url) =
                    users_map.get(&msg.sent_by).cloned().unwrap_or((None, None));
                let room_title = rooms_map.get(&msg.room_id).cloned();
                EnrichedMessage {
                    message_id: msg.message_id.clone(),
                    room_id: msg.room_id.clone(),
                    sent_at: msg.sent_at,
                    sent_by: msg.sent_by.clone(),
                    content_type: msg.content_type.clone(),
                    content_text: msg.content_text.clone(),
                    content_tsv: msg.content_tsv.clone(),
                    attachment_url: msg.attachment_url.clone(),
                    attachment_file: msg.attachment_file.clone(),
                    sticker_id: msg.sticker_id.clone(),
                    alt_text: msg.alt_text.clone(),
                    metadata: msg.metadata.clone(),
                    raw_data: msg.raw_data.clone(),
                    source: msg.source.clone(),
                    created_at: msg.created_at,
                    updated_at: msg.updated_at,
                    is_deleted: msg.is_deleted,
                    deleted_at: msg.deleted_at,
                    deleted_by: msg.deleted_by.clone(),
                    is_recalled: msg.is_recalled,
                    is_edited: msg.is_edited,
                    history: msg.history.clone(),
                    reference_message_id: msg.reference_message_id.clone(),
                    reference_data: msg.reference_data.clone(),
                    user_display_name,
                    user_avatar_url,
                    room_title,
                }
            })
            .collect();

        Ok(enriched)
    }

    #[instrument(skip(self), fields(room_id = ?room_id, limit))]
    pub async fn get_deleted_messages(
        &mut self,
        room_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Message>> {
        let mut sql = format!(
            "SELECT {} FROM messages WHERE (is_deleted = true OR is_recalled = true)",
            SELECT_MESSAGE_COLS
        );
        let mut param_idx = 0;

        if room_id.is_some() {
            param_idx += 1;
            sql.push_str(&format!(" AND room_id = ${}", param_idx));
        }

        param_idx += 1;
        sql.push_str(&format!(
            " ORDER BY deleted_at DESC NULLS LAST LIMIT ${}",
            param_idx
        ));

        let mut q = sqlx::query_as::<_, Message>(&sql);
        if let Some(rid) = room_id {
            q = q.bind(rid);
        }
        q = q.bind(limit);
        let rows = q.fetch_all(self.session.as_mut()).await?;
        Ok(rows)
    }

    #[instrument(skip(self), fields(room_id = %room_id))]
    pub async fn get_room_stats(&mut self, room_id: &str) -> Result<MessageStats> {
        let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            "SELECT COUNT(*)::bigint,
                    COALESCE(SUM(CASE WHEN is_deleted THEN 1 ELSE 0 END), 0)::bigint,
                    COALESCE(SUM(CASE WHEN is_recalled THEN 1 ELSE 0 END), 0)::bigint,
                    COALESCE(SUM(CASE WHEN is_edited THEN 1 ELSE 0 END), 0)::bigint
             FROM messages WHERE room_id = $1",
        )
        .bind(room_id)
        .fetch_one(self.session.as_mut())
        .await?;

        let type_rows = sqlx::query_as::<_, (String, i64)>(
            "SELECT content_type, COUNT(*)::bigint FROM messages WHERE room_id = $1 GROUP BY content_type",
        )
        .bind(room_id)
        .fetch_all(self.session.as_mut())
        .await?;

        Ok(MessageStats {
            total: row.0,
            deleted: row.1,
            recalled: row.2,
            edited: row.3,
            by_content_type: type_rows.into_iter().collect(),
            by_room: None,
        })
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_user_stats(&mut self, user_id: &str) -> Result<MessageStats> {
        let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            "SELECT COUNT(*)::bigint,
                    COALESCE(SUM(CASE WHEN is_deleted THEN 1 ELSE 0 END), 0)::bigint,
                    COALESCE(SUM(CASE WHEN is_recalled THEN 1 ELSE 0 END), 0)::bigint,
                    COALESCE(SUM(CASE WHEN is_edited THEN 1 ELSE 0 END), 0)::bigint
             FROM messages WHERE sent_by = $1",
        )
        .bind(user_id)
        .fetch_one(self.session.as_mut())
        .await?;

        let type_rows = sqlx::query_as::<_, (String, i64)>(
            "SELECT content_type, COUNT(*)::bigint FROM messages WHERE sent_by = $1 GROUP BY content_type",
        )
        .bind(user_id)
        .fetch_all(self.session.as_mut())
        .await?;

        let room_rows = sqlx::query_as::<_, (String, i64)>(
            "SELECT room_id, COUNT(*)::bigint FROM messages WHERE sent_by = $1 GROUP BY room_id ORDER BY COUNT(*) DESC LIMIT 10",
        )
        .bind(user_id)
        .fetch_all(self.session.as_mut())
        .await?;

        Ok(MessageStats {
            total: row.0,
            deleted: row.1,
            recalled: row.2,
            edited: row.3,
            by_content_type: type_rows.into_iter().collect(),
            by_room: Some(room_rows.into_iter().collect()),
        })
    }

    #[instrument(skip(self), fields(message_id = %message_id))]
    pub async fn message_exists(&mut self, message_id: &str) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM messages WHERE message_id = $1)",
        )
        .bind(message_id)
        .fetch_one(self.session.as_mut())
        .await?;
        Ok(exists)
    }

    #[instrument(skip(self, message), fields(message_id = %message.message_id))]
    pub async fn create_message(&mut self, message: &Message) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO messages (message_id, room_id, sent_at, sent_by, content_type, content_text,
               attachment_url, attachment_file, sticker_id, alt_text, metadata, raw_data, source,
               created_at, updated_at, is_deleted, deleted_at, deleted_by, is_recalled, is_edited,
               history, reference_message_id, reference_data)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23)"#,
        )
        .bind(&message.message_id)
        .bind(&message.room_id)
        .bind(message.sent_at)
        .bind(&message.sent_by)
        .bind(&message.content_type)
        .bind(&message.content_text)
        .bind(&message.attachment_url)
        .bind(&message.attachment_file)
        .bind(&message.sticker_id)
        .bind(&message.alt_text)
        .bind(&message.metadata)
        .bind(&message.raw_data)
        .bind(&message.source)
        .bind(message.created_at)
        .bind(message.updated_at)
        .bind(message.is_deleted)
        .bind(message.deleted_at)
        .bind(&message.deleted_by)
        .bind(message.is_recalled)
        .bind(message.is_edited)
        .bind(&message.history)
        .bind(&message.reference_message_id)
        .bind(&message.reference_data)
        .execute(self.session.as_mut())
        .await?;
        Ok(())
    }

    #[instrument(skip(self, message), fields(message_id = %message.message_id))]
    pub async fn create_message_if_missing(&mut self, message: &Message) -> Result<bool> {
        lilium_database::queries::messages::create_message_if_missing(
            self.session.as_mut(),
            message,
        )
        .await
    }

    #[instrument(skip(self, messages), fields(message_count = messages.len()))]
    pub async fn batch_create_if_missing(
        &mut self,
        messages: &[Message],
    ) -> Result<Vec<(String, DateTime<Utc>)>> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let mut sql = String::from(
            "INSERT INTO messages (message_id, room_id, sent_at, sent_by, content_type, content_text, raw_data, source, created_at, is_deleted, is_recalled, is_edited, history) VALUES ",
        );
        let mut param_idx = 0u32;
        let mut value_rows: Vec<String> = Vec::new();

        for _i in 0..messages.len() {
            let base = param_idx;
            let cols = [
                base + 1,
                base + 2,
                base + 3,
                base + 4,
                base + 5,
                base + 6,
                base + 7,
                base + 8,
                base + 9,
                base + 10,
                base + 11,
                base + 12,
                base + 13,
            ];
            value_rows.push(format!(
                "(${},{},{},{},{},{},{},{},{},{},{},{},{})",
                cols[0],
                cols[1],
                cols[2],
                cols[3],
                cols[4],
                cols[5],
                cols[6],
                cols[7],
                cols[8],
                cols[9],
                cols[10],
                cols[11],
                cols[12],
            ));
            param_idx += 13;
        }

        sql.push_str(&value_rows.join(", "));
        sql.push_str(" ON CONFLICT (message_id, sent_at) DO NOTHING RETURNING message_id, sent_at");

        let mut q = sqlx::query_as::<_, (String, DateTime<Utc>)>(&sql);
        for msg in messages {
            q = q
                .bind(&msg.message_id)
                .bind(&msg.room_id)
                .bind(msg.sent_at)
                .bind(&msg.sent_by)
                .bind(&msg.content_type)
                .bind(&msg.content_text)
                .bind(&msg.raw_data)
                .bind(&msg.source)
                .bind(msg.created_at)
                .bind(msg.is_deleted)
                .bind(msg.is_recalled)
                .bind(msg.is_edited)
                .bind(&msg.history);
        }

        let rows = q.fetch_all(self.session.as_mut()).await?;
        Ok(rows)
    }

    #[instrument(skip(self, message), fields(message_id = %message.message_id))]
    pub async fn update_message(&mut self, message: &Message) -> Result<()> {
        sqlx::query(
            r#"UPDATE messages SET
               content_type = $3, content_text = $4, attachment_url = $5,
               attachment_file = $6, sticker_id = $7, alt_text = $8,
               metadata = $9, updated_at = NOW(), is_edited = $10, history = $11
               WHERE message_id = $1 AND sent_at = $2"#,
        )
        .bind(&message.message_id)
        .bind(message.sent_at)
        .bind(&message.content_type)
        .bind(&message.content_text)
        .bind(&message.attachment_url)
        .bind(&message.attachment_file)
        .bind(&message.sticker_id)
        .bind(&message.alt_text)
        .bind(&message.metadata)
        .bind(message.is_edited)
        .bind(&message.history)
        .execute(self.session.as_mut())
        .await?;
        Ok(())
    }

    #[instrument(skip(self, payload), fields(message_id = %message_id))]
    pub async fn update_message_from_payload(
        &mut self,
        message_id: &str,
        payload: &serde_json::Value,
    ) -> Result<()> {
        if let Some(content) = payload.get("message").and_then(|m| m.get("content")) {
            if content.get("type").and_then(|v| v.as_str()) == Some("recalled") {
                lilium_database::queries::messages::mark_recalled(self.session.as_mut(), message_id)
                    .await
            } else {
                let sent_at = payload
                    .get("message")
                    .and_then(|m| m.get("sent_at").or_else(|| m.get("sentAt")))
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));

                if let Some(sent_at) = sent_at {
                    let existing = lilium_database::queries::messages::get_by_id_at(
                        self.session.as_mut(),
                        message_id,
                        sent_at,
                    )
                    .await?;
                    if existing.is_none() {
                        return Ok(());
                    }
                    if let Some(text) = content.get("text").and_then(|v| v.as_str()) {
                        lilium_database::queries::messages::update_content(
                            self.session.as_mut(),
                            message_id,
                            text,
                        )
                        .await
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self), fields(message_id = %message_id, has_deleted_by = deleted_by.is_some()))]
    pub async fn mark_deleted(&mut self, message_id: &str, deleted_by: Option<&str>) -> Result<()> {
        sqlx::query(
            r#"UPDATE messages
               SET is_deleted = true, deleted_at = NOW(), deleted_by = $2, updated_at = NOW()
               WHERE message_id = $1"#,
        )
        .bind(message_id)
        .bind(deleted_by)
        .execute(self.session.as_mut())
        .await?;
        Ok(())
    }

    #[instrument(skip(self, message_ids), fields(message_count = message_ids.len(), has_deleted_by = deleted_by.is_some()))]
    pub async fn mark_deleted_batch(
        &mut self,
        message_ids: &[String],
        deleted_by: Option<&str>,
    ) -> Result<i64> {
        if message_ids.is_empty() {
            return Ok(0);
        }
        let result = sqlx::query(
            r#"UPDATE messages
               SET is_deleted = true, deleted_at = NOW(), deleted_by = $2, updated_at = NOW()
               WHERE message_id = ANY($1)"#,
        )
        .bind(message_ids)
        .bind(deleted_by)
        .execute(self.session.as_mut())
        .await?;
        Ok(result.rows_affected() as i64)
    }

    #[instrument(skip(self), fields(message_id = %message_id))]
    pub async fn mark_recalled(&mut self, message_id: &str) -> Result<()> {
        lilium_database::queries::messages::mark_recalled(self.session.as_mut(), message_id).await
    }

    #[instrument(skip(self, message_ids), fields(message_count = message_ids.len()))]
    pub async fn mark_recalled_batch(&mut self, message_ids: &[String]) -> Result<i64> {
        if message_ids.is_empty() {
            return Ok(0);
        }
        let result = sqlx::query(
            "UPDATE messages SET is_recalled = true, updated_at = NOW() WHERE message_id = ANY($1)",
        )
        .bind(message_ids)
        .execute(self.session.as_mut())
        .await?;
        Ok(result.rows_affected() as i64)
    }

    #[instrument(skip(self, messages), fields(message_count = messages.len()))]
    pub async fn batch_create(&mut self, messages: &[Message]) -> Result<i64> {
        if messages.is_empty() {
            return Ok(0);
        }

        let mut sql = String::from(
            "INSERT INTO messages (message_id, room_id, sent_at, sent_by, content_type, content_text, attachment_url, attachment_file, sticker_id, alt_text, metadata, raw_data, source, created_at, updated_at, is_deleted, deleted_at, deleted_by, is_recalled, is_edited, history, reference_message_id, reference_data) VALUES ",
        );
        let mut param_idx = 0u32;
        let mut value_rows: Vec<String> = Vec::new();
        const COLS_PER_ROW: u32 = 23;

        for _i in 0..messages.len() {
            let base = param_idx;
            let col_nums: Vec<String> = (1..=COLS_PER_ROW)
                .map(|o| format!("${}", base + o))
                .collect();
            value_rows.push(format!("({})", col_nums.join(",")));
            param_idx += COLS_PER_ROW;
        }

        sql.push_str(&value_rows.join(", "));

        let mut q = sqlx::query(&sql);
        for msg in messages {
            q = q
                .bind(&msg.message_id)
                .bind(&msg.room_id)
                .bind(msg.sent_at)
                .bind(&msg.sent_by)
                .bind(&msg.content_type)
                .bind(&msg.content_text)
                .bind(&msg.attachment_url)
                .bind(&msg.attachment_file)
                .bind(&msg.sticker_id)
                .bind(&msg.alt_text)
                .bind(&msg.metadata)
                .bind(&msg.raw_data)
                .bind(&msg.source)
                .bind(msg.created_at)
                .bind(msg.updated_at)
                .bind(msg.is_deleted)
                .bind(msg.deleted_at)
                .bind(&msg.deleted_by)
                .bind(msg.is_recalled)
                .bind(msg.is_edited)
                .bind(&msg.history)
                .bind(&msg.reference_message_id)
                .bind(&msg.reference_data);
        }

        let result = q.execute(self.session.as_mut()).await?;
        Ok(result.rows_affected() as i64)
    }

    #[instrument(skip(self), fields(room_id = %room_id))]
    pub async fn get_latest_message_time(
        &mut self,
        room_id: &str,
    ) -> Result<Option<DateTime<Utc>>> {
        sqlx::query_scalar(
            "SELECT sent_at FROM messages WHERE room_id = $1 ORDER BY sent_at DESC LIMIT 1",
        )
        .bind(room_id)
        .fetch_optional(self.session.as_mut())
        .await
        .map_err(anyhow::Error::from)
    }

    #[instrument(skip(self), fields(room_id = %room_id))]
    pub async fn get_earliest_message_time(
        &mut self,
        room_id: &str,
    ) -> Result<Option<DateTime<Utc>>> {
        sqlx::query_scalar(
            "SELECT sent_at FROM messages WHERE room_id = $1 ORDER BY sent_at ASC LIMIT 1",
        )
        .bind(room_id)
        .fetch_optional(self.session.as_mut())
        .await
        .map_err(anyhow::Error::from)
    }

    #[instrument(skip(self, filters), fields(has_gps_only = filters.gps_only.unwrap_or(false)))]
    pub async fn count_messages(&mut self, filters: &MessageFilters) -> Result<MessageCounts> {
        let query_parts = build_query_parts(filters, false);

        if query_parts.params.is_empty()
            && query_parts.conditions.is_empty()
            && !filters.gps_only.unwrap_or(false)
        {
            let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
                "SELECT COALESCE(SUM(message_count), 0)::bigint,
                        COALESCE(SUM(deleted_count), 0)::bigint,
                        COALESCE(SUM(recalled_count), 0)::bigint,
                        COALESCE(SUM(edited_count), 0)::bigint
                 FROM rooms",
            )
            .fetch_one(self.session.as_mut())
            .await?;

            return Ok(MessageCounts {
                total_messages: row.0,
                deleted_messages: row.1,
                recalled_messages: row.2,
                edited_messages: row.3,
            });
        }

        let sql = format!(
            "SELECT COUNT(*)::bigint,
                    COALESCE(SUM(CASE WHEN m.is_deleted THEN 1 ELSE 0 END), 0)::bigint,
                    COALESCE(SUM(CASE WHEN m.is_recalled THEN 1 ELSE 0 END), 0)::bigint,
                    COALESCE(SUM(CASE WHEN m.is_edited THEN 1 ELSE 0 END), 0)::bigint
             FROM messages m{} WHERE 1=1{}",
            query_parts.joins, query_parts.conditions,
        );

        let mut q = sqlx::query_as::<_, (i64, i64, i64, i64)>(&sql);
        for p in &query_parts.params {
            q = match p {
                BindVal::Text(v) => q.bind(v.clone()),
                BindVal::Timestamp(v) => q.bind(*v),
                BindVal::Bool(v) => q.bind(*v),
                BindVal::TextArray(v) => q.bind(v.clone()),
            };
        }

        let row = q.fetch_one(self.session.as_mut()).await?;

        Ok(MessageCounts {
            total_messages: row.0,
            deleted_messages: row.1,
            recalled_messages: row.2,
            edited_messages: row.3,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for EnrichedMessage {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        use sqlx::Row;
        Ok(Self {
            message_id: row.try_get("message_id")?,
            room_id: row.try_get("room_id")?,
            sent_at: row.try_get("sent_at")?,
            sent_by: row.try_get("sent_by")?,
            content_type: row.try_get("content_type")?,
            content_text: row.try_get("content_text")?,
            content_tsv: row.try_get("content_tsv")?,
            attachment_url: row.try_get("attachment_url")?,
            attachment_file: row.try_get("attachment_file")?,
            sticker_id: row.try_get("sticker_id")?,
            alt_text: row.try_get("alt_text")?,
            metadata: row.try_get("metadata")?,
            raw_data: row.try_get("raw_data")?,
            source: row.try_get("source")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            is_deleted: row.try_get("is_deleted")?,
            deleted_at: row.try_get("deleted_at")?,
            deleted_by: row.try_get("deleted_by")?,
            is_recalled: row.try_get("is_recalled")?,
            is_edited: row.try_get("is_edited")?,
            history: row.try_get("history")?,
            reference_message_id: row.try_get("reference_message_id")?,
            reference_data: row.try_get("reference_data")?,
            user_display_name: row.try_get("user_display_name")?,
            user_avatar_url: row.try_get("user_avatar_url")?,
            room_title: row.try_get("room_title")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    mod message_filters {
        use super::*;

        #[test]
        fn default_all_fields_none() {
            let f = MessageFilters::default();
            assert!(f.room_id.is_none());
            assert!(f.room_ids.is_none());
            assert!(f.account_id.is_none());
            assert!(f.user_or_account_id.is_none());
            assert!(f.user_id.is_none());
            assert!(f.content_types.is_none());
            assert!(f.message_types.is_none());
            assert!(f.search_query.is_none());
            assert!(f.sender_name.is_none());
            assert!(f.start_time.is_none());
            assert!(f.end_time.is_none());
            assert!(f.has_attachment.is_none());
            assert!(f.has_reference.is_none());
            assert!(f.source.is_none());
            assert!(f.created_after.is_none());
            assert!(f.gps_only.is_none());
            assert!(f.visible_only.is_none());
        }

        #[test]
        fn build_query_room_id_generates_condition() {
            let f = MessageFilters {
                room_id: Some("room1".into()),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("m.room_id"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_room_ids_generates_any_condition() {
            let f = MessageFilters {
                room_ids: Some(vec!["r1".into(), "r2".into()]),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("= ANY"));
            assert!(qp.conditions.contains("m.room_id"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_room_ids_empty_skips() {
            let f = MessageFilters {
                room_ids: Some(vec![]),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(!qp.conditions.contains("room_id"));
            assert_eq!(qp.params.len(), 0);
        }

        #[test]
        fn build_query_user_id_generates_condition() {
            let f = MessageFilters {
                user_id: Some("user1".into()),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("m.sent_by"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_content_types_generates_any_condition() {
            let f = MessageFilters {
                content_types: Some(vec!["text".into(), "image".into()]),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("m.content_type"));
            assert!(qp.conditions.contains("= ANY"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_content_types_empty_skips() {
            let f = MessageFilters {
                content_types: Some(vec![]),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(!qp.conditions.contains("content_type"));
            assert_eq!(qp.params.len(), 0);
        }

        #[test]
        fn build_query_message_type_deleted() {
            let f = MessageFilters {
                message_types: Some(vec!["deleted".into()]),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("is_deleted"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_message_type_recalled() {
            let f = MessageFilters {
                message_types: Some(vec!["recalled".into()]),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("is_recalled"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_message_type_edited() {
            let f = MessageFilters {
                message_types: Some(vec!["edited".into()]),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("is_edited"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_message_types_multiple() {
            let f = MessageFilters {
                message_types: Some(vec!["deleted".into(), "recalled".into()]),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains(" OR "));
            assert!(qp.conditions.contains("is_deleted"));
            assert!(qp.conditions.contains("is_recalled"));
            assert_eq!(qp.params.len(), 2);
        }

        #[test]
        fn build_query_start_time_generates_condition() {
            let t = Utc::now();
            let f = MessageFilters {
                start_time: Some(t),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("m.sent_at >="));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_end_time_generates_condition() {
            let t = Utc::now();
            let f = MessageFilters {
                end_time: Some(t),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("m.sent_at <="));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_has_attachment_true() {
            let f = MessageFilters {
                has_attachment: Some(true),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("attachment_file IS NOT NULL"));
            assert_eq!(qp.params.len(), 0);
        }

        #[test]
        fn build_query_has_attachment_false() {
            let f = MessageFilters {
                has_attachment: Some(false),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("attachment_file IS NULL"));
            assert_eq!(qp.params.len(), 0);
        }

        #[test]
        fn build_query_has_reference_true() {
            let f = MessageFilters {
                has_reference: Some(true),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("reference_message_id IS NOT NULL"));
            assert_eq!(qp.params.len(), 0);
        }

        #[test]
        fn build_query_has_reference_false() {
            let f = MessageFilters {
                has_reference: Some(false),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("reference_message_id IS NULL"));
            assert_eq!(qp.params.len(), 0);
        }

        #[test]
        fn build_query_gps_only_adds_join() {
            let f = MessageFilters {
                gps_only: Some(true),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.joins.contains("image_gps"));
        }

        #[test]
        fn build_query_gps_only_false_no_join() {
            let f = MessageFilters {
                gps_only: Some(false),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(!qp.joins.contains("image_gps"));
        }

        #[test]
        fn build_query_visible_only_adds_condition() {
            let f = MessageFilters::default();
            let qp = build_query_parts(&f, true);
            assert!(qp.conditions.contains("is_deleted = false"));
            assert!(qp.conditions.contains("is_recalled = false"));
        }

        #[test]
        fn build_query_combined_filters() {
            let t = Utc::now();
            let f = MessageFilters {
                room_id: Some("room1".into()),
                user_id: Some("user1".into()),
                content_types: Some(vec!["text".into()]),
                start_time: Some(t),
                end_time: Some(t),
                has_attachment: Some(true),
                has_reference: Some(true),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert_eq!(qp.params.len(), 5);
            assert!(qp.conditions.contains("m.room_id"));
            assert!(qp.conditions.contains("m.sent_by"));
            assert!(qp.conditions.contains("m.content_type"));
            assert!(qp.conditions.contains("m.sent_at >="));
            assert!(qp.conditions.contains("m.sent_at <="));
            assert!(qp.conditions.contains("attachment_file IS NOT NULL"));
            assert!(qp.conditions.contains("reference_message_id IS NOT NULL"));
        }

        #[test]
        fn build_query_source_filter() {
            let f = MessageFilters {
                source: Some("spider".into()),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("m.source"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_created_after_filter() {
            let t = Utc::now();
            let f = MessageFilters {
                created_after: Some(t),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("m.created_at >"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_account_id_adds_room_join() {
            let f = MessageFilters {
                account_id: Some("acct1".into()),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.joins.contains("LEFT JOIN rooms"));
            assert!(qp.conditions.contains("account_ids"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_user_or_account_id_adds_room_join() {
            let f = MessageFilters {
                user_or_account_id: Some("id1".into()),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.joins.contains("LEFT JOIN rooms"));
            assert!(qp.conditions.contains("m.sent_by"));
            assert!(qp.conditions.contains("account_ids"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_sender_name_adds_user_join() {
            let f = MessageFilters {
                sender_name: Some("Test".into()),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.joins.contains("LEFT JOIN users"));
            assert!(qp.conditions.contains("name_tsv"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_search_query_uuid_detection() {
            let f = MessageFilters {
                search_query: Some("550e8400-e29b-41d4-a716-446655440000".into()),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("m.message_id"));
            assert!(!qp.conditions.contains("content_tsv"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_search_query_non_uuid_uses_fts() {
            let f = MessageFilters {
                search_query: Some("Hello".into()),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.contains("content_tsv"));
            assert!(qp.conditions.contains("zhparser"));
            assert_eq!(qp.params.len(), 1);
        }

        #[test]
        fn build_query_default_no_filters() {
            let f = MessageFilters::default();
            let qp = build_query_parts(&f, false);
            assert!(qp.conditions.trim().is_empty() || qp.conditions == " AND ".repeat(0));
            assert!(qp.joins.is_empty());
            assert!(qp.params.is_empty());
        }
    }

    mod pagination_params {
        use super::PaginationParams;

        #[test]
        fn default_values() {
            let p = PaginationParams {
                limit: 10,
                per_page: 20,
                cursor: None,
                reverse: false,
                page: None,
                sort_by: None,
            };
            assert_eq!(p.limit, 10);
            assert_eq!(p.per_page, 20);
            assert!(p.cursor.is_none());
            assert!(!p.reverse);
            assert!(p.page.is_none());
            assert!(p.sort_by.is_none());
        }

        #[test]
        fn reverse_true() {
            let p = PaginationParams {
                limit: 0,
                per_page: 50,
                cursor: None,
                reverse: true,
                page: None,
                sort_by: None,
            };
            assert!(p.reverse);
        }

        #[test]
        fn with_cursor() {
            let p = PaginationParams {
                limit: 0,
                per_page: 25,
                cursor: Some("2024-01-01T00:00:00+00:00|msg1".into()),
                reverse: false,
                page: None,
                sort_by: None,
            };
            assert!(p.cursor.is_some());
        }

        #[test]
        fn with_page() {
            let p = PaginationParams {
                limit: 0,
                per_page: 10,
                cursor: None,
                reverse: false,
                page: Some(3),
                sort_by: None,
            };
            assert_eq!(p.page, Some(3));
        }

        #[test]
        fn sort_by_created_at() {
            let p = PaginationParams {
                limit: 0,
                per_page: 30,
                cursor: None,
                reverse: false,
                page: None,
                sort_by: Some("created_at".into()),
            };
            assert_eq!(p.sort_by.as_deref(), Some("created_at"));
        }
    }

    mod cursor_encoding {
        use super::*;
        use chrono::{TimeZone, Utc};

        #[test]
        fn encode_cursor_two_produces_expected_format() {
            let sent_at = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
            let cursor = encode_cursor_two(sent_at, "msg1");
            assert!(cursor.contains("msg1"));
            assert!(cursor.contains("2024-01-01T10:00:00"));
            let parts: Vec<&str> = cursor.split('|').collect();
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[1], "msg1");
        }

        #[test]
        fn encode_cursor_three_produces_expected_format() {
            let sent_at = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
            let created_at = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
            let cursor = encode_cursor_three(sent_at, created_at, "msg1");
            assert!(cursor.contains("msg1"));
            assert!(cursor.contains("2024-01-01T10:00:00"));
            assert!(cursor.contains("2024-01-01T12:00:00"));
            let parts: Vec<&str> = cursor.split('|').collect();
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[2], "msg1");
        }

        #[test]
        fn decode_cursor_two_part() {
            let cursor = "2024-01-01T10:00:00+00:00|msg1";
            let decoded = decode_cursor(cursor).expect("should decode");
            match decoded {
                Cursor::TwoPart(sent_at, msg_id) => {
                    assert_eq!(msg_id, "msg1");
                    assert_eq!(sent_at.to_rfc3339(), "2024-01-01T10:00:00+00:00");
                }
                _ => panic!("expected TwoPart cursor"),
            }
        }

        #[test]
        fn decode_cursor_three_part() {
            let cursor = "2024-01-01T10:00:00+00:00|2024-01-01T12:00:00+00:00|msg1";
            let decoded = decode_cursor(cursor).expect("should decode");
            match decoded {
                Cursor::ThreePart(sent_at, created_at, msg_id) => {
                    assert_eq!(msg_id, "msg1");
                    assert_eq!(sent_at.to_rfc3339(), "2024-01-01T10:00:00+00:00");
                    assert_eq!(created_at.to_rfc3339(), "2024-01-01T12:00:00+00:00");
                }
                _ => panic!("expected ThreePart cursor"),
            }
        }

        #[test]
        fn decode_cursor_invalid_too_many_parts() {
            let cursor = "a|b|c|d";
            let result = decode_cursor(cursor);
            let err = result.expect_err("should reject invalid cursor");
            assert_eq!(err.code(), Some("MESSAGE_INVALID_CURSOR"));
        }

        #[test]
        fn decode_cursor_invalid_timestamp() {
            let cursor = "not-a-date|msg1";
            let result = decode_cursor(cursor);
            let err = result.expect_err("should reject invalid timestamp");
            assert_eq!(err.code(), Some("MESSAGE_INVALID_CURSOR"));
        }

        #[test]
        fn encode_then_decode_cursor_two_is_roundtrip() {
            let sent_at = Utc.with_ymd_and_hms(2024, 6, 15, 8, 30, 0).unwrap();
            let cursor = encode_cursor_two(sent_at, "roundtrip_id");
            let decoded = decode_cursor(&cursor).expect("should decode");
            match decoded {
                Cursor::TwoPart(decoded_sent_at, msg_id) => {
                    assert_eq!(msg_id, "roundtrip_id");
                    assert_eq!(decoded_sent_at, sent_at);
                }
                _ => panic!("expected TwoPart"),
            }
        }

        #[test]
        fn encode_then_decode_cursor_three_is_roundtrip() {
            let sent_at = Utc.with_ymd_and_hms(2024, 6, 15, 8, 30, 0).unwrap();
            let created_at = Utc.with_ymd_and_hms(2024, 6, 15, 9, 0, 0).unwrap();
            let cursor = encode_cursor_three(sent_at, created_at, "roundtrip_id");
            let decoded = decode_cursor(&cursor).expect("should decode");
            match decoded {
                Cursor::ThreePart(decoded_sent_at, decoded_created_at, msg_id) => {
                    assert_eq!(msg_id, "roundtrip_id");
                    assert_eq!(decoded_sent_at, sent_at);
                    assert_eq!(decoded_created_at, created_at);
                }
                _ => panic!("expected ThreePart"),
            }
        }
    }

    mod types_construction {
        use super::*;

        #[test]
        fn message_counts_creation() {
            let c = MessageCounts {
                total_messages: 100,
                deleted_messages: 5,
                recalled_messages: 3,
                edited_messages: 10,
            };
            assert_eq!(c.total_messages, 100);
            assert_eq!(c.deleted_messages, 5);
            assert_eq!(c.recalled_messages, 3);
            assert_eq!(c.edited_messages, 10);
        }

        #[test]
        fn message_stats_creation() {
            let by_type = {
                let mut m = std::collections::HashMap::new();
                m.insert("text".to_string(), 50i64);
                m.insert("image".to_string(), 30i64);
                m
            };
            let s = MessageStats {
                total: 80,
                deleted: 5,
                recalled: 2,
                edited: 3,
                by_content_type: by_type,
                by_room: None,
            };
            assert_eq!(s.total, 80);
            assert_eq!(s.deleted, 5);
            assert_eq!(s.edited, 3);
            assert_eq!(s.by_content_type.get("text"), Some(&50));
        }

        #[test]
        fn message_stats_serializes() {
            let s = MessageStats {
                total: 1,
                deleted: 0,
                recalled: 0,
                edited: 0,
                by_content_type: std::collections::HashMap::new(),
                by_room: None,
            };
            let json = serde_json::to_string(&s).expect("serialize");
            assert!(json.contains("\"total\":1"));
        }

        #[test]
        fn paginated_result_serializes() {
            let result: PaginatedResult<String> = PaginatedResult {
                data: vec!["a".into()],
                next_cursor: Some("cursor1".into()),
                has_more: true,
            };
            let json = serde_json::to_string(&result).expect("serialize");
            assert!(json.contains("cursor1"));
            assert!(json.contains("\"has_more\":true"));
        }

        #[test]
        fn message_counts_serializes() {
            let c = MessageCounts {
                total_messages: 42,
                deleted_messages: 3,
                recalled_messages: 1,
                edited_messages: 7,
            };
            let json = serde_json::to_string(&c).expect("serialize");
            assert!(json.contains("\"total_messages\":42"));
            assert!(json.contains("\"deleted_messages\":3"));
            assert!(json.contains("\"recalled_messages\":1"));
            assert!(json.contains("\"edited_messages\":7"));
        }

        #[test]
        fn message_context_result_construction() {
            let r = MessageContextResult {
                messages: vec![],
                before_count: 3,
                after_count: 2,
            };
            assert_eq!(r.before_count, 3);
            assert_eq!(r.after_count, 2);
            assert!(r.messages.is_empty());
        }
    }

    mod bind_val {
        use super::*;

        #[test]
        fn text_variant() {
            let v = BindVal::Text("hello".into());
            assert!(matches!(v, BindVal::Text(_)));
        }

        #[test]
        fn timestamp_variant() {
            let v = BindVal::Timestamp(Utc::now());
            assert!(matches!(v, BindVal::Timestamp(_)));
        }

        #[test]
        fn bool_variant() {
            let v = BindVal::Bool(true);
            assert!(matches!(v, BindVal::Bool(true)));
        }

        #[test]
        fn text_array_variant() {
            let v = BindVal::TextArray(vec!["a".into()]);
            assert!(matches!(v, BindVal::TextArray(_)));
        }
    }

    mod query_parts {
        use super::QueryParts;

        #[test]
        fn default_empty() {
            let qp = QueryParts::default();
            assert!(qp.conditions.is_empty() || qp.conditions.trim().is_empty());
            assert!(qp.joins.is_empty());
            assert!(qp.params.is_empty());
        }
    }

    mod message_service_integration {
        use super::*;
        use chrono::{TimeZone, Utc};

        fn test_message() -> lilium_models::dzmm::message::Message {
            lilium_models::dzmm::message::Message {
                message_id: "test_msg_1".into(),
                room_id: "test_room".into(),
                sent_at: Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
                sent_by: "test_user".into(),
                content_type: "text".into(),
                content_text: Some("Hello".into()),
                content_tsv: None,
                attachment_url: None,
                attachment_file: None,
                sticker_id: None,
                alt_text: None,
                metadata: None,
                raw_data: serde_json::json!({}),
                source: "spider".into(),
                created_at: Utc::now(),
                updated_at: None,
                is_deleted: false,
                deleted_at: None,
                deleted_by: None,
                is_recalled: false,
                is_edited: false,
                history: None,
                reference_message_id: None,
                reference_data: None,
            }
        }

        #[tokio::test]
        async fn service_struct_can_be_created() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut _svc = MessageService::new(session);
                        Ok(())
                    })
                },
            )
            .await
            .expect("service_struct_can_be_created");
        }

        #[tokio::test]
        async fn get_messages_no_filters() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters::default();
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        assert!(result.data.len() <= 100);
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_messages_no_filters");
        }

        #[tokio::test]
        async fn get_messages_ordered_newest_first() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters::default();
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        for i in 0..result.data.len().saturating_sub(1) {
                            assert!(result.data[i].sent_at >= result.data[i + 1].sent_at);
                        }
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_messages_ordered_newest_first");
        }

        #[tokio::test]
        async fn get_messages_reverse_order() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters::default();
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: true,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        for i in 0..result.data.len().saturating_sub(1) {
                            assert!(result.data[i].sent_at <= result.data[i + 1].sent_at);
                        }
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_messages_reverse_order");
        }

        #[tokio::test]
        async fn filter_by_room() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters {
                            room_id: Some("room1".into()),
                            ..Default::default()
                        };
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        assert!(result.data.iter().all(|m| m.room_id == "room1"));
                        Ok(())
                    })
                },
            )
            .await
            .expect("filter_by_room");
        }

        #[tokio::test]
        async fn filter_by_user() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters {
                            user_id: Some("user1".into()),
                            ..Default::default()
                        };
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        assert!(result.data.iter().all(|m| m.sent_by == "user1"));
                        Ok(())
                    })
                },
            )
            .await
            .expect("filter_by_user");
        }

        #[tokio::test]
        async fn filter_by_content_types() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters {
                            content_types: Some(vec!["text".into()]),
                            ..Default::default()
                        };
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        assert!(result.data.iter().all(|m| m.content_type == "text"));
                        Ok(())
                    })
                },
            )
            .await
            .expect("filter_by_content_types");
        }

        #[tokio::test]
        async fn filter_deleted_messages() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters {
                            message_types: Some(vec!["deleted".into()]),
                            ..Default::default()
                        };
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        assert!(result.data.iter().all(|m| m.is_deleted));
                        Ok(())
                    })
                },
            )
            .await
            .expect("filter_deleted_messages");
        }

        #[tokio::test]
        async fn filter_recalled_messages() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters {
                            message_types: Some(vec!["recalled".into()]),
                            ..Default::default()
                        };
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        assert!(result.data.iter().all(|m| m.is_recalled));
                        Ok(())
                    })
                },
            )
            .await
            .expect("filter_recalled_messages");
        }

        #[tokio::test]
        async fn filter_by_time_range() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let start = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
                        let end = Utc.with_ymd_and_hms(2024, 1, 1, 14, 0, 0).unwrap();
                        let filters = MessageFilters {
                            start_time: Some(start),
                            end_time: Some(end),
                            ..Default::default()
                        };
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        for m in &result.data {
                            assert!(m.sent_at >= start);
                            assert!(m.sent_at <= end);
                        }
                        Ok(())
                    })
                },
            )
            .await
            .expect("filter_by_time_range");
        }

        #[tokio::test]
        async fn filter_has_attachment() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters {
                            has_attachment: Some(true),
                            ..Default::default()
                        };
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        assert!(result.data.iter().all(|m| m.attachment_file.is_some()));
                        Ok(())
                    })
                },
            )
            .await
            .expect("filter_has_attachment");
        }

        #[tokio::test]
        async fn filter_has_reference() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters {
                            has_reference: Some(true),
                            ..Default::default()
                        };
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        assert!(result.data.iter().all(|m| m.reference_message_id.is_some()));
                        Ok(())
                    })
                },
            )
            .await
            .expect("filter_has_reference");
        }

        #[tokio::test]
        async fn filter_combined() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters {
                            room_id: Some("room1".into()),
                            user_id: Some("user1".into()),
                            content_types: Some(vec!["text".into()]),
                            ..Default::default()
                        };
                        let pagination = PaginationParams {
                            limit: 100,
                            per_page: 100,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        for m in &result.data {
                            assert_eq!(m.room_id, "room1");
                            assert_eq!(m.sent_by, "user1");
                            assert_eq!(m.content_type, "text");
                        }
                        Ok(())
                    })
                },
            )
            .await
            .expect("filter_combined");
        }

        #[tokio::test]
        async fn pagination_first_page() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters::default();
                        let pagination = PaginationParams {
                            limit: 3,
                            per_page: 3,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        assert!(result.data.len() <= 3);
                        Ok(())
                    })
                },
            )
            .await
            .expect("pagination_first_page");
        }

        #[tokio::test]
        async fn pagination_with_cursor_no_overlap() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters::default();
                        let p1 = PaginationParams {
                            limit: 3,
                            per_page: 3,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let page1 = svc.get_messages(&filters, &p1, true).await.expect("page1");
                        if let Some(ref cursor) = page1.next_cursor {
                            let p2 = PaginationParams {
                                limit: 3,
                                per_page: 3,
                                cursor: Some(cursor.clone()),
                                reverse: false,
                                page: None,
                                sort_by: None,
                            };
                            let page2 = svc.get_messages(&filters, &p2, true).await.expect("page2");
                            let ids1: std::collections::HashSet<_> =
                                page1.data.iter().map(|m| m.message_id.clone()).collect();
                            let ids2: std::collections::HashSet<_> =
                                page2.data.iter().map(|m| m.message_id.clone()).collect();
                            assert!(ids1.intersection(&ids2).next().is_none());
                        }
                        Ok(())
                    })
                },
            )
            .await
            .expect("pagination_with_cursor_no_overlap");
        }

        #[tokio::test]
        async fn empty_database_returns_none() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters {
                            room_id: Some("__nonexistent_room__".into()),
                            ..Default::default()
                        };
                        let pagination = PaginationParams {
                            limit: 10,
                            per_page: 10,
                            cursor: None,
                            reverse: false,
                            page: None,
                            sort_by: None,
                        };
                        let result = svc
                            .get_messages(&filters, &pagination, true)
                            .await
                            .expect("query");
                        assert!(result.data.is_empty());
                        assert!(result.next_cursor.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("empty_database_returns_none");
        }

        #[tokio::test]
        async fn get_by_id_nonexistent() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let result = svc.get_by_id("__nonexistent__", true).await.expect("query");
                        assert!(result.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_by_id_nonexistent");
        }

        #[tokio::test]
        async fn message_exists_false() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let exists = svc.message_exists("__nonexistent__").await.expect("query");
                        assert!(!exists);
                        Ok(())
                    })
                },
            )
            .await
            .expect("message_exists_false");
        }

        #[tokio::test]
        async fn get_before_nonexistent_returns_empty() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let messages = svc.get_before("__nonexistent__", 5).await.expect("query");
                        assert!(messages.is_empty());
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_before_nonexistent_returns_empty");
        }

        #[tokio::test]
        async fn get_after_nonexistent_returns_empty() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let messages = svc.get_after("__nonexistent__", 5).await.expect("query");
                        assert!(messages.is_empty());
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_after_nonexistent_returns_empty");
        }

        #[tokio::test]
        async fn get_context_nonexistent_returns_none() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let result = svc
                            .get_context("__nonexistent__", 2, 2)
                            .await
                            .expect("query");
                        assert!(result.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_context_nonexistent_returns_none");
        }

        #[tokio::test]
        async fn get_latest_message_time_empty_room() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let result = svc
                            .get_latest_message_time("__nonexistent__")
                            .await
                            .expect("query");
                        assert!(result.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_latest_message_time_empty_room");
        }

        #[tokio::test]
        async fn get_earliest_message_time_empty_room() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let result = svc
                            .get_earliest_message_time("__nonexistent__")
                            .await
                            .expect("query");
                        assert!(result.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_earliest_message_time_empty_room");
        }

        #[tokio::test]
        async fn batch_create_empty_list_returns_zero() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let count = svc.batch_create(&[]).await.expect("batch create");
                        assert_eq!(count, 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("batch_create_empty_list_returns_zero");
        }

        #[tokio::test]
        async fn batch_create_if_missing_empty() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let rows = svc.batch_create_if_missing(&[]).await.expect("batch");
                        assert!(rows.is_empty());
                        Ok(())
                    })
                },
            )
            .await
            .expect("batch_create_if_missing_empty");
        }

        #[tokio::test]
        async fn enrich_batch_empty() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let result = svc.enrich_batch(&[]).await.expect("enrich");
                        assert!(result.is_empty());
                        Ok(())
                    })
                },
            )
            .await
            .expect("enrich_batch_empty");
        }

        #[tokio::test]
        async fn mark_deleted_batch_empty_returns_zero() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let count = svc
                            .mark_deleted_batch(&[], None)
                            .await
                            .expect("mark deleted");
                        assert_eq!(count, 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("mark_deleted_batch_empty_returns_zero");
        }

        #[tokio::test]
        async fn mark_recalled_batch_empty_returns_zero() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let count = svc.mark_recalled_batch(&[]).await.expect("mark recalled");
                        assert_eq!(count, 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("mark_recalled_batch_empty_returns_zero");
        }

        #[tokio::test]
        async fn get_deleted_messages() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let messages = svc.get_deleted_messages(None, 10).await.expect("query");
                        for m in &messages {
                            assert!(m.is_deleted || m.is_recalled);
                        }
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_deleted_messages");
        }

        #[tokio::test]
        async fn get_deleted_messages_with_room() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let messages = svc
                            .get_deleted_messages(Some("room1"), 10)
                            .await
                            .expect("query");
                        for m in &messages {
                            assert!(m.is_deleted || m.is_recalled);
                        }
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_deleted_messages_with_room");
        }

        #[tokio::test]
        async fn get_room_stats_returns_stats() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let stats = svc.get_room_stats("room1").await.expect("query");
                        assert!(stats.total >= 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_room_stats_returns_stats");
        }

        #[tokio::test]
        async fn get_user_stats_returns_stats() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let stats = svc.get_user_stats("user1").await.expect("query");
                        assert!(stats.total >= 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_user_stats_returns_stats");
        }

        #[tokio::test]
        async fn count_messages_with_filters() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let filters = MessageFilters {
                            room_id: Some("room1".into()),
                            ..Default::default()
                        };
                        let counts = svc.count_messages(&filters).await.expect("count");
                        assert!(counts.total_messages >= 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("count_messages_with_filters");
        }

        #[tokio::test]
        async fn count_messages_no_filters_uses_rooms_table() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let counts = svc
                            .count_messages(&MessageFilters::default())
                            .await
                            .expect("count");
                        assert!(counts.total_messages >= 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("count_messages_no_filters_uses_rooms_table");
        }

        #[tokio::test]
        async fn create_message_if_missing_duplicate() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::FixtureProfile::Message,
                |session| {
                    Box::pin(async move {
                        let mut svc = MessageService::new(session);
                        let msg = test_message();
                        let first = svc.create_message_if_missing(&msg).await.expect("first");
                        let second = svc.create_message_if_missing(&msg).await.expect("second");
                        assert!(first);
                        assert!(!second);
                        Ok(())
                    })
                },
            )
            .await
            .expect("create_message_if_missing_duplicate");
        }
    }
}
