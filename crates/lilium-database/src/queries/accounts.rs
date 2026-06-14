use anyhow::Result;
use sqlx::{Executor, Postgres};

pub async fn list_enabled_account_ids<'e, E>(exec: &mut E) -> Result<Vec<String>>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
    let ids = sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM dzmm_account WHERE is_enabled = true ORDER BY user_id",
    )
    .fetch_all(&mut *exec)
    .await?;
    Ok(ids)
}
