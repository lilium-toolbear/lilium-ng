use anyhow::Result;
use lilium_database::Database;
use sea_orm::ConnectionTrait;

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

        let database = Database::create(connection.database_config()).await?;
        lilium_database::transaction!(database, |conn| { prepare_database(conn, profile).await })
            .await?;

        Ok(Self {
            database,
            _connection: connection,
        })
    }

    pub fn database(&self) -> &lilium_database::Database {
        &self.database
    }
}

pub async fn prepare_database<C: ConnectionTrait>(db: &C, profile: FixtureProfile) -> Result<()> {
    reset_database(db).await?;

    match profile {
        FixtureProfile::Empty
        | FixtureProfile::RoomMember
        | FixtureProfile::OutgoingCommand
        | FixtureProfile::Event
        | FixtureProfile::Account
        | FixtureProfile::Notification => {}
        FixtureProfile::Shared => seed_shared_profile(db).await?,
        FixtureProfile::User => seed_user_profile(db).await?,
        FixtureProfile::Message => seed_message_profile(db).await?,
        FixtureProfile::WebsocketConnection => seed_websocket_profile(db).await?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
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
