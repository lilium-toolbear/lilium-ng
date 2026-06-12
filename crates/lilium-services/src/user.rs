use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

/// Service for fetching and updating user information
pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Batch fetch and update users from API
    /// Returns (new_count, updated_count)
    pub async fn batch_fetch_and_update(
        &self,
        user_room_pairs: &[(String, String)],
    ) -> Result<(i64, i64)> {
        let mut new_count = 0;
        let mut updated_count = 0;

        for (user_id, _room_id) in user_room_pairs {
            // Check if user exists
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1)"
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

            if exists {
                // Update user
                sqlx::query(
                    r#"UPDATE users SET last_seen = NOW() WHERE user_id = $1"#,
                )
                .bind(user_id)
                .execute(&self.pool)
                .await?;
                updated_count += 1;
            } else {
                // Create new user
                sqlx::query(
                    r#"INSERT INTO users (user_id, created_at, updated_at)
                       VALUES ($1, NOW(), NOW())
                       ON CONFLICT (user_id) DO NOTHING"#,
                )
                .bind(user_id)
                .execute(&self.pool)
                .await?;
                new_count += 1;
            }
        }

        info!(
            new = new_count,
            updated = updated_count,
            total = user_room_pairs.len(),
            "Batch fetched users"
        );

        Ok((new_count, updated_count))
    }
}
