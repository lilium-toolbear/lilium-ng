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

    /// Batch fetch and update users
    /// Updates last_seen for existing users, creates new users
    pub async fn batch_fetch_and_update(
        &self,
        user_room_pairs: &[(String, String)],
    ) -> Result<(i64, i64)> {
        let mut new_count = 0;
        let mut updated_count = 0;

        for (user_id, _room_id) in user_room_pairs {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1)"
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

            if exists {
                sqlx::query(
                    "UPDATE users SET last_seen = NOW(), updated_at = NOW() WHERE user_id = $1",
                )
                .bind(user_id)
                .execute(&self.pool)
                .await?;
                updated_count += 1;
            } else {
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

    /// Fetch user profile from database
    pub async fn fetch_user_profile(&self, user_id: &str) -> Result<Option<UserProfile>> {
        let user = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
            "SELECT user_id, full_name, avatar_url FROM users WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(|(uid, name, avatar)| UserProfile {
            user_id: uid,
            display_name: name,
            avatar_url: avatar,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub user_id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
