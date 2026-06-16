use crate::Result;
use chrono::{Duration, Utc};
use lilium_models::dzmm::websocket_connection as websocket_connections;
use md5::{Digest, Md5};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set,
    Statement, Value,
};

use lilium_common::error::LiliumError;
use tracing::instrument;

type WebsocketConnection = websocket_connections::Model;

#[instrument(fields(account_user_id = %account_user_id))]
pub fn calculate_lock_id(account_user_id: &str) -> i64 {
    let hash = Md5::digest(account_user_id.as_bytes());
    let mut first_eight = [0u8; 8];
    first_eight.copy_from_slice(&hash[..8]);
    let lock_id = u64::from_be_bytes(first_eight) & 0x7FFFFFFFFFFFFFFF;
    lock_id as i64
}

fn split_advisory_lock_id(lock_id: i64) -> (i64, i64) {
    (lock_id >> 32, lock_id & 0xFFFFFFFF)
}

fn statement<C: ConnectionTrait>(db: &C, sql: impl Into<String>, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(db.get_database_backend(), sql, values)
}

async fn query_bool<C: ConnectionTrait>(
    db: &C,
    sql: impl Into<String>,
    values: Vec<Value>,
) -> Result<bool> {
    let row = db
        .query_one(statement(db, sql, values))
        .await?
        .ok_or_else(|| LiliumError::database("expected one boolean row"))?;
    row.try_get("", "value")
        .map_err(|error| LiliumError::database(error.to_string()))
}

#[allow(clippy::result_large_err)]
async fn query_i64s<C: ConnectionTrait>(
    db: &C,
    sql: impl Into<String>,
    values: Vec<Value>,
) -> Result<Vec<i64>> {
    let rows = db.query_all(statement(db, sql, values)).await?;
    rows.into_iter()
        .map(|row| {
            row.try_get("", "value")
                .map_err(|error| LiliumError::database(error.to_string()))
        })
        .collect()
}

#[instrument(skip(db), fields(user_id = %user_id))]
pub async fn acquire_connection_lock(db: &impl ConnectionTrait, user_id: &str) -> Result<i64> {
    let lock_id = calculate_lock_id(user_id);
    acquire_connection_lock_inner(db, lock_id, user_id).await
}

#[instrument(skip(db), fields(lock_id, user_id = %user_id))]
async fn acquire_connection_lock_inner(
    db: &impl ConnectionTrait,
    lock_id: i64,
    user_id: &str,
) -> Result<i64> {
    let lock_acquired = query_bool(
        db,
        "SELECT pg_try_advisory_lock($1) AS value",
        vec![lock_id.into()],
    )
    .await?;

    if !lock_acquired {
        return Err(LiliumError::connection_conflict(
            format!("Account {} is already in use by another client", user_id),
            Some(lock_id),
        ));
    }

    replace_connection_record(db, lock_id, user_id).await?;

    Ok(lock_id)
}

#[instrument(skip(db), fields(lock_id, user_id = %user_id))]
async fn replace_connection_record(
    db: &impl ConnectionTrait,
    lock_id: i64,
    user_id: &str,
) -> Result<()> {
    websocket_connections::Entity::delete_many()
        .filter(websocket_connections::Column::LockId.eq(lock_id))
        .exec(db)
        .await?;

    let now = Utc::now();
    websocket_connections::ActiveModel {
        lock_id: Set(lock_id),
        account_user_id: Set(user_id.to_owned()),
        connected_at: Set(now),
        last_heartbeat: Set(now),
    }
    .insert(db)
    .await?;
    Ok(())
}

#[instrument(skip(db), fields(user_id = %user_id, has_expected_lock_id = expected_lock_id.is_some()))]
pub async fn ensure_connection_lock(
    db: &impl ConnectionTrait,
    user_id: &str,
    expected_lock_id: Option<i64>,
) -> Result<i64> {
    let lock_id = calculate_lock_id(user_id);

    if let Some(expected) = expected_lock_id
        && expected != lock_id
    {
        return Err(LiliumError::domain_service_with_code(
            "WEBSOCKET_CONNECTION_INVALID_REQUEST",
            format!(
                "Expected lock_id {} does not match user_id {}",
                expected, user_id
            ),
        ));
    }

    if current_connection_holds_lock(db, lock_id).await? {
        return Ok(lock_id);
    }

    acquire_connection_lock_inner(db, lock_id, user_id).await
}

#[instrument(skip(db), fields(lock_id))]
async fn current_connection_holds_lock(db: &impl ConnectionTrait, lock_id: i64) -> Result<bool> {
    let (classid, objid) = split_advisory_lock_id(lock_id);

    query_bool(
        db,
        r#"SELECT EXISTS(
                SELECT 1
                FROM pg_locks
                WHERE pid = pg_backend_pid()
                  AND classid = $1
                  AND objid = $2
                  AND locktype = 'advisory'
                  AND granted = true
            ) AS value"#,
        vec![classid.into(), objid.into()],
    )
    .await
}

#[instrument(skip(db), fields(lock_id))]
pub async fn release_connection_lock(db: &impl ConnectionTrait, lock_id: i64) -> Result<()> {
    let connection = websocket_connections::Entity::find_by_id(lock_id)
        .one(db)
        .await?;

    if connection.is_some() {
        websocket_connections::Entity::delete_by_id(lock_id)
            .exec(db)
            .await?;
    }

    let _ = query_bool(
        db,
        "SELECT pg_advisory_unlock($1) AS value",
        vec![lock_id.into()],
    )
    .await;
    Ok(())
}

#[instrument(skip(db), fields(lock_id))]
pub async fn update_heartbeat(db: &impl ConnectionTrait, lock_id: i64) -> Result<()> {
    let now = Utc::now();
    websocket_connections::Entity::update_many()
        .set(websocket_connections::ActiveModel {
            last_heartbeat: Set(now),
            ..Default::default()
        })
        .filter(websocket_connections::Column::LockId.eq(lock_id))
        .exec(db)
        .await?;
    Ok(())
}

#[instrument(skip(db), fields(account_user_id = ?account_user_id))]
pub async fn get_active_connections(
    db: &impl ConnectionTrait,
    account_user_id: Option<&str>,
) -> Result<Vec<WebsocketConnection>> {
    let lock_is_held = Expr::cust(
        r#"EXISTS(
                SELECT 1
                FROM pg_locks l
                WHERE l.classid = (websocket_connections.lock_id >> 32)
                  AND l.objid = (websocket_connections.lock_id & 4294967295)
                  AND l.locktype = 'advisory'
                  AND l.granted = true
            )"#,
    );

    let mut query = websocket_connections::Entity::find()
        .filter(lock_is_held)
        .order_by_desc(websocket_connections::Column::ConnectedAt);

    if let Some(uid) = account_user_id {
        query = query.filter(websocket_connections::Column::AccountUserId.eq(uid));
    }

    let rows = query.all(db).await?;
    Ok(rows.into_iter().collect())
}

#[instrument(skip(db), fields(account_user_id = %account_user_id))]
pub async fn is_credential_in_use(
    db: &impl ConnectionTrait,
    account_user_id: &str,
) -> Result<bool> {
    let lock_id = calculate_lock_id(account_user_id);

    query_bool(
        db,
        r#"SELECT EXISTS(
                SELECT 1
                FROM websocket_connections c
                INNER JOIN pg_locks l ON l.classid = (c.lock_id >> 32)
                                      AND l.objid = (c.lock_id & 4294967295)
                                      AND l.locktype = 'advisory'
                                      AND l.granted = true
                WHERE c.lock_id = $1
            ) AS value"#,
        vec![lock_id.into()],
    )
    .await
}

#[instrument(skip(db), fields(timeout_seconds))]
pub async fn cleanup_stale_connections(
    db: &impl ConnectionTrait,
    timeout_seconds: i64,
) -> Result<i64> {
    let stale_lock_ids = if timeout_seconds > 0 {
        let cutoff = Utc::now() - Duration::seconds(timeout_seconds);
        query_i64s(
            db,
            r#"SELECT c.lock_id AS value
               FROM websocket_connections c
               LEFT JOIN pg_locks l ON l.classid = (c.lock_id >> 32)
                                    AND l.objid = (c.lock_id & 4294967295)
                                    AND l.locktype = 'advisory'
                                    AND l.granted = true
               WHERE l.classid IS NULL
                 AND c.last_heartbeat < $1"#,
            vec![cutoff.into()],
        )
        .await?
    } else {
        query_i64s(
            db,
            r#"SELECT c.lock_id AS value
               FROM websocket_connections c
               LEFT JOIN pg_locks l ON l.classid = (c.lock_id >> 32)
                                    AND l.objid = (c.lock_id & 4294967295)
                                    AND l.locktype = 'advisory'
                                    AND l.granted = true
               WHERE l.classid IS NULL"#,
            Vec::new(),
        )
        .await?
    };

    if stale_lock_ids.is_empty() {
        return Ok(0);
    }

    let count = stale_lock_ids.len() as i64;
    websocket_connections::Entity::delete_many()
        .filter(websocket_connections::Column::LockId.is_in(stale_lock_ids))
        .exec(db)
        .await?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_lock_id_is_deterministic() {
        let id1 = calculate_lock_id("user_test_deterministic");
        let id2 = calculate_lock_id("user_test_deterministic");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_calculate_lock_id_matches_python_md5_algorithm() {
        let id = calculate_lock_id("f9791c4c-6103-4fbb-8910-c11ae47772b3");
        assert_eq!(id, 1746043848041696062);
    }

    #[test]
    fn test_calculate_lock_id_different_users_produce_different_ids() {
        let id1 = calculate_lock_id("user_a");
        let id2 = calculate_lock_id("user_b");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_calculate_lock_id_is_non_negative() {
        let id = calculate_lock_id("user_any");
        assert!(id >= 0);
    }

    #[test]
    fn test_advisory_lock_parts_reconstruct_lock_id() {
        let lock_id = calculate_lock_id("user_test_split");
        let (classid, objid) = split_advisory_lock_id(lock_id);
        assert_eq!((classid << 32) | objid, lock_id);
    }

    #[tokio::test]
    async fn test_acquire_connection_lock_creates_connection_record() {
        let test_db = lilium_test_fixtures::TestDb::acquire(
            lilium_test_fixtures::FixtureProfile::WebsocketConnection,
        )
        .await
        .expect("init websocket db");

        lilium_database::transaction!(test_db.database(), |session| {
            let lock_id = acquire_connection_lock(session, "user_test_acquire")
                .await
                .unwrap();
            assert!(lock_id >= 0);
            Ok(())
        })
        .await
        .expect("test_acquire_connection_lock_creates_connection_record");
    }

    #[tokio::test]
    async fn test_release_connection_lock_deletes_record() {
        let test_db = lilium_test_fixtures::TestDb::acquire(
            lilium_test_fixtures::FixtureProfile::WebsocketConnection,
        )
        .await
        .expect("init websocket db");

        lilium_database::transaction!(test_db.database(), |session| {
            let lock_id = acquire_connection_lock(session, "user_test_release")
                .await
                .unwrap();
            release_connection_lock(session, lock_id).await.unwrap();
            Ok(())
        })
        .await
        .expect("test_release_connection_lock_deletes_record");
    }

    #[tokio::test]
    async fn test_update_heartbeat_updates_timestamp() {
        let test_db = lilium_test_fixtures::TestDb::acquire(
            lilium_test_fixtures::FixtureProfile::WebsocketConnection,
        )
        .await
        .expect("init websocket db");

        lilium_database::transaction!(test_db.database(), |session| {
            let lock_id = acquire_connection_lock(session, "user_test_heartbeat")
                .await
                .unwrap();
            update_heartbeat(session, lock_id).await.unwrap();
            Ok(())
        })
        .await
        .expect("test_update_heartbeat_updates_timestamp");
    }

    #[tokio::test]
    async fn test_get_active_connections_returns_all_connections() {
        let test_db = lilium_test_fixtures::TestDb::acquire(
            lilium_test_fixtures::FixtureProfile::WebsocketConnection,
        )
        .await
        .expect("init websocket db");

        lilium_database::transaction!(test_db.database(), |session| {
            let lock1 = acquire_connection_lock(session, "user_test_active1")
                .await
                .unwrap();
            let lock2 = acquire_connection_lock(session, "user_test_active2")
                .await
                .unwrap();
            let connections = get_active_connections(session, None).await.unwrap();
            assert!(connections.len() >= 2);
            let lock_ids: Vec<i64> = connections.iter().map(|c| c.lock_id).collect();
            assert!(lock_ids.contains(&lock1));
            assert!(lock_ids.contains(&lock2));
            Ok(())
        })
        .await
        .expect("test_get_active_connections_returns_all_connections");
    }

    #[tokio::test]
    async fn test_get_active_connections_filters_by_credential() {
        let test_db = lilium_test_fixtures::TestDb::acquire(
            lilium_test_fixtures::FixtureProfile::WebsocketConnection,
        )
        .await
        .expect("init websocket db");

        lilium_database::transaction!(test_db.database(), |session| {
            acquire_connection_lock(session, "user_test_filter1")
                .await
                .unwrap();
            acquire_connection_lock(session, "user_test_filter2")
                .await
                .unwrap();
            let connections = get_active_connections(session, Some("user_test_filter1"))
                .await
                .unwrap();
            assert_eq!(connections.len(), 1);
            assert_eq!(connections[0].account_user_id, "user_test_filter1");
            Ok(())
        })
        .await
        .expect("test_get_active_connections_filters_by_credential");
    }

    #[tokio::test]
    async fn test_is_credential_in_use_returns_true_when_locked() {
        let test_db = lilium_test_fixtures::TestDb::acquire(
            lilium_test_fixtures::FixtureProfile::WebsocketConnection,
        )
        .await
        .expect("init websocket db");

        lilium_database::transaction!(test_db.database(), |session| {
            assert!(
                !is_credential_in_use(session, "user_test_in_use")
                    .await
                    .unwrap()
            );
            acquire_connection_lock(session, "user_test_in_use")
                .await
                .unwrap();
            assert!(
                is_credential_in_use(session, "user_test_in_use")
                    .await
                    .unwrap()
            );
            Ok(())
        })
        .await
        .expect("test_is_credential_in_use_returns_true_when_locked");
    }

    #[tokio::test]
    async fn test_is_credential_in_use_returns_false_when_not_locked() {
        let test_db = lilium_test_fixtures::TestDb::acquire(
            lilium_test_fixtures::FixtureProfile::WebsocketConnection,
        )
        .await
        .expect("init websocket db");

        lilium_database::transaction!(test_db.database(), |session| {
            assert!(
                !is_credential_in_use(session, "user_test_not_in_use")
                    .await
                    .unwrap()
            );
            Ok(())
        })
        .await
        .expect("test_is_credential_in_use_returns_false_when_not_locked");
    }

    #[tokio::test]
    async fn test_cleanup_stale_connections_removes_old_records() {
        let test_db = lilium_test_fixtures::TestDb::acquire(
            lilium_test_fixtures::FixtureProfile::WebsocketConnection,
        )
        .await
        .expect("init websocket db");

        lilium_database::transaction!(test_db.database(), |session| {
            let cleaned = cleanup_stale_connections(session, 300).await.unwrap();
            assert!(cleaned >= 0);
            Ok(())
        })
        .await
        .expect("test_cleanup_stale_connections_removes_old_records");
    }

    #[tokio::test]
    async fn test_cleanup_stale_connections_preserves_fresh_records() {
        let test_db = lilium_test_fixtures::TestDb::acquire(
            lilium_test_fixtures::FixtureProfile::WebsocketConnection,
        )
        .await
        .expect("init websocket db");

        lilium_database::transaction!(test_db.database(), |session| {
            let _lock_id = acquire_connection_lock(session, "user_test_fresh")
                .await
                .unwrap();
            let cleaned = cleanup_stale_connections(session, 60).await.unwrap();
            assert_eq!(cleaned, 0);
            Ok(())
        })
        .await
        .expect("test_cleanup_stale_connections_preserves_fresh_records");
    }

    #[test]
    fn test_lock_id_calculation_is_deterministic() {
        let id1 = calculate_lock_id("user_test_deterministic");
        let id2 = calculate_lock_id("user_test_deterministic");
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_acquire_cleans_up_stale_record() {
        let test_db = lilium_test_fixtures::TestDb::acquire(
            lilium_test_fixtures::FixtureProfile::WebsocketConnection,
        )
        .await
        .expect("init websocket db");

        lilium_database::transaction!(test_db.database(), |session| {
            let _lock_id = acquire_connection_lock(session, "user_test_stale")
                .await
                .unwrap();
            assert!(_lock_id >= 0);
            Ok(())
        })
        .await
        .expect("test_acquire_cleans_up_stale_record");
    }
}
