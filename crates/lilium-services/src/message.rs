use std::collections::HashMap;

use chrono::{DateTime, Utc};
use lilium_common::LiliumError;
use lilium_models::dzmm::message::Message;
use lilium_models::dzmm::{image_gps, message as messages, room as rooms, user as users};
use sea_orm::sea_query::{Expr, JoinType, NullOrdering, OnConflict, Order};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder,
    QuerySelect, Select, Set,
};
use tracing::instrument;

type MessageRow = messages::Model;

#[derive(Debug, Clone)]
struct QueryParts {
    condition: Condition,
    has_condition: bool,
    join_users: bool,
    join_rooms: bool,
    join_gps: bool,
}

impl Default for QueryParts {
    fn default() -> Self {
        Self {
            condition: Condition::all(),
            has_condition: false,
            join_users: false,
            join_rooms: false,
            join_gps: false,
        }
    }
}

#[derive(Debug, Clone, FromQueryResult)]
struct MessageStatsRow {
    total: i64,
    deleted: i64,
    recalled: i64,
    edited: i64,
}

#[derive(Debug, Clone, FromQueryResult)]
struct CountPairRow {
    key: String,
    count: i64,
}

fn db_error(error: impl std::fmt::Display) -> LiliumError {
    LiliumError::database(error.to_string())
}

fn message_active_model(message: &Message) -> messages::ActiveModel {
    messages::ActiveModel {
        message_id: Set(message.message_id.clone()),
        room_id: Set(message.room_id.clone()),
        sent_at: Set(message.sent_at),
        sent_by: Set(message.sent_by.clone()),
        content_type: Set(message.content_type.clone()),
        content_text: Set(message.content_text.clone()),
        attachment_url: Set(message.attachment_url.clone()),
        attachment_file: Set(message.attachment_file.clone()),
        sticker_id: Set(message.sticker_id.clone()),
        alt_text: Set(message.alt_text.clone()),
        metadata: Set(message.metadata.clone()),
        raw_data: Set(message.raw_data.clone()),
        source: Set(message.source.clone()),
        created_at: Set(message.created_at),
        updated_at: Set(message.updated_at),
        is_deleted: Set(message.is_deleted),
        deleted_at: Set(message.deleted_at),
        deleted_by: Set(message.deleted_by.clone()),
        is_recalled: Set(message.is_recalled),
        is_edited: Set(message.is_edited),
        history: Set(message.history.clone()),
        reference_message_id: Set(message.reference_message_id.clone()),
        reference_data: Set(message.reference_data.clone()),
    }
}

fn enriched_message_query(enriched: bool) -> Select<messages::Entity> {
    let query = messages::Entity::find()
        .select_only()
        .column_as(messages::Column::MessageId, "message_id")
        .column_as(messages::Column::RoomId, "room_id")
        .column_as(messages::Column::SentAt, "sent_at")
        .column_as(messages::Column::SentBy, "sent_by")
        .column_as(messages::Column::ContentType, "content_type")
        .column_as(messages::Column::ContentText, "content_text")
        .column_as(messages::Column::AttachmentUrl, "attachment_url")
        .column_as(messages::Column::AttachmentFile, "attachment_file")
        .column_as(messages::Column::StickerId, "sticker_id")
        .column_as(messages::Column::AltText, "alt_text")
        .column_as(messages::Column::Metadata, "metadata")
        .column_as(messages::Column::RawData, "raw_data")
        .column_as(messages::Column::Source, "source")
        .column_as(messages::Column::CreatedAt, "created_at")
        .column_as(messages::Column::UpdatedAt, "updated_at")
        .column_as(messages::Column::IsDeleted, "is_deleted")
        .column_as(messages::Column::DeletedAt, "deleted_at")
        .column_as(messages::Column::DeletedBy, "deleted_by")
        .column_as(messages::Column::IsRecalled, "is_recalled")
        .column_as(messages::Column::IsEdited, "is_edited")
        .column_as(messages::Column::History, "history")
        .column_as(messages::Column::ReferenceMessageId, "reference_message_id")
        .column_as(messages::Column::ReferenceData, "reference_data")
        .join(
            JoinType::LeftJoin,
            messages::Entity::belongs_to(users::Entity)
                .from(messages::Column::SentBy)
                .to(users::Column::UserId)
                .into(),
        )
        .join(
            JoinType::LeftJoin,
            messages::Entity::belongs_to(rooms::Entity)
                .from(messages::Column::RoomId)
                .to(rooms::Column::RoomId)
                .into(),
        );

    if enriched {
        query
            .column_as(users::Column::FullName, "user_display_name")
            .column_as(users::Column::AvatarUrl, "user_avatar_url")
            .column_as(rooms::Column::Title, "room_title")
    } else {
        query
            .column_as(Expr::cust("NULL::varchar"), "user_display_name")
            .column_as(Expr::cust("NULL::varchar"), "user_avatar_url")
            .column_as(Expr::cust("NULL::varchar"), "room_title")
    }
}

fn before_anchor_condition(anchor: &EnrichedMessage) -> Condition {
    Condition::any()
        .add(messages::Column::SentAt.lt(anchor.sent_at))
        .add(
            Condition::all()
                .add(messages::Column::SentAt.eq(anchor.sent_at))
                .add(messages::Column::MessageId.lt(anchor.message_id.clone())),
        )
}

fn after_anchor_condition(anchor: &EnrichedMessage) -> Condition {
    Condition::any()
        .add(messages::Column::SentAt.gt(anchor.sent_at))
        .add(
            Condition::all()
                .add(messages::Column::SentAt.eq(anchor.sent_at))
                .add(messages::Column::MessageId.gt(anchor.message_id.clone())),
        )
}

fn join_message_users(query: Select<messages::Entity>) -> Select<messages::Entity> {
    query.join(
        JoinType::LeftJoin,
        messages::Entity::belongs_to(users::Entity)
            .from(messages::Column::SentBy)
            .to(users::Column::UserId)
            .into(),
    )
}

fn join_message_rooms(query: Select<messages::Entity>) -> Select<messages::Entity> {
    query.join(
        JoinType::LeftJoin,
        messages::Entity::belongs_to(rooms::Entity)
            .from(messages::Column::RoomId)
            .to(rooms::Column::RoomId)
            .into(),
    )
}

fn join_message_gps(query: Select<messages::Entity>) -> Select<messages::Entity> {
    query.join(
        JoinType::InnerJoin,
        messages::Entity::belongs_to(image_gps::Entity)
            .from(messages::Column::MessageId)
            .to(image_gps::Column::MessageId)
            .into(),
    )
}

fn apply_query_parts(
    mut query: Select<messages::Entity>,
    parts: &QueryParts,
    user_room_already_joined: bool,
) -> Select<messages::Entity> {
    if parts.join_users && !user_room_already_joined {
        query = join_message_users(query);
    }
    if parts.join_rooms && !user_room_already_joined {
        query = join_message_rooms(query);
    }
    if parts.join_gps {
        query = join_message_gps(query);
    }
    if parts.has_condition {
        query = query.filter(parts.condition.clone());
    }
    query
}

fn two_part_cursor_condition(
    sort_column: messages::Column,
    reverse: bool,
    value: DateTime<Utc>,
    message_id: String,
) -> Condition {
    if reverse {
        Condition::any().add(sort_column.gt(value)).add(
            Condition::all()
                .add(sort_column.eq(value))
                .add(messages::Column::MessageId.gt(message_id)),
        )
    } else {
        Condition::any().add(sort_column.lt(value)).add(
            Condition::all()
                .add(sort_column.eq(value))
                .add(messages::Column::MessageId.lt(message_id)),
        )
    }
}

fn three_part_cursor_condition(
    reverse: bool,
    sent_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    message_id: String,
) -> Condition {
    if reverse {
        Condition::any()
            .add(messages::Column::SentAt.gt(sent_at))
            .add(
                Condition::all()
                    .add(messages::Column::SentAt.eq(sent_at))
                    .add(messages::Column::CreatedAt.gt(created_at)),
            )
            .add(
                Condition::all()
                    .add(messages::Column::SentAt.eq(sent_at))
                    .add(messages::Column::CreatedAt.eq(created_at))
                    .add(messages::Column::MessageId.gt(message_id)),
            )
    } else {
        Condition::any()
            .add(messages::Column::SentAt.lt(sent_at))
            .add(
                Condition::all()
                    .add(messages::Column::SentAt.eq(sent_at))
                    .add(messages::Column::CreatedAt.lt(created_at)),
            )
            .add(
                Condition::all()
                    .add(messages::Column::SentAt.eq(sent_at))
                    .add(messages::Column::CreatedAt.eq(created_at))
                    .add(messages::Column::MessageId.lt(message_id)),
            )
    }
}

fn apply_message_order(
    mut query: Select<messages::Entity>,
    use_three_part: bool,
    reverse: bool,
    sort_by_created_at: bool,
) -> Select<messages::Entity> {
    if use_three_part {
        if reverse {
            query = query
                .order_by_asc(messages::Column::SentAt)
                .order_by_asc(messages::Column::CreatedAt)
                .order_by_asc(messages::Column::MessageId);
        } else {
            query = query
                .order_by_desc(messages::Column::SentAt)
                .order_by_desc(messages::Column::CreatedAt)
                .order_by_desc(messages::Column::MessageId);
        }
    } else {
        let sort_column = if sort_by_created_at {
            messages::Column::CreatedAt
        } else {
            messages::Column::SentAt
        };
        if reverse {
            query = query
                .order_by_asc(sort_column)
                .order_by_asc(messages::Column::MessageId);
        } else {
            query = query
                .order_by_desc(sort_column)
                .order_by_desc(messages::Column::MessageId);
        }
    }
    query
}

fn build_query_parts(filters: &MessageFilters, include_visible_only: bool) -> QueryParts {
    let mut p = QueryParts::default();
    macro_rules! add_condition {
        ($condition:expr) => {{
            p.condition = std::mem::replace(&mut p.condition, Condition::all()).add($condition);
            p.has_condition = true;
        }};
    }

    if let Some(ref v) = filters.room_id {
        add_condition!(messages::Column::RoomId.eq(v.clone()));
    }

    if let Some(ref ids) = filters.room_ids
        && !ids.is_empty()
    {
        add_condition!(messages::Column::RoomId.is_in(ids.iter().cloned()));
    }

    if let Some(ref v) = filters.account_id {
        p.join_rooms = true;
        add_condition!(Expr::cust_with_values(
            r#""rooms"."account_ids" @> ARRAY[$1]::varchar[]"#,
            [v.as_str()],
        ));
    } else if filters.user_or_account_id.is_some() {
        let Some(id) = filters.user_or_account_id.as_ref() else {
            unreachable!("checked is_some above")
        };
        p.join_rooms = true;
        add_condition!(
            Condition::any()
                .add(messages::Column::SentBy.eq(id.clone()))
                .add(Expr::cust_with_values(
                    r#""rooms"."account_ids" @> ARRAY[$1]::varchar[]"#,
                    [id.as_str()],
                ))
        );
    }

    if let Some(ref v) = filters.user_id {
        add_condition!(messages::Column::SentBy.eq(v.clone()));
    }

    if let Some(ref types) = filters.content_types
        && !types.is_empty()
    {
        add_condition!(messages::Column::ContentType.is_in(types.iter().cloned()));
    }

    if let Some(ref types) = filters.message_types {
        let mut type_conditions = Condition::any();
        let mut has_type_condition = false;
        if types.iter().any(|t| t == "deleted") {
            type_conditions = type_conditions.add(messages::Column::IsDeleted.eq(true));
            has_type_condition = true;
        }
        if types.iter().any(|t| t == "recalled") {
            type_conditions = type_conditions.add(messages::Column::IsRecalled.eq(true));
            has_type_condition = true;
        }
        if types.iter().any(|t| t == "edited") {
            type_conditions = type_conditions.add(messages::Column::IsEdited.eq(true));
            has_type_condition = true;
        }
        if has_type_condition {
            add_condition!(type_conditions);
        }
    }

    if let Some(ref q) = filters.search_query {
        let is_uuid = q.contains('-') && q.len() >= 36 && q.matches('-').count() >= 4;
        if is_uuid {
            add_condition!(messages::Column::MessageId.eq(q.clone()));
        } else {
            // `content_tsv` is a DB-maintained search vector intentionally
            // omitted from `lilium_models::dzmm::message::Model`.
            add_condition!(Expr::cust_with_values(
                r#""messages"."content_tsv" @@ plainto_tsquery('zhparser', $1)"#,
                [q.as_str()],
            ));
        }
    }

    if let Some(ref name) = filters.sender_name {
        p.join_users = true;
        // `name_tsv` is a DB-maintained search vector intentionally omitted
        // from `lilium_models::dzmm::user::Model`.
        add_condition!(Expr::cust_with_values(
            r#""users"."name_tsv" @@ plainto_tsquery('zhparser', $1)"#,
            [name.as_str()],
        ));
    }

    if let Some(ref v) = filters.start_time {
        add_condition!(messages::Column::SentAt.gte(*v));
    }

    if let Some(ref v) = filters.end_time {
        add_condition!(messages::Column::SentAt.lte(*v));
    }

    if let Some(v) = filters.has_attachment {
        if v {
            add_condition!(messages::Column::AttachmentFile.is_not_null());
        } else {
            add_condition!(messages::Column::AttachmentFile.is_null());
        }
    }

    if let Some(v) = filters.has_reference {
        if v {
            add_condition!(messages::Column::ReferenceMessageId.is_not_null());
        } else {
            add_condition!(messages::Column::ReferenceMessageId.is_null());
        }
    }

    if let Some(ref v) = filters.source {
        add_condition!(messages::Column::Source.eq(v.clone()));
    }

    if let Some(ref v) = filters.created_after {
        add_condition!(messages::Column::CreatedAt.gt(*v));
    }

    if filters.gps_only.unwrap_or(false) {
        p.join_gps = true;
    }

    if include_visible_only {
        add_condition!(messages::Column::IsDeleted.eq(false));
        add_condition!(messages::Column::IsRecalled.eq(false));
    }

    p
}

async fn insert_message_if_missing<C: ConnectionTrait>(
    db: &C,
    message: &Message,
) -> crate::Result<bool> {
    let result = messages::Entity::insert(message_active_model(message))
        .on_conflict(
            OnConflict::columns([messages::Column::MessageId, messages::Column::SentAt])
                .do_nothing()
                .to_owned(),
        )
        .exec_without_returning(db)
        .await
        .map_err(db_error)?;
    Ok(result > 0)
}

async fn set_message_recalled<C: ConnectionTrait>(db: &C, message_id: &str) -> crate::Result<()> {
    let now = Utc::now();
    let active = messages::ActiveModel {
        is_recalled: Set(true),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    messages::Entity::update_many()
        .set(active)
        .filter(messages::Column::MessageId.eq(message_id))
        .exec(db)
        .await
        .map_err(db_error)?;
    Ok(())
}

async fn update_message_content<C: ConnectionTrait>(
    db: &C,
    message_id: &str,
    text: &str,
) -> crate::Result<()> {
    messages::Entity::update_many()
        .col_expr(messages::Column::ContentText, Expr::value(text.to_owned()))
        .col_expr(messages::Column::IsEdited, Expr::value(true))
        .col_expr(messages::Column::UpdatedAt, Expr::cust("NOW()"))
        .col_expr(
            messages::Column::History,
            Expr::cust(
                "COALESCE(history, '[]'::jsonb) || jsonb_build_object('content', content_text, 'edited_at', NOW())",
            ),
        )
        .filter(messages::Column::MessageId.eq(message_id))
        .exec(db)
    .await
    .map_err(db_error)?;
    Ok(())
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

#[derive(Debug, Clone, FromQueryResult, serde::Serialize, serde::Deserialize)]
pub struct EnrichedMessage {
    pub message_id: String,
    pub room_id: String,
    pub sent_at: DateTime<Utc>,
    pub sent_by: String,
    pub content_type: String,
    pub content_text: Option<String>,
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

#[allow(clippy::result_large_err)]
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

#[instrument(skip(db, filters, pagination), fields(enriched))]
pub async fn get_messages<C: ConnectionTrait>(
    db: &C,
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
    let sort_by_created_at = pagination.sort_by.as_deref() == Some("created_at");
    let use_three_part = !sort_by_created_at
        && pagination
            .cursor
            .as_deref()
            .map(|cursor| cursor.split('|').count() == 3)
            .unwrap_or(false);

    let mut query = apply_query_parts(enriched_message_query(enriched), &query_parts, true);

    if let Some(ref cursor) = pagination.cursor {
        let decoded = decode_cursor(cursor)?;

        match (&decoded, use_three_part) {
            (Cursor::ThreePart(..), true) => {
                if let Cursor::ThreePart(sa, ca, mid) = decoded {
                    query =
                        query.filter(three_part_cursor_condition(pagination.reverse, sa, ca, mid));
                }
            }
            _ => {
                let sort_column = if sort_by_created_at {
                    messages::Column::CreatedAt
                } else {
                    messages::Column::SentAt
                };
                match decoded {
                    Cursor::TwoPart(sa, mid) => {
                        query = query.filter(two_part_cursor_condition(
                            sort_column,
                            pagination.reverse,
                            sa,
                            mid,
                        ));
                    }
                    Cursor::ThreePart(sa, _, mid) => {
                        query = query.filter(two_part_cursor_condition(
                            sort_column,
                            pagination.reverse,
                            sa,
                            mid,
                        ));
                    }
                }
            }
        }
    }

    if pagination.page.unwrap_or(1) > 1 && pagination.cursor.is_none() {
        let offset = (pagination.page.unwrap() - 1) * pagination.per_page;
        if offset > 0 {
            query = query.offset(offset as u64);
        }
    }

    let fetch_limit = (per_page + 1).max(0) as u64;
    let mut messages = apply_message_order(
        query,
        use_three_part,
        pagination.reverse,
        sort_by_created_at,
    )
    .limit(fetch_limit)
    .into_model::<EnrichedMessage>()
    .all(db)
    .await
    .map_err(db_error)?;

    let has_more = per_page >= 0 && messages.len() as i64 > per_page;
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

#[instrument(skip(db), fields(message_id = %message_id, enriched))]
pub async fn get_by_id<C: ConnectionTrait>(
    db: &C,
    message_id: &str,
    enriched: bool,
) -> crate::Result<Option<EnrichedMessage>> {
    enriched_message_query(enriched)
        .filter(messages::Column::MessageId.eq(message_id))
        .order_by_desc(messages::Column::SentAt)
        .limit(1)
        .into_model::<EnrichedMessage>()
        .one(db)
        .await
        .map_err(db_error)
}

#[instrument(skip(db), fields(message_id = %message_id, sent_at = %sent_at, enriched))]
pub async fn get_by_id_at<C: ConnectionTrait>(
    db: &C,
    message_id: &str,
    sent_at: DateTime<Utc>,
    enriched: bool,
) -> crate::Result<Option<EnrichedMessage>> {
    enriched_message_query(enriched)
        .filter(messages::Column::MessageId.eq(message_id))
        .filter(messages::Column::SentAt.eq(sent_at))
        .into_model::<EnrichedMessage>()
        .one(db)
        .await
        .map_err(db_error)
}

#[instrument(skip(db), fields(message_id = %message_id, before_count, after_count))]
pub async fn get_context<C: ConnectionTrait>(
    db: &C,
    message_id: &str,
    before_count: i64,
    after_count: i64,
) -> crate::Result<Option<MessageContextResult>> {
    let anchor = match get_by_id(db, message_id, false).await? {
        Some(m) => m,
        None => return Ok(None),
    };

    let before_limit = before_count.min(50);
    let after_limit = after_count.min(50);

    let before = enriched_message_query(true)
        .filter(messages::Column::RoomId.eq(anchor.room_id.clone()))
        .filter(before_anchor_condition(&anchor))
        .order_by_desc(messages::Column::SentAt)
        .order_by_desc(messages::Column::MessageId)
        .limit(before_limit as u64)
        .into_model::<EnrichedMessage>()
        .all(db)
        .await
        .map_err(db_error)?;

    let after = enriched_message_query(true)
        .filter(messages::Column::RoomId.eq(anchor.room_id.clone()))
        .filter(after_anchor_condition(&anchor))
        .order_by_asc(messages::Column::SentAt)
        .order_by_asc(messages::Column::MessageId)
        .limit(after_limit as u64)
        .into_model::<EnrichedMessage>()
        .all(db)
        .await
        .map_err(db_error)?;

    let anchor_enriched = get_by_id_at(db, &anchor.message_id, anchor.sent_at, true)
        .await?
        .ok_or_else(|| LiliumError::database("anchor message disappeared during context fetch"))?;
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

#[instrument(skip(db), fields(message_id = %message_id, count))]
pub async fn get_before<C: ConnectionTrait>(
    db: &C,
    message_id: &str,
    count: i64,
) -> crate::Result<Vec<EnrichedMessage>> {
    let target = get_by_id(db, message_id, false).await?;
    let target = match target {
        Some(m) => m,
        None => return Ok(Vec::new()),
    };

    let limit = count.min(50);

    let mut messages = enriched_message_query(true)
        .filter(messages::Column::RoomId.eq(target.room_id.clone()))
        .filter(before_anchor_condition(&target))
        .order_by_desc(messages::Column::SentAt)
        .order_by_desc(messages::Column::MessageId)
        .limit(limit as u64)
        .into_model::<EnrichedMessage>()
        .all(db)
        .await
        .map_err(db_error)?;

    messages.reverse();
    Ok(messages)
}

#[instrument(skip(db), fields(message_id = %message_id, count))]
pub async fn get_after<C: ConnectionTrait>(
    db: &C,
    message_id: &str,
    count: i64,
) -> crate::Result<Vec<EnrichedMessage>> {
    let target = get_by_id(db, message_id, false).await?;
    let target = match target {
        Some(m) => m,
        None => return Ok(Vec::new()),
    };

    let limit = count.min(50);

    enriched_message_query(true)
        .filter(messages::Column::RoomId.eq(target.room_id.clone()))
        .filter(after_anchor_condition(&target))
        .order_by_asc(messages::Column::SentAt)
        .order_by_asc(messages::Column::MessageId)
        .limit(limit as u64)
        .into_model::<EnrichedMessage>()
        .all(db)
        .await
        .map_err(db_error)
}

#[instrument(skip(db, messages), fields(message_count = messages.len()))]
pub async fn enrich_batch<C: ConnectionTrait>(
    db: &C,
    messages: &[Message],
) -> crate::Result<Vec<EnrichedMessage>> {
    if messages.is_empty() {
        return Ok(Vec::new());
    }

    let mut identity_condition = Condition::any();
    for msg in messages {
        identity_condition = identity_condition.add(
            Condition::all()
                .add(messages::Column::MessageId.eq(msg.message_id.clone()))
                .add(messages::Column::SentAt.eq(msg.sent_at)),
        );
    }

    let enriched_by_id: HashMap<(String, DateTime<Utc>), EnrichedMessage> =
        enriched_message_query(true)
            .filter(identity_condition)
            .into_model::<EnrichedMessage>()
            .all(db)
            .await
            .map_err(db_error)?
            .into_iter()
            .map(|msg| ((msg.message_id.clone(), msg.sent_at), msg))
            .collect();

    let enriched = messages
        .iter()
        .filter_map(|msg| {
            enriched_by_id
                .get(&(msg.message_id.clone(), msg.sent_at))
                .cloned()
        })
        .collect();

    Ok(enriched)
}

#[instrument(skip(db), fields(room_id = ?room_id, limit))]
pub async fn get_deleted_messages<C: ConnectionTrait>(
    db: &C,
    room_id: Option<&str>,
    limit: i64,
) -> crate::Result<Vec<MessageRow>> {
    let mut query = messages::Entity::find().filter(
        Condition::any()
            .add(messages::Column::IsDeleted.eq(true))
            .add(messages::Column::IsRecalled.eq(true)),
    );

    if let Some(rid) = room_id {
        query = query.filter(messages::Column::RoomId.eq(rid));
    }

    let messages = query
        .order_by_with_nulls(messages::Column::DeletedAt, Order::Desc, NullOrdering::Last)
        .limit(limit as u64)
        .all(db)
        .await
        .map_err(db_error)?;
    Ok(messages.into_iter().collect())
}

#[instrument(skip(db), fields(room_id = %room_id))]
pub async fn get_room_stats<C: ConnectionTrait>(
    db: &C,
    room_id: &str,
) -> crate::Result<MessageStats> {
    let row = messages::Entity::find()
        .select_only()
        .column_as(messages::Column::MessageId.count(), "total")
        .column_as(
            Expr::cust(
                r#"COALESCE(SUM(CASE WHEN "messages"."is_deleted" THEN 1 ELSE 0 END), 0)::bigint"#,
            ),
            "deleted",
        )
        .column_as(
            Expr::cust(
                r#"COALESCE(SUM(CASE WHEN "messages"."is_recalled" THEN 1 ELSE 0 END), 0)::bigint"#,
            ),
            "recalled",
        )
        .column_as(
            Expr::cust(
                r#"COALESCE(SUM(CASE WHEN "messages"."is_edited" THEN 1 ELSE 0 END), 0)::bigint"#,
            ),
            "edited",
        )
        .filter(messages::Column::RoomId.eq(room_id))
        .into_model::<MessageStatsRow>()
        .one(db)
        .await
        .map_err(db_error)?
        .unwrap_or(MessageStatsRow {
            total: 0,
            deleted: 0,
            recalled: 0,
            edited: 0,
        });

    let type_rows = messages::Entity::find()
        .select_only()
        .column_as(messages::Column::ContentType, "key")
        .column_as(messages::Column::MessageId.count(), "count")
        .filter(messages::Column::RoomId.eq(room_id))
        .group_by(messages::Column::ContentType)
        .into_model::<CountPairRow>()
        .all(db)
        .await
        .map_err(db_error)?;

    Ok(MessageStats {
        total: row.total,
        deleted: row.deleted,
        recalled: row.recalled,
        edited: row.edited,
        by_content_type: type_rows
            .into_iter()
            .map(|row| (row.key, row.count))
            .collect(),
        by_room: None,
    })
}

#[instrument(skip(db), fields(user_id = %user_id))]
pub async fn get_user_stats<C: ConnectionTrait>(
    db: &C,
    user_id: &str,
) -> crate::Result<MessageStats> {
    let row = messages::Entity::find()
        .select_only()
        .column_as(messages::Column::MessageId.count(), "total")
        .column_as(
            Expr::cust(
                r#"COALESCE(SUM(CASE WHEN "messages"."is_deleted" THEN 1 ELSE 0 END), 0)::bigint"#,
            ),
            "deleted",
        )
        .column_as(
            Expr::cust(
                r#"COALESCE(SUM(CASE WHEN "messages"."is_recalled" THEN 1 ELSE 0 END), 0)::bigint"#,
            ),
            "recalled",
        )
        .column_as(
            Expr::cust(
                r#"COALESCE(SUM(CASE WHEN "messages"."is_edited" THEN 1 ELSE 0 END), 0)::bigint"#,
            ),
            "edited",
        )
        .filter(messages::Column::SentBy.eq(user_id))
        .into_model::<MessageStatsRow>()
        .one(db)
        .await
        .map_err(db_error)?
        .unwrap_or(MessageStatsRow {
            total: 0,
            deleted: 0,
            recalled: 0,
            edited: 0,
        });

    let type_rows = messages::Entity::find()
        .select_only()
        .column_as(messages::Column::ContentType, "key")
        .column_as(messages::Column::MessageId.count(), "count")
        .filter(messages::Column::SentBy.eq(user_id))
        .group_by(messages::Column::ContentType)
        .into_model::<CountPairRow>()
        .all(db)
        .await
        .map_err(db_error)?;

    let room_rows = messages::Entity::find()
        .select_only()
        .column_as(messages::Column::RoomId, "key")
        .column_as(messages::Column::MessageId.count(), "count")
        .filter(messages::Column::SentBy.eq(user_id))
        .group_by(messages::Column::RoomId)
        .order_by_desc(Expr::cust("COUNT(*)"))
        .limit(10)
        .into_model::<CountPairRow>()
        .all(db)
        .await
        .map_err(db_error)?;

    Ok(MessageStats {
        total: row.total,
        deleted: row.deleted,
        recalled: row.recalled,
        edited: row.edited,
        by_content_type: type_rows
            .into_iter()
            .map(|row| (row.key, row.count))
            .collect(),
        by_room: Some(
            room_rows
                .into_iter()
                .map(|row| (row.key, row.count))
                .collect(),
        ),
    })
}

#[instrument(skip(db), fields(message_id = %message_id))]
pub async fn message_exists<C: ConnectionTrait>(db: &C, message_id: &str) -> crate::Result<bool> {
    let exists = messages::Entity::find()
        .select_only()
        .column(messages::Column::MessageId)
        .filter(messages::Column::MessageId.eq(message_id))
        .into_tuple::<(String,)>()
        .one(db)
        .await
        .map_err(db_error)?
        .is_some();
    Ok(exists)
}

#[instrument(skip(db, message), fields(message_id = %message.message_id))]
pub async fn create_message<C: ConnectionTrait>(db: &C, message: &Message) -> crate::Result<()> {
    messages::Entity::insert(message_active_model(message))
        .exec(db)
        .await
        .map_err(db_error)?;
    Ok(())
}

#[instrument(skip(db, message), fields(message_id = %message.message_id))]
pub async fn create_message_if_missing<C: ConnectionTrait>(
    db: &C,
    message: &Message,
) -> crate::Result<bool> {
    insert_message_if_missing(db, message).await
}

#[instrument(skip(db, items), fields(message_count = items.len()))]
pub async fn batch_create_if_missing<C: ConnectionTrait>(
    db: &C,
    items: &[Message],
) -> crate::Result<Vec<(String, DateTime<Utc>)>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let rows = messages::Entity::insert_many(items.iter().map(message_active_model))
        .on_conflict(
            OnConflict::columns([messages::Column::MessageId, messages::Column::SentAt])
                .do_nothing()
                .to_owned(),
        )
        .exec_with_returning_many(db)
        .await
        .map_err(db_error)?;
    Ok(rows
        .into_iter()
        .map(|row| (row.message_id, row.sent_at))
        .collect())
}

#[instrument(skip(db, message), fields(message_id = %message.message_id))]
pub async fn update_message<C: ConnectionTrait>(db: &C, message: &Message) -> crate::Result<()> {
    messages::Entity::update_many()
        .set(messages::ActiveModel {
            content_type: Set(message.content_type.clone()),
            content_text: Set(message.content_text.clone()),
            attachment_url: Set(message.attachment_url.clone()),
            attachment_file: Set(message.attachment_file.clone()),
            sticker_id: Set(message.sticker_id.clone()),
            alt_text: Set(message.alt_text.clone()),
            metadata: Set(message.metadata.clone()),
            updated_at: Set(Some(Utc::now())),
            is_edited: Set(message.is_edited),
            history: Set(message.history.clone()),
            ..Default::default()
        })
        .filter(messages::Column::MessageId.eq(message.message_id.clone()))
        .filter(messages::Column::SentAt.eq(message.sent_at))
        .exec(db)
        .await
        .map_err(db_error)?;
    Ok(())
}

#[instrument(skip(db, payload), fields(message_id = %message_id))]
pub async fn update_message_from_payload<C: ConnectionTrait>(
    db: &C,
    message_id: &str,
    payload: &serde_json::Value,
) -> crate::Result<()> {
    if let Some(content) = payload.get("message").and_then(|m| m.get("content")) {
        if content.get("type").and_then(|v| v.as_str()) == Some("recalled") {
            set_message_recalled(db, message_id).await
        } else {
            let sent_at = payload
                .get("message")
                .and_then(|m| m.get("sent_at").or_else(|| m.get("sentAt")))
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            if let Some(sent_at) = sent_at {
                let existing = get_by_id_at(db, message_id, sent_at, false).await?;
                if existing.is_none() {
                    return Ok(());
                }
                if let Some(text) = content.get("text").and_then(|v| v.as_str()) {
                    update_message_content(db, message_id, text).await
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

#[instrument(skip(db), fields(message_id = %message_id, has_deleted_by = deleted_by.is_some()))]
pub async fn mark_deleted<C: ConnectionTrait>(
    db: &C,
    message_id: &str,
    deleted_by: Option<&str>,
) -> crate::Result<()> {
    let now = Utc::now();
    let active = messages::ActiveModel {
        is_deleted: Set(true),
        deleted_at: Set(Some(now)),
        deleted_by: Set(deleted_by.map(|v| v.to_owned())),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    messages::Entity::update_many()
        .set(active)
        .filter(messages::Column::MessageId.eq(message_id))
        .exec(db)
        .await
        .map_err(db_error)?;
    Ok(())
}

#[instrument(skip(db, message_ids), fields(message_count = message_ids.len(), has_deleted_by = deleted_by.is_some()))]
pub async fn mark_deleted_batch<C: ConnectionTrait>(
    db: &C,
    message_ids: &[String],
    deleted_by: Option<&str>,
) -> crate::Result<i64> {
    if message_ids.is_empty() {
        return Ok(0);
    }
    let now = Utc::now();
    let active = messages::ActiveModel {
        is_deleted: Set(true),
        deleted_at: Set(Some(now)),
        deleted_by: Set(deleted_by.map(|v| v.to_owned())),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    let result = messages::Entity::update_many()
        .set(active)
        .filter(messages::Column::MessageId.is_in(message_ids.iter().cloned()))
        .exec(db)
        .await
        .map_err(db_error)?;
    Ok(result.rows_affected as i64)
}

#[instrument(skip(db), fields(message_id = %message_id))]
pub async fn mark_recalled<C: ConnectionTrait>(db: &C, message_id: &str) -> crate::Result<()> {
    let now = Utc::now();
    let active = messages::ActiveModel {
        is_recalled: Set(true),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    messages::Entity::update_many()
        .set(active)
        .filter(messages::Column::MessageId.eq(message_id))
        .exec(db)
        .await
        .map_err(db_error)?;
    Ok(())
}

#[instrument(skip(db, message_ids), fields(message_count = message_ids.len()))]
pub async fn mark_recalled_batch<C: ConnectionTrait>(
    db: &C,
    message_ids: &[String],
) -> crate::Result<i64> {
    if message_ids.is_empty() {
        return Ok(0);
    }
    let now = Utc::now();
    let active = messages::ActiveModel {
        is_recalled: Set(true),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    let result = messages::Entity::update_many()
        .set(active)
        .filter(messages::Column::MessageId.is_in(message_ids.iter().cloned()))
        .exec(db)
        .await
        .map_err(db_error)?;
    Ok(result.rows_affected as i64)
}

#[instrument(skip(db, items), fields(message_count = items.len()))]
pub async fn batch_create<C: ConnectionTrait>(db: &C, items: &[Message]) -> crate::Result<i64> {
    if items.is_empty() {
        return Ok(0);
    }

    let rows_affected = messages::Entity::insert_many(items.iter().map(message_active_model))
        .exec_without_returning(db)
        .await
        .map_err(db_error)?;
    Ok(rows_affected as i64)
}

#[instrument(skip(db), fields(room_id = %room_id))]
pub async fn get_latest_message_time<C: ConnectionTrait>(
    db: &C,
    room_id: &str,
) -> crate::Result<Option<DateTime<Utc>>> {
    messages::Entity::find()
        .select_only()
        .column(messages::Column::SentAt)
        .filter(messages::Column::RoomId.eq(room_id))
        .order_by_desc(messages::Column::SentAt)
        .into_tuple()
        .one(db)
        .await
        .map_err(db_error)
}

#[instrument(skip(db), fields(room_id = %room_id))]
pub async fn get_earliest_message_time<C: ConnectionTrait>(
    db: &C,
    room_id: &str,
) -> crate::Result<Option<DateTime<Utc>>> {
    messages::Entity::find()
        .select_only()
        .column(messages::Column::SentAt)
        .filter(messages::Column::RoomId.eq(room_id))
        .order_by_asc(messages::Column::SentAt)
        .into_tuple()
        .one(db)
        .await
        .map_err(db_error)
}

#[instrument(skip(db, filters), fields(has_gps_only = filters.gps_only.unwrap_or(false)))]
pub async fn count_messages<C: ConnectionTrait>(
    db: &C,
    filters: &MessageFilters,
) -> crate::Result<MessageCounts> {
    let query_parts = build_query_parts(filters, false);

    if !query_parts.has_condition && !query_parts.join_gps {
        let row = rooms::Entity::find()
            .select_only()
            .column_as(
                Expr::cust(r#"COALESCE(SUM("rooms"."message_count"), 0)::bigint"#),
                "total",
            )
            .column_as(
                Expr::cust(r#"COALESCE(SUM("rooms"."deleted_count"), 0)::bigint"#),
                "deleted",
            )
            .column_as(
                Expr::cust(r#"COALESCE(SUM("rooms"."recalled_count"), 0)::bigint"#),
                "recalled",
            )
            .column_as(
                Expr::cust(r#"COALESCE(SUM("rooms"."edited_count"), 0)::bigint"#),
                "edited",
            )
            .into_model::<MessageStatsRow>()
            .one(db)
            .await
            .map_err(db_error)?
            .unwrap_or(MessageStatsRow {
                total: 0,
                deleted: 0,
                recalled: 0,
                edited: 0,
            });

        return Ok(MessageCounts {
            total_messages: row.total,
            deleted_messages: row.deleted,
            recalled_messages: row.recalled,
            edited_messages: row.edited,
        });
    }

    let row = apply_query_parts(
        messages::Entity::find()
            .select_only()
            .column_as(Expr::cust("COUNT(*)::bigint"), "total")
            .column_as(
                Expr::cust(
                    r#"COALESCE(SUM(CASE WHEN "messages"."is_deleted" THEN 1 ELSE 0 END), 0)::bigint"#,
                ),
                "deleted",
            )
            .column_as(
                Expr::cust(
                    r#"COALESCE(SUM(CASE WHEN "messages"."is_recalled" THEN 1 ELSE 0 END), 0)::bigint"#,
                ),
                "recalled",
            )
            .column_as(
                Expr::cust(
                    r#"COALESCE(SUM(CASE WHEN "messages"."is_edited" THEN 1 ELSE 0 END), 0)::bigint"#,
                ),
                "edited",
            ),
        &query_parts,
        false,
    )
    .into_model::<MessageStatsRow>()
    .one(db)
    .await
    .map_err(db_error)?
    .unwrap_or(MessageStatsRow {
        total: 0,
        deleted: 0,
        recalled: 0,
        edited: 0,
    });

    Ok(MessageCounts {
        total_messages: row.total,
        deleted_messages: row.deleted,
        recalled_messages: row.recalled,
        edited_messages: row.edited,
    })
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
        fn build_query_default_no_filters() {
            let qp = build_query_parts(&MessageFilters::default(), false);
            assert!(!qp.has_condition);
            assert!(!qp.join_users);
            assert!(!qp.join_rooms);
            assert!(!qp.join_gps);
        }

        #[test]
        fn build_query_simple_filters_add_conditions_without_joins() {
            let t = Utc::now();
            let f = MessageFilters {
                room_id: Some("room1".into()),
                user_id: Some("user1".into()),
                content_types: Some(vec!["text".into()]),
                start_time: Some(t),
                end_time: Some(t),
                has_attachment: Some(true),
                has_reference: Some(true),
                source: Some("spider".into()),
                created_after: Some(t),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.has_condition);
            assert!(!qp.join_users);
            assert!(!qp.join_rooms);
            assert!(!qp.join_gps);
        }

        #[test]
        fn build_query_join_filters_set_join_flags() {
            let f = MessageFilters {
                account_id: Some("acct1".into()),
                sender_name: Some("Test".into()),
                gps_only: Some(true),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(qp.has_condition);
            assert!(qp.join_users);
            assert!(qp.join_rooms);
            assert!(qp.join_gps);
        }

        #[test]
        fn build_query_visible_only_adds_condition() {
            let qp = build_query_parts(&MessageFilters::default(), true);
            assert!(qp.has_condition);
            assert!(!qp.join_users);
            assert!(!qp.join_rooms);
            assert!(!qp.join_gps);
        }

        #[test]
        fn build_query_empty_vector_filters_are_noops() {
            let f = MessageFilters {
                room_ids: Some(vec![]),
                content_types: Some(vec![]),
                gps_only: Some(false),
                ..Default::default()
            };
            let qp = build_query_parts(&f, false);
            assert!(!qp.has_condition);
            assert!(!qp.join_gps);
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

    mod query_parts {
        use super::QueryParts;

        #[test]
        fn default_empty() {
            let qp = QueryParts::default();
            assert!(!qp.has_condition);
            assert!(!qp.join_users);
            assert!(!qp.join_rooms);
            assert!(!qp.join_gps);
        }
    }

    mod message_integration {
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

        macro_rules! message_db_test {
            ($name:ident, $session:ident, $body:block) => {
                #[tokio::test]
                async fn $name() {
                    let test_db = lilium_test_fixtures::TestDb::acquire(
                        lilium_test_fixtures::FixtureProfile::Message,
                    )
                    .await
                    .expect("init message db");

                    lilium_database::transaction!(test_db.database(), |$session| $body)
                        .await
                        .expect(stringify!($name));
                }
            };
        }

        message_db_test!(message_exists_can_be_called_directly, session, {
            let exists = message_exists(session, "__nonexistent__")
                .await
                .expect("query");
            assert!(!exists);
            Ok(())
        });

        message_db_test!(get_messages_no_filters, session, {
            let filters = MessageFilters::default();
            let pagination = PaginationParams {
                limit: 100,
                per_page: 100,
                cursor: None,
                reverse: false,
                page: None,
                sort_by: None,
            };
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            assert!(result.data.len() <= 100);
            Ok(())
        });

        message_db_test!(get_messages_ordered_newest_first, session, {
            let filters = MessageFilters::default();
            let pagination = PaginationParams {
                limit: 100,
                per_page: 100,
                cursor: None,
                reverse: false,
                page: None,
                sort_by: None,
            };
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            for i in 0..result.data.len().saturating_sub(1) {
                assert!(result.data[i].sent_at >= result.data[i + 1].sent_at);
            }
            Ok(())
        });

        message_db_test!(get_messages_reverse_order, session, {
            let filters = MessageFilters::default();
            let pagination = PaginationParams {
                limit: 100,
                per_page: 100,
                cursor: None,
                reverse: true,
                page: None,
                sort_by: None,
            };
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            for i in 0..result.data.len().saturating_sub(1) {
                assert!(result.data[i].sent_at <= result.data[i + 1].sent_at);
            }
            Ok(())
        });

        message_db_test!(filter_by_room, session, {
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
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            assert!(result.data.iter().all(|m| m.room_id == "room1"));
            Ok(())
        });

        message_db_test!(filter_by_user, session, {
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
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            assert!(result.data.iter().all(|m| m.sent_by == "user1"));
            Ok(())
        });

        message_db_test!(filter_by_content_types, session, {
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
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            assert!(result.data.iter().all(|m| m.content_type == "text"));
            Ok(())
        });

        message_db_test!(filter_deleted_messages, session, {
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
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            assert!(result.data.iter().all(|m| m.is_deleted));
            Ok(())
        });

        message_db_test!(filter_recalled_messages, session, {
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
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            assert!(result.data.iter().all(|m| m.is_recalled));
            Ok(())
        });

        message_db_test!(filter_by_time_range, session, {
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
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            for m in &result.data {
                assert!(m.sent_at >= start);
                assert!(m.sent_at <= end);
            }
            Ok(())
        });

        message_db_test!(filter_has_attachment, session, {
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
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            assert!(result.data.iter().all(|m| m.attachment_file.is_some()));
            Ok(())
        });

        message_db_test!(filter_has_reference, session, {
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
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            assert!(result.data.iter().all(|m| m.reference_message_id.is_some()));
            Ok(())
        });

        message_db_test!(filter_combined, session, {
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
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            for m in &result.data {
                assert_eq!(m.room_id, "room1");
                assert_eq!(m.sent_by, "user1");
                assert_eq!(m.content_type, "text");
            }
            Ok(())
        });

        message_db_test!(pagination_first_page, session, {
            let filters = MessageFilters::default();
            let pagination = PaginationParams {
                limit: 3,
                per_page: 3,
                cursor: None,
                reverse: false,
                page: None,
                sort_by: None,
            };
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            assert!(result.data.len() <= 3);
            Ok(())
        });

        message_db_test!(pagination_with_cursor_no_overlap, session, {
            let filters = MessageFilters::default();
            let p1 = PaginationParams {
                limit: 3,
                per_page: 3,
                cursor: None,
                reverse: false,
                page: None,
                sort_by: None,
            };
            let page1 = get_messages(session, &filters, &p1, true)
                .await
                .expect("page1");
            if let Some(ref cursor) = page1.next_cursor {
                let p2 = PaginationParams {
                    limit: 3,
                    per_page: 3,
                    cursor: Some(cursor.clone()),
                    reverse: false,
                    page: None,
                    sort_by: None,
                };
                let page2 = get_messages(session, &filters, &p2, true)
                    .await
                    .expect("page2");
                let ids1: std::collections::HashSet<_> =
                    page1.data.iter().map(|m| m.message_id.clone()).collect();
                let ids2: std::collections::HashSet<_> =
                    page2.data.iter().map(|m| m.message_id.clone()).collect();
                assert!(ids1.intersection(&ids2).next().is_none());
            }
            Ok(())
        });

        message_db_test!(empty_database_returns_none, session, {
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
            let result = get_messages(session, &filters, &pagination, true)
                .await
                .expect("query");
            assert!(result.data.is_empty());
            assert!(result.next_cursor.is_none());
            Ok(())
        });

        message_db_test!(get_by_id_nonexistent, session, {
            let result = get_by_id(session, "__nonexistent__", true)
                .await
                .expect("query");
            assert!(result.is_none());
            Ok(())
        });

        message_db_test!(message_exists_false, session, {
            let exists = message_exists(session, "__nonexistent__")
                .await
                .expect("query");
            assert!(!exists);
            Ok(())
        });

        message_db_test!(get_before_nonexistent_returns_empty, session, {
            let messages = get_before(session, "__nonexistent__", 5)
                .await
                .expect("query");
            assert!(messages.is_empty());
            Ok(())
        });

        message_db_test!(get_after_nonexistent_returns_empty, session, {
            let messages = get_after(session, "__nonexistent__", 5)
                .await
                .expect("query");
            assert!(messages.is_empty());
            Ok(())
        });

        message_db_test!(get_context_nonexistent_returns_none, session, {
            let result = get_context(session, "__nonexistent__", 2, 2)
                .await
                .expect("query");
            assert!(result.is_none());
            Ok(())
        });

        message_db_test!(get_latest_message_time_empty_room, session, {
            let result = get_latest_message_time(session, "__nonexistent__")
                .await
                .expect("query");
            assert!(result.is_none());
            Ok(())
        });

        message_db_test!(get_earliest_message_time_empty_room, session, {
            let result = get_earliest_message_time(session, "__nonexistent__")
                .await
                .expect("query");
            assert!(result.is_none());
            Ok(())
        });

        message_db_test!(batch_create_empty_list_returns_zero, session, {
            let count = batch_create(session, &[]).await.expect("batch create");
            assert_eq!(count, 0);
            Ok(())
        });

        message_db_test!(batch_create_if_missing_empty, session, {
            let rows = batch_create_if_missing(session, &[]).await.expect("batch");
            assert!(rows.is_empty());
            Ok(())
        });

        message_db_test!(enrich_batch_empty, session, {
            let result = enrich_batch(session, &[]).await.expect("enrich");
            assert!(result.is_empty());
            Ok(())
        });

        message_db_test!(mark_deleted_batch_empty_returns_zero, session, {
            let count = mark_deleted_batch(session, &[], None)
                .await
                .expect("mark deleted");
            assert_eq!(count, 0);
            Ok(())
        });

        message_db_test!(mark_recalled_batch_empty_returns_zero, session, {
            let count = mark_recalled_batch(session, &[])
                .await
                .expect("mark recalled");
            assert_eq!(count, 0);
            Ok(())
        });

        message_db_test!(get_deleted_messages, session, {
            let messages = super::get_deleted_messages(session, None, 10)
                .await
                .expect("query");
            for m in &messages {
                assert!(m.is_deleted || m.is_recalled);
            }
            Ok(())
        });

        message_db_test!(get_deleted_messages_with_room, session, {
            let messages = super::get_deleted_messages(session, Some("room1"), 10)
                .await
                .expect("query");
            for m in &messages {
                assert!(m.is_deleted || m.is_recalled);
            }
            Ok(())
        });

        message_db_test!(get_room_stats_returns_stats, session, {
            let stats = get_room_stats(session, "room1").await.expect("query");
            assert!(stats.total >= 0);
            Ok(())
        });

        message_db_test!(get_user_stats_returns_stats, session, {
            let stats = get_user_stats(session, "user1").await.expect("query");
            assert!(stats.total >= 0);
            Ok(())
        });

        message_db_test!(count_messages_with_filters, session, {
            let filters = MessageFilters {
                room_id: Some("room1".into()),
                ..Default::default()
            };
            let counts = count_messages(session, &filters).await.expect("count");
            assert!(counts.total_messages >= 0);
            Ok(())
        });

        message_db_test!(count_messages_no_filters_uses_rooms_table, session, {
            let counts = count_messages(session, &MessageFilters::default())
                .await
                .expect("count");
            assert!(counts.total_messages >= 0);
            Ok(())
        });

        message_db_test!(create_message_if_missing_duplicate, session, {
            let msg = test_message();
            let first = create_message_if_missing(session, &msg)
                .await
                .expect("first");
            let second = create_message_if_missing(session, &msg)
                .await
                .expect("second");
            assert!(first);
            assert!(!second);
            Ok(())
        });

        message_db_test!(
            create_message_if_missing_persists_attachment_and_reference_fields,
            session,
            {
                let mut msg = test_message();
                msg.message_id = "attachment_msg".into();
                msg.content_type = "image".into();
                msg.attachment_url = Some("https://example.com/image.png".into());
                msg.sticker_id = Some("sticker_1".into());
                msg.alt_text = Some("image alt".into());
                msg.metadata = Some(serde_json::json!({"video": {"width": 640}}));
                msg.reference_message_id = Some("referenced_msg".into());
                msg.reference_data =
                    Some(serde_json::json!({"id": "referenced_msg", "text": "quoted"}));
                msg.history = Some(serde_json::json!([{"content": "old"}]));

                let created = create_message_if_missing(session, &msg)
                    .await
                    .expect("create");
                assert!(created);

                let saved = get_by_id(session, "attachment_msg", false)
                    .await
                    .expect("load")
                    .expect("message exists");
                assert_eq!(
                    saved.attachment_url.as_deref(),
                    Some("https://example.com/image.png")
                );
                assert_eq!(saved.sticker_id.as_deref(), Some("sticker_1"));
                assert_eq!(saved.alt_text.as_deref(), Some("image alt"));
                assert_eq!(saved.metadata, msg.metadata);
                assert_eq!(
                    saved.reference_message_id.as_deref(),
                    Some("referenced_msg")
                );
                assert_eq!(saved.reference_data, msg.reference_data);
                assert_eq!(saved.history, msg.history);
                Ok(())
            }
        );
    }
}
