use anyhow::Result;
use lilium_database::pool::SessionFuture;
use lilium_database::{DbSession, DbSessionContext};

use crate::database::{connect_test_database, TestDatabaseConnection};
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
