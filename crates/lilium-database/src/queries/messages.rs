use anyhow::Result;
use lilium_models::dzmm::message::Message;
use sqlx::{Executor, Postgres};

pub async fn create_message_if_missing<'e, E>(exec: &mut E, message: &Message) -> Result<bool>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
    let result = sqlx::query(
        r#"INSERT INTO messages (
               message_id, room_id, sent_at, sent_by, content_type, content_text,
               raw_data, source, created_at, is_deleted, is_recalled, is_edited, history
           ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
           ON CONFLICT DO NOTHING"#,
    )
    .bind(&message.message_id)
    .bind(&message.room_id)
    .bind(message.sent_at)
    .bind(&message.sent_by)
    .bind(&message.content_type)
    .bind(&message.content_text)
    .bind(&message.raw_data)
    .bind(&message.source)
    .bind(message.created_at)
    .bind(message.is_deleted)
    .bind(message.is_recalled)
    .bind(message.is_edited)
    .bind(&message.history)
    .execute(&mut *exec)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn mark_deleted<'e, E>(exec: &mut E, message_id: &str) -> Result<()>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
    sqlx::query(
        r#"UPDATE messages
           SET is_deleted = true, deleted_at = NOW(), updated_at = NOW()
           WHERE message_id = $1"#,
    )
    .bind(message_id)
    .execute(&mut *exec)
    .await?;
    Ok(())
}

pub async fn mark_recalled<'e, E>(exec: &mut E, message_id: &str) -> Result<()>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
    sqlx::query("UPDATE messages SET is_recalled = true, updated_at = NOW() WHERE message_id = $1")
        .bind(message_id)
        .execute(&mut *exec)
        .await?;
    Ok(())
}

pub async fn get_by_id_at<'e, E>(
    exec: &mut E,
    message_id: &str,
    sent_at: chrono::DateTime<chrono::Utc>,
) -> Result<Option<Message>>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
    sqlx::query_as::<_, Message>(
        r#"SELECT message_id, room_id, sent_at, sent_by, content_type, content_text,
           content_tsv::text, attachment_url, attachment_file, sticker_id, alt_text,
           metadata, raw_data, source, created_at, updated_at,
           is_deleted, deleted_at, deleted_by, is_recalled, is_edited,
           history, reference_message_id, reference_data
           FROM messages
           WHERE message_id = $1 AND sent_at = $2"#,
    )
    .bind(message_id)
    .bind(sent_at)
    .fetch_optional(&mut *exec)
    .await
    .map_err(|e| e.into())
}

pub async fn update_content<'e, E>(exec: &mut E, message_id: &str, text: &str) -> Result<()>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
    sqlx::query(
        r#"UPDATE messages SET
           content_text = $1,
           is_edited = true,
           updated_at = NOW(),
           history = COALESCE(history, '[]'::jsonb) || jsonb_build_object(
               'content', content_text,
               'edited_at', NOW()
           )
           WHERE message_id = $2"#,
    )
    .bind(text)
    .bind(message_id)
    .execute(&mut *exec)
    .await?;
    Ok(())
}
