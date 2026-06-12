use anyhow::Result;
use sqlx::PgPool;
use chrono::{DateTime, Utc};

/// Service for managing room members
pub struct RoomMemberService {
    pool: PgPool,
}

impl RoomMemberService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Upsert a room member
    pub async fn upsert_member(
        &self,
        room_id: &str,
        user_id: &str,
        role: &str,
        joined_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO room_members (room_id, user_id, role, joined_at)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (room_id, user_id) DO UPDATE SET
                   role = $3,
                   joined_at = $4,
                   left_at = NULL"#,
        )
        .bind(room_id)
        .bind(user_id)
        .bind(role)
        .bind(joined_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Mark a member as left
    pub async fn mark_member_left(
        &self,
        room_id: &str,
        user_id: &str,
        left_at: Option<DateTime<Utc>>,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"UPDATE room_members SET left_at = $3
               WHERE room_id = $1 AND user_id = $2 AND left_at IS NULL"#,
        )
        .bind(room_id)
        .bind(user_id)
        .bind(left_at)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
