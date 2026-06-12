use sqlx::PgPool;
use anyhow::Result;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Account {
    pub user_id: String,
    pub is_enabled: bool,
}

pub async fn list_enabled_account_ids(pool: &PgPool) -> Result<Vec<String>> {
    let ids = sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM accounts WHERE is_enabled = true ORDER BY user_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(ids)
}
