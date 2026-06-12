use sqlx::PgPool;
use lilium_models::dzmm::message::Message;
use anyhow::Result;

pub async fn create_message_if_missing(pool: &PgPool, message: &Message) -> Result<bool> {
    let result = sqlx::query(
        r#"INSERT INTO messages (message_id, room_id, sent_by, content_text, content_type,
               sent_at, is_deleted, is_recalled, is_edited, history, raw_data, source)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               ON CONFLICT DO NOTHING"#,
    )
    .bind(&message.message_id)
    .bind(&message.room_id)
    .bind(&message.sent_by)
    .bind(&message.content_text)
    .bind(&message.content_type)
    .bind(message.sent_at)
    .bind(message.is_deleted)
    .bind(message.is_recalled)
    .bind(message.is_edited)
    .bind(&message.history)
    .bind(&message.raw_data)
    .bind(&message.source)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn mark_deleted(pool: &PgPool, message_id: &str) -> Result<()> {
    sqlx::query("UPDATE messages SET is_deleted = true WHERE message_id = $1")
        .bind(message_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_recalled(pool: &PgPool, message_id: &str) -> Result<()> {
    sqlx::query("UPDATE messages SET is_recalled = true WHERE message_id = $1")
        .bind(message_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_by_id_at(
    pool: &PgPool,
    message_id: &str,
    sent_at: chrono::DateTime<chrono::Utc>,
) -> Result<Option<Message>> {
    sqlx::query_as::<_, Message>(
        r#"SELECT message_id, room_id, sent_by, content_text, content_type,
           sent_at, is_deleted, is_recalled, is_edited, history, raw_data, source
           FROM messages
           WHERE message_id = $1 AND sent_at = $2"#,
    )
    .bind(message_id)
    .bind(sent_at)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.into())
}

pub async fn update_content(pool: &PgPool, message_id: &str, text: &str) -> Result<()> {
    sqlx::query(
        r#"UPDATE messages SET
           content_text = $1,
           is_edited = true,
           history = COALESCE(history, '[]'::jsonb) || jsonb_build_object(
               'content', content_text,
               'edited_at', NOW()
           )
           WHERE message_id = $2"#,
    )
    .bind(text)
    .bind(message_id)
    .execute(pool)
    .await?;
    Ok(())
}
