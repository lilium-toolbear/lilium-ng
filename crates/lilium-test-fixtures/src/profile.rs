use anyhow::Result;
use lilium_database::DbPool;
use lilium_database::pool::SessionFuture;
use lilium_database::{DbSession, DbSessionContext};

use crate::database::{TestDatabaseConnection, connect_test_database};
use crate::reset::reset_database;
use crate::seeds::{
    seed_message_profile, seed_shared_profile, seed_user_profile, seed_websocket_profile,
};

#[derive(Copy, Clone, Debug)]
pub enum FixtureProfile {
    Empty,
    Shared,
    RoomMember,
    WebsocketConnection,
    OutgoingCommand,
    Event,
    User,
    Account,
    Message,
    Notification,
}

pub struct TestDb {
    database: lilium_database::Database,
    _connection: TestDatabaseConnection,
}

impl TestDb {
    pub async fn acquire(profile: FixtureProfile) -> Result<Self> {
        let connection = connect_test_database().await;

        connection
            .with_session_context(|session| {
                Box::pin(async move {
                    let mut session = session;
                    prepare_database(&mut session, profile).await
                })
            })
            .await?;

        let database = lilium_database::Database::create(connection.database_config()).await?;

        Ok(Self {
            database,
            _connection: connection,
        })
    }

    pub fn database(&self) -> &lilium_database::Database {
        &self.database
    }

    pub fn raw_pool(&self) -> &DbPool {
        self.database.raw_pool()
    }
}

pub async fn prepare_database(session: &mut DbSession, profile: FixtureProfile) -> Result<()> {
    reset_database(session).await?;

    match profile {
        FixtureProfile::Empty
        | FixtureProfile::RoomMember
        | FixtureProfile::OutgoingCommand
        | FixtureProfile::Event
        | FixtureProfile::Account
        | FixtureProfile::Notification => {}
        FixtureProfile::Shared => seed_shared_profile(session).await?,
        FixtureProfile::User => seed_user_profile(session).await?,
        FixtureProfile::Message => seed_message_profile(session).await?,
        FixtureProfile::WebsocketConnection => seed_websocket_profile(session).await?,
    }

    Ok(())
}

pub async fn with_db_session<T, F>(profile: FixtureProfile, f: F) -> Result<T>
where
    F: for<'a> FnOnce(DbSessionContext<'a>) -> SessionFuture<'a, T> + Send + 'static,
{
    let pool = connect_test_database().await;
    pool.with_rollback_session_context(|session| {
        Box::pin(async move {
            let mut session = session;
            prepare_database(&mut session, profile).await?;
            f(session).await
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires TEST_DATABASE_URL"]
    async fn acquire_empty_profile_exposes_database_runtime() {
        let test_db = TestDb::acquire(FixtureProfile::Empty)
            .await
            .expect("acquire test database guard");

        let mut conn = test_db.database().raw_connection().await.unwrap();
        let public_table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pg_catalog.pg_tables WHERE schemaname = 'public'",
        )
        .fetch_one(conn.as_mut())
        .await
        .unwrap();

        assert!(
            public_table_count > 0,
            "expected the live schema bootstrap to create public tables"
        );
    }
}

pub async fn with_db_session_and_pool<T, F>(profile: FixtureProfile, f: F) -> Result<T>
where
    F: for<'a> FnOnce(DbSessionContext<'a>, TestDatabaseConnection) -> SessionFuture<'a, T>
        + Send
        + 'static,
{
    let pool = connect_test_database().await;
    let pool_for_callback = pool.clone();

    pool.with_rollback_session_context(move |session| {
        let pool_for_callback = pool_for_callback.clone();
        Box::pin(async move {
            let mut session = session;
            prepare_database(&mut session, profile).await?;
            f(session, pool_for_callback).await
        })
    })
    .await
}
