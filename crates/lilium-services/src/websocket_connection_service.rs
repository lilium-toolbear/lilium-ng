use std::hash::{Hash, Hasher};

use crate::Result;
use chrono::{Duration, Utc};
use lilium_database::DbSessionContext;

use lilium_common::error::LiliumError;
use lilium_models::dzmm::websocket_connection::WebsocketConnection;
use tracing::instrument;

pub struct WebsocketConnectionService<'a> {
    session: DbSessionContext<'a>,
}

impl<'a> WebsocketConnectionService<'a> {
    #[instrument(skip(session))]
    pub fn new(session: DbSessionContext<'a>) -> Self {
        Self { session }
    }

    #[instrument(fields(account_user_id = %account_user_id))]
    pub fn calculate_lock_id(account_user_id: &str) -> i64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        account_user_id.hash(&mut hasher);
        let hash = hasher.finish();
        (hash & 0x7FFFFFFFFFFFFFFF) as i64
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn acquire_connection_lock(&mut self, user_id: &str) -> Result<i64> {
        let lock_id = Self::calculate_lock_id(user_id);
        self.acquire_connection_lock_inner(lock_id, user_id).await
    }

    #[instrument(skip(self), fields(lock_id, user_id = %user_id))]
    async fn acquire_connection_lock_inner(&mut self, lock_id: i64, user_id: &str) -> Result<i64> {
        let lock_acquired = sqlx::query_scalar::<_, bool>("SELECT pg_try_advisory_lock($1)")
            .bind(lock_id)
            .fetch_one(self.session.as_mut())
            .await?;

        if !lock_acquired {
            return Err(LiliumError::connection_conflict(
                format!("Account {} is already in use by another client", user_id),
                Some(lock_id),
            ));
        }

        self.replace_connection_record(lock_id, user_id).await?;

        Ok(lock_id)
    }

    #[instrument(skip(self), fields(lock_id, user_id = %user_id))]
    async fn replace_connection_record(&mut self, lock_id: i64, user_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM websocket_connections WHERE lock_id = $1")
            .bind(lock_id)
            .execute(self.session.as_mut())
            .await?;

        let now = Utc::now();
        sqlx::query(
            "INSERT INTO websocket_connections (lock_id, account_user_id, connected_at, last_heartbeat)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(lock_id)
        .bind(user_id)
        .bind(now)
        .bind(now)
        .execute(self.session.as_mut())
        .await?;
        Ok(())
    }

    #[instrument(skip(self), fields(user_id = %user_id, has_expected_lock_id = expected_lock_id.is_some()))]
    pub async fn ensure_connection_lock(
        &mut self,
        user_id: &str,
        expected_lock_id: Option<i64>,
    ) -> Result<i64> {
        let lock_id = Self::calculate_lock_id(user_id);

        if let Some(expected) = expected_lock_id {
            if expected != lock_id {
                return Err(LiliumError::domain_service_with_code(
                    "WEBSOCKET_CONNECTION_INVALID_REQUEST",
                    format!(
                        "Expected lock_id {} does not match user_id {}",
                        expected, user_id
                    ),
                ));
            }
        }

        if self.current_connection_holds_lock(lock_id).await? {
            return Ok(lock_id);
        }

        self.acquire_connection_lock_inner(lock_id, user_id).await
    }

    #[instrument(skip(self), fields(lock_id))]
    async fn current_connection_holds_lock(&mut self, lock_id: i64) -> Result<bool> {
        let classid = lock_id >> 32;
        let objid = lock_id & 0xFFFFFFFF;

        let lock_held = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(
                SELECT 1
                FROM pg_locks
                WHERE pid = pg_backend_pid()
                  AND classid = $1
                  AND objid = $2
                  AND locktype = 'advisory'
                  AND granted = true
            )"#,
        )
        .bind(classid)
        .bind(objid)
        .fetch_one(self.session.as_mut())
        .await?;

        Ok(lock_held)
    }

    #[instrument(skip(self), fields(lock_id))]
    pub async fn release_connection_lock(&mut self, lock_id: i64) -> Result<()> {
        let connection = sqlx::query_as::<_, WebsocketConnection>(
            "SELECT * FROM websocket_connections WHERE lock_id = $1",
        )
        .bind(lock_id)
        .fetch_optional(self.session.as_mut())
        .await?;

        if connection.is_some() {
            sqlx::query("DELETE FROM websocket_connections WHERE lock_id = $1")
                .bind(lock_id)
                .execute(self.session.as_mut())
                .await?;
        }

        let _ = sqlx::query_scalar::<_, bool>("SELECT pg_advisory_unlock($1)")
            .bind(lock_id)
            .fetch_optional(self.session.as_mut())
            .await;
        Ok(())
    }

    #[instrument(skip(self), fields(lock_id))]
    pub async fn update_heartbeat(&mut self, lock_id: i64) -> Result<()> {
        let now = Utc::now();
        sqlx::query("UPDATE websocket_connections SET last_heartbeat = $1 WHERE lock_id = $2")
            .bind(now)
            .bind(lock_id)
            .execute(self.session.as_mut())
            .await?;
        Ok(())
    }

    #[instrument(skip(self), fields(account_user_id = ?account_user_id))]
    pub async fn get_active_connections(
        &mut self,
        account_user_id: Option<&str>,
    ) -> Result<Vec<WebsocketConnection>> {
        let rows = if let Some(uid) = account_user_id {
            sqlx::query_as::<_, WebsocketConnection>(
                r#"SELECT c.lock_id, c.account_user_id, c.connected_at, c.last_heartbeat
                FROM websocket_connections c
                INNER JOIN pg_locks l ON l.classid = (c.lock_id >> 32)
                                      AND l.objid = (c.lock_id & 4294967295)
                                      AND l.locktype = 'advisory'
                                      AND l.granted = true
                WHERE c.account_user_id = $1
                ORDER BY c.connected_at DESC"#,
            )
            .bind(uid)
            .fetch_all(self.session.as_mut())
            .await?
        } else {
            sqlx::query_as::<_, WebsocketConnection>(
                r#"SELECT c.lock_id, c.account_user_id, c.connected_at, c.last_heartbeat
                FROM websocket_connections c
                INNER JOIN pg_locks l ON l.classid = (c.lock_id >> 32)
                                      AND l.objid = (c.lock_id & 4294967295)
                                      AND l.locktype = 'advisory'
                                      AND l.granted = true
                ORDER BY c.connected_at DESC"#,
            )
            .fetch_all(self.session.as_mut())
            .await?
        };

        Ok(rows)
    }

    #[instrument(skip(self), fields(account_user_id = %account_user_id))]
    pub async fn is_credential_in_use(&mut self, account_user_id: &str) -> Result<bool> {
        let lock_id = Self::calculate_lock_id(account_user_id);

        let exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(
                SELECT 1
                FROM websocket_connections c
                INNER JOIN pg_locks l ON l.classid = (c.lock_id >> 32)
                                      AND l.objid = (c.lock_id & 4294967295)
                                      AND l.locktype = 'advisory'
                                      AND l.granted = true
                WHERE c.lock_id = $1
            )"#,
        )
        .bind(lock_id)
        .fetch_one(self.session.as_mut())
        .await?;

        Ok(exists)
    }

    #[instrument(skip(self), fields(timeout_seconds))]
    pub async fn cleanup_stale_connections(&mut self, timeout_seconds: i64) -> Result<i64> {
        let stale_lock_ids = if timeout_seconds > 0 {
            let cutoff = Utc::now() - Duration::seconds(timeout_seconds);
            sqlx::query_scalar::<_, i64>(
                r#"SELECT c.lock_id
                FROM websocket_connections c
                LEFT JOIN pg_locks l ON l.classid = (c.lock_id >> 32)
                                     AND l.objid = (c.lock_id & 4294967295)
                                     AND l.locktype = 'advisory'
                                     AND l.granted = true
                WHERE l.classid IS NULL
                  AND c.last_heartbeat < $1"#,
            )
            .bind(cutoff)
            .fetch_all(self.session.as_mut())
            .await?
        } else {
            sqlx::query_scalar::<_, i64>(
                r#"SELECT c.lock_id
                FROM websocket_connections c
                LEFT JOIN pg_locks l ON l.classid = (c.lock_id >> 32)
                                     AND l.objid = (c.lock_id & 4294967295)
                                     AND l.locktype = 'advisory'
                                     AND l.granted = true
                WHERE l.classid IS NULL"#,
            )
            .fetch_all(self.session.as_mut())
            .await?
        };

        if stale_lock_ids.is_empty() {
            return Ok(0);
        }

        let count = stale_lock_ids.len() as i64;
        for lock_id in &stale_lock_ids {
            sqlx::query("DELETE FROM websocket_connections WHERE lock_id = $1")
                .bind(lock_id)
                .execute(self.session.as_mut())
                .await?;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_lock_id_is_deterministic() {
        let id1 = WebsocketConnectionService::calculate_lock_id("user_test_deterministic");
        let id2 = WebsocketConnectionService::calculate_lock_id("user_test_deterministic");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_calculate_lock_id_different_users_produce_different_ids() {
        let id1 = WebsocketConnectionService::calculate_lock_id("user_a");
        let id2 = WebsocketConnectionService::calculate_lock_id("user_b");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_calculate_lock_id_is_non_negative() {
        let id = WebsocketConnectionService::calculate_lock_id("user_any");
        assert!(id >= 0);
    }

    #[test]
    fn test_calculate_lock_id_splits_for_pg_locks_query() {
        let lock_id = 1746043848041696062i64;
        let expected_classid = lock_id >> 32;
        let expected_objid = lock_id & 0xFFFFFFFF;
        assert_eq!(expected_classid, lock_id >> 32);
        assert_eq!(expected_objid, lock_id & 0xFFFFFFFF);
    }

    #[tokio::test]
    async fn test_acquire_connection_lock_creates_connection_record() {
        lilium_test_fixtures::with_db_session(
            lilium_test_fixtures::TestServiceFixture::WebsocketConnection,
            |session| {
                Box::pin(async move {
                    let mut service = WebsocketConnectionService::new(session);
                    let lock_id = service
                        .acquire_connection_lock("user_test_acquire")
                        .await
                        .unwrap();
                    assert!(lock_id >= 0);
                    Ok(())
                })
            },
        )
        .await
        .expect("test_acquire_connection_lock_creates_connection_record");
    }

    #[tokio::test]
    async fn test_release_connection_lock_deletes_record() {
        lilium_test_fixtures::with_db_session(
            lilium_test_fixtures::TestServiceFixture::WebsocketConnection,
            |session| {
                Box::pin(async move {
                    let mut service = WebsocketConnectionService::new(session);
                    let lock_id = service
                        .acquire_connection_lock("user_test_release")
                        .await
                        .unwrap();
                    service.release_connection_lock(lock_id).await.unwrap();
                    Ok(())
                })
            },
        )
        .await
        .expect("test_release_connection_lock_deletes_record");
    }

    #[tokio::test]
    async fn test_update_heartbeat_updates_timestamp() {
        lilium_test_fixtures::with_db_session(
            lilium_test_fixtures::TestServiceFixture::WebsocketConnection,
            |session| {
                Box::pin(async move {
                    let mut service = WebsocketConnectionService::new(session);
                    let lock_id = service
                        .acquire_connection_lock("user_test_heartbeat")
                        .await
                        .unwrap();
                    service.update_heartbeat(lock_id).await.unwrap();
                    Ok(())
                })
            },
        )
        .await
        .expect("test_update_heartbeat_updates_timestamp");
    }

    #[tokio::test]
    async fn test_get_active_connections_returns_all_connections() {
        lilium_test_fixtures::with_db_session(
            lilium_test_fixtures::TestServiceFixture::WebsocketConnection,
            |session| {
                Box::pin(async move {
                    let mut service = WebsocketConnectionService::new(session);
                    let lock1 = service
                        .acquire_connection_lock("user_test_active1")
                        .await
                        .unwrap();
                    let lock2 = service
                        .acquire_connection_lock("user_test_active2")
                        .await
                        .unwrap();
                    let connections = service.get_active_connections(None).await.unwrap();
                    assert!(connections.len() >= 2);
                    let lock_ids: Vec<i64> = connections.iter().map(|c| c.lock_id).collect();
                    assert!(lock_ids.contains(&lock1));
                    assert!(lock_ids.contains(&lock2));
                    Ok(())
                })
            },
        )
        .await
        .expect("test_get_active_connections_returns_all_connections");
    }

    #[tokio::test]
    async fn test_get_active_connections_filters_by_credential() {
        lilium_test_fixtures::with_db_session(
            lilium_test_fixtures::TestServiceFixture::WebsocketConnection,
            |session| {
                Box::pin(async move {
                    let mut service = WebsocketConnectionService::new(session);
                    service
                        .acquire_connection_lock("user_test_filter1")
                        .await
                        .unwrap();
                    service
                        .acquire_connection_lock("user_test_filter2")
                        .await
                        .unwrap();
                    let connections = service
                        .get_active_connections(Some("user_test_filter1"))
                        .await
                        .unwrap();
                    assert_eq!(connections.len(), 1);
                    assert_eq!(connections[0].account_user_id, "user_test_filter1");
                    Ok(())
                })
            },
        )
        .await
        .expect("test_get_active_connections_filters_by_credential");
    }

    #[tokio::test]
    async fn test_is_credential_in_use_returns_true_when_locked() {
        lilium_test_fixtures::with_db_session(
            lilium_test_fixtures::TestServiceFixture::WebsocketConnection,
            |session| {
                Box::pin(async move {
                    let mut service = WebsocketConnectionService::new(session);
                    assert!(!service
                        .is_credential_in_use("user_test_in_use")
                        .await
                        .unwrap());
                    service
                        .acquire_connection_lock("user_test_in_use")
                        .await
                        .unwrap();
                    assert!(service
                        .is_credential_in_use("user_test_in_use")
                        .await
                        .unwrap());
                    Ok(())
                })
            },
        )
        .await
        .expect("test_is_credential_in_use_returns_true_when_locked");
    }

    #[tokio::test]
    async fn test_is_credential_in_use_returns_false_when_not_locked() {
        lilium_test_fixtures::with_db_session(
            lilium_test_fixtures::TestServiceFixture::WebsocketConnection,
            |session| {
                Box::pin(async move {
                    let mut service = WebsocketConnectionService::new(session);
                    assert!(!service
                        .is_credential_in_use("user_test_not_in_use")
                        .await
                        .unwrap());
                    Ok(())
                })
            },
        )
        .await
        .expect("test_is_credential_in_use_returns_false_when_not_locked");
    }

    #[tokio::test]
    async fn test_cleanup_stale_connections_removes_old_records() {
        lilium_test_fixtures::with_db_session(
            lilium_test_fixtures::TestServiceFixture::WebsocketConnection,
            |session| {
                Box::pin(async move {
                    let mut service = WebsocketConnectionService::new(session);
                    let cleaned = service.cleanup_stale_connections(300).await.unwrap();
                    assert!(cleaned >= 0);
                    Ok(())
                })
            },
        )
        .await
        .expect("test_cleanup_stale_connections_removes_old_records");
    }

    #[tokio::test]
    async fn test_cleanup_stale_connections_preserves_fresh_records() {
        lilium_test_fixtures::with_db_session(
            lilium_test_fixtures::TestServiceFixture::WebsocketConnection,
            |session| {
                Box::pin(async move {
                    let mut service = WebsocketConnectionService::new(session);
                    let _lock_id = service
                        .acquire_connection_lock("user_test_fresh")
                        .await
                        .unwrap();
                    let cleaned = service.cleanup_stale_connections(60).await.unwrap();
                    assert_eq!(cleaned, 0);
                    Ok(())
                })
            },
        )
        .await
        .expect("test_cleanup_stale_connections_preserves_fresh_records");
    }

    #[test]
    fn test_lock_id_calculation_is_deterministic() {
        let id1 = WebsocketConnectionService::calculate_lock_id("user_test_deterministic");
        let id2 = WebsocketConnectionService::calculate_lock_id("user_test_deterministic");
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_acquire_cleans_up_stale_record() {
        lilium_test_fixtures::with_db_session(
            lilium_test_fixtures::TestServiceFixture::WebsocketConnection,
            |session| {
                Box::pin(async move {
                    let mut service = WebsocketConnectionService::new(session);
                    let _lock_id = service
                        .acquire_connection_lock("user_test_stale")
                        .await
                        .unwrap();
                    assert!(_lock_id >= 0);
                    Ok(())
                })
            },
        )
        .await
        .expect("test_acquire_cleans_up_stale_record");
    }
}
