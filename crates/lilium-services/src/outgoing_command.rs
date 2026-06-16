use crate::Result;
use chrono::{Duration, Utc};
use lilium_models::dzmm::outgoing_command::{self as outgoing_commands, status};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use tracing::instrument;

type OutgoingCommand = outgoing_commands::Model;

const STANDARD_MAX_ATTEMPTS: i32 = 3;
const MESSAGE_SEND_RATE_LIMIT_MAX_ATTEMPTS: i32 = 6;
const MESSAGE_SEND_RATE_LIMIT_DELAYS: &[i64] = &[5, 10, 15, 20, 30];
const RATE_LIMIT_ERROR_MARKERS: &[&str] = &[
    "发送消息过于频繁",
    "过于频繁",
    "too many requests",
    "rate limit",
    "rate limited",
    "429",
];

#[instrument(fields(event = %event))]
pub(crate) fn default_max_attempts_for_event(event: &str) -> i32 {
    if event == "message:send" {
        MESSAGE_SEND_RATE_LIMIT_MAX_ATTEMPTS
    } else {
        STANDARD_MAX_ATTEMPTS
    }
}

#[instrument(fields(has_error = error_message.is_some()))]
pub(crate) fn is_rate_limited_error(error_message: Option<&str>) -> bool {
    let msg = match error_message {
        Some(m) => m,
        None => return false,
    };
    let normalized = msg.to_lowercase();
    RATE_LIMIT_ERROR_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
}

#[instrument(skip(command), fields(event = %command.event))]
pub(crate) fn is_rate_limited_message_send(command: &OutgoingCommand) -> bool {
    command.event == "message:send" && is_rate_limited_error(command.error_message.as_deref())
}

#[instrument(skip(command), fields(command_id = command.id))]
pub(crate) fn rate_limited_retry_not_before(command: &OutgoingCommand) -> chrono::DateTime<Utc> {
    let delay_limit = command
        .attempt_count
        .min(MESSAGE_SEND_RATE_LIMIT_DELAYS.len() as i32) as usize;
    let total_delay: i64 = MESSAGE_SEND_RATE_LIMIT_DELAYS[..delay_limit].iter().sum();
    command.created_at + Duration::seconds(total_delay)
}

#[instrument(skip(command), fields(command_id = command.id, now = %now))]
pub(crate) fn is_ready_for_processing(
    command: &OutgoingCommand,
    now: chrono::DateTime<Utc>,
) -> bool {
    if command.status != status::PENDING {
        return false;
    }
    if !is_rate_limited_message_send(command) {
        return true;
    }
    rate_limited_retry_not_before(command) <= now
}

#[instrument(skip(db, data), fields(account_user_id = %account_user_id, event = %event, require_ack, has_max_attempts = max_attempts.is_some()))]
pub async fn create_command<C>(
    db: &C,
    account_user_id: &str,
    event: &str,
    data: serde_json::Value,
    require_ack: bool,
    max_attempts: Option<i32>,
) -> Result<OutgoingCommand>
where
    C: ConnectionTrait,
{
    let max_attempts = max_attempts.unwrap_or_else(|| default_max_attempts_for_event(event));

    let command = outgoing_commands::ActiveModel {
        account_user_id: Set(account_user_id.to_owned()),
        event: Set(event.to_owned()),
        data: Set(data),
        require_ack: Set(require_ack),
        status: Set(status::PENDING.to_owned()),
        attempt_count: Set(0),
        max_attempts: Set(max_attempts),
        created_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(command)
}

#[instrument(skip(db), fields(account_user_id = %account_user_id, limit))]
pub async fn get_pending_commands<C>(
    db: &C,
    account_user_id: &str,
    limit: i64,
) -> Result<Vec<OutgoingCommand>>
where
    C: ConnectionTrait,
{
    if limit <= 0 {
        return Ok(Vec::new());
    }

    let rows = outgoing_commands::Entity::find()
        .filter(outgoing_commands::Column::AccountUserId.eq(account_user_id))
        .filter(outgoing_commands::Column::Status.eq(status::PENDING))
        .order_by_asc(outgoing_commands::Column::Id)
        .limit(limit as u64)
        .all(db)
        .await?
        .into_iter()
        .collect::<Vec<_>>();

    let now = Utc::now();
    let mut ready_commands = Vec::new();
    for command in rows {
        if !is_ready_for_processing(&command, now) {
            break;
        }
        ready_commands.push(command);
    }
    Ok(ready_commands)
}

#[instrument(skip(db), fields(command_id))]
pub async fn get_command<C>(db: &C, command_id: i32) -> Result<Option<OutgoingCommand>>
where
    C: ConnectionTrait,
{
    let command = outgoing_commands::Entity::find_by_id(command_id)
        .one(db)
        .await?;

    Ok(command)
}

#[instrument(skip(db), fields(command_id))]
pub async fn mark_processing<C>(db: &C, command_id: i32) -> Result<()>
where
    C: ConnectionTrait,
{
    let command = match get_command(db, command_id).await? {
        Some(command) => command,
        None => return Ok(()),
    };

    outgoing_commands::ActiveModel {
        id: Set(command_id),
        status: Set(status::PROCESSING.to_owned()),
        attempt_count: Set(command.attempt_count + 1),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(())
}

#[instrument(skip(db, ack_response), fields(command_id))]
pub async fn mark_success<C>(
    db: &C,
    command_id: i32,
    ack_response: Option<serde_json::Value>,
) -> Result<()>
where
    C: ConnectionTrait,
{
    if get_command(db, command_id).await?.is_none() {
        return Ok(());
    }

    outgoing_commands::ActiveModel {
        id: Set(command_id),
        status: Set(status::SUCCESS.to_owned()),
        processed_at: Set(Some(Utc::now())),
        ack_response: Set(ack_response),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(())
}

#[instrument(skip(db, error_message), fields(command_id))]
pub async fn mark_failed<C>(db: &C, command_id: i32, error_message: &str) -> Result<()>
where
    C: ConnectionTrait,
{
    if get_command(db, command_id).await?.is_none() {
        return Ok(());
    }

    outgoing_commands::ActiveModel {
        id: Set(command_id),
        status: Set(status::FAILED.to_owned()),
        processed_at: Set(Some(Utc::now())),
        error_message: Set(Some(error_message.to_owned())),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(())
}

#[instrument(skip(db), fields(command_id))]
pub async fn mark_timeout<C>(db: &C, command_id: i32) -> Result<()>
where
    C: ConnectionTrait,
{
    if get_command(db, command_id).await?.is_none() {
        return Ok(());
    }

    outgoing_commands::ActiveModel {
        id: Set(command_id),
        status: Set(status::TIMEOUT.to_owned()),
        processed_at: Set(Some(Utc::now())),
        error_message: Set(Some("Operation timed out".to_owned())),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(())
}

#[instrument(skip(db, error_message), fields(command_id))]
pub async fn retry_or_fail<C>(db: &C, command_id: i32, error_message: &str) -> Result<bool>
where
    C: ConnectionTrait,
{
    let command = match get_command(db, command_id).await? {
        Some(c) => c,
        None => return Ok(false),
    };

    let is_rate_limited_send =
        is_rate_limited_error(Some(error_message)) && command.event == "message:send";

    if is_rate_limited_send && command.max_attempts == STANDARD_MAX_ATTEMPTS {
        outgoing_commands::ActiveModel {
            id: Set(command_id),
            max_attempts: Set(MESSAGE_SEND_RATE_LIMIT_MAX_ATTEMPTS),
            ..Default::default()
        }
        .update(db)
        .await?;
    }

    let max_attempts = if is_rate_limited_send && command.max_attempts == STANDARD_MAX_ATTEMPTS {
        MESSAGE_SEND_RATE_LIMIT_MAX_ATTEMPTS
    } else {
        command.max_attempts
    };

    if command.attempt_count >= max_attempts {
        let msg = format!("Max attempts ({}) reached: {}", max_attempts, error_message);
        outgoing_commands::ActiveModel {
            id: Set(command_id),
            status: Set(status::FAILED.to_owned()),
            processed_at: Set(Some(Utc::now())),
            error_message: Set(Some(msg)),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(false)
    } else {
        outgoing_commands::ActiveModel {
            id: Set(command_id),
            status: Set(status::PENDING.to_owned()),
            error_message: Set(Some(error_message.to_owned())),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(true)
    }
}

#[instrument(skip(db), fields(command_id))]
pub async fn get_command_result<C>(db: &C, command_id: i32) -> Result<Option<OutgoingCommand>>
where
    C: ConnectionTrait,
{
    get_command(db, command_id).await
}

#[instrument(skip(db), fields(cutoff = %cutoff))]
pub async fn prune_processed_before<C>(db: &C, cutoff: chrono::DateTime<Utc>) -> Result<i64>
where
    C: ConnectionTrait,
{
    let result = outgoing_commands::Entity::delete_many()
        .filter(outgoing_commands::Column::Status.is_in(status::TERMINAL_STATUSES.iter().copied()))
        .filter(outgoing_commands::Column::ProcessedAt.is_not_null())
        .filter(outgoing_commands::Column::ProcessedAt.lte(cutoff))
        .exec(db)
        .await?;

    Ok(result.rows_affected as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_default_max_attempts {
        use super::*;

        #[test]
        fn test_message_send_gets_rate_limit_max() {
            assert_eq!(default_max_attempts_for_event("message:send"), 6);
        }

        #[test]
        fn test_non_send_gets_standard_max() {
            assert_eq!(default_max_attempts_for_event("message:join-room"), 3);
        }

        #[test]
        fn test_arbitrary_event_gets_standard_max() {
            assert_eq!(default_max_attempts_for_event("test:event"), 3);
        }
    }

    mod test_is_rate_limited_error {
        use super::*;

        #[test]
        fn test_none_is_not_rate_limited() {
            assert!(!is_rate_limited_error(None));
        }

        #[test]
        fn test_chinese_rate_limit_phrase() {
            assert!(is_rate_limited_error(Some("发送消息过于频繁")));
        }

        #[test]
        fn test_chinese_short_phrase() {
            assert!(is_rate_limited_error(Some("过于频繁")));
        }

        #[test]
        fn test_english_rate_limit() {
            assert!(is_rate_limited_error(Some("rate limit exceeded")));
        }

        #[test]
        fn test_http_429() {
            assert!(is_rate_limited_error(Some("HTTP 429")));
        }

        #[test]
        fn test_case_insensitive() {
            assert!(is_rate_limited_error(Some("RATE LIMIT")));
        }

        #[test]
        fn test_socket_failure_not_rate_limited() {
            assert!(!is_rate_limited_error(Some("temporary socket failure")));
        }

        #[test]
        fn test_connection_refused_not_rate_limited() {
            assert!(!is_rate_limited_error(Some("connection refused")));
        }
    }

    mod test_is_rate_limited_message_send {
        use super::*;
        use chrono::Utc;

        fn make_command(event: &str, error: Option<&str>) -> OutgoingCommand {
            OutgoingCommand {
                id: 1,
                created_at: Utc::now(),
                account_user_id: "user1".into(),
                event: event.into(),
                data: serde_json::json!({}),
                require_ack: true,
                status: status::PENDING.into(),
                processed_at: None,
                ack_response: None,
                error_message: error.map(String::from),
                attempt_count: 1,
                max_attempts: 6,
            }
        }

        #[test]
        fn test_message_send_with_rate_limit_error_is_identified() {
            let cmd = make_command("message:send", Some("rate limit"));
            assert!(is_rate_limited_message_send(&cmd));
        }

        #[test]
        fn test_message_send_without_error_is_not_identified() {
            let cmd = make_command("message:send", None);
            assert!(!is_rate_limited_message_send(&cmd));
        }

        #[test]
        fn test_non_send_with_rate_limit_error_is_not_identified() {
            let cmd = make_command("message:join-room", Some("rate limit"));
            assert!(!is_rate_limited_message_send(&cmd));
        }

        #[test]
        fn test_non_send_without_error_is_not_identified() {
            let cmd = make_command("test:event", None);
            assert!(!is_rate_limited_message_send(&cmd));
        }
    }

    mod test_rate_limited_retry_not_before {
        use super::*;
        use chrono::Utc;

        fn make_command(attempt_count: i32, created_at: chrono::DateTime<Utc>) -> OutgoingCommand {
            OutgoingCommand {
                id: 1,
                created_at,
                account_user_id: "user1".into(),
                event: "message:send".into(),
                data: serde_json::json!({}),
                require_ack: true,
                status: status::PENDING.into(),
                processed_at: None,
                ack_response: None,
                error_message: Some("rate limit".into()),
                attempt_count,
                max_attempts: 6,
            }
        }

        #[test]
        fn test_attempt_1_has_5_second_delay() {
            let now = Utc::now();
            let cmd = make_command(1, now);
            let not_before = rate_limited_retry_not_before(&cmd);
            assert_eq!(not_before, now + Duration::seconds(5));
        }

        #[test]
        fn test_attempt_2_has_15_second_delay() {
            let now = Utc::now();
            let cmd = make_command(2, now);
            let not_before = rate_limited_retry_not_before(&cmd);
            assert_eq!(not_before, now + Duration::seconds(15));
        }

        #[test]
        fn test_attempt_3_has_30_second_delay() {
            let now = Utc::now();
            let cmd = make_command(3, now);
            let not_before = rate_limited_retry_not_before(&cmd);
            assert_eq!(not_before, now + Duration::seconds(30));
        }

        #[test]
        fn test_attempt_beyond_delays_capped_at_80() {
            let now = Utc::now();
            let cmd = make_command(10, now);
            let not_before = rate_limited_retry_not_before(&cmd);
            assert_eq!(not_before, now + Duration::seconds(80));
        }
    }

    mod test_is_ready_for_processing {
        use super::*;
        use chrono::Utc;

        fn make_command(
            status: &str,
            event: &str,
            error: Option<&str>,
            attempt_count: i32,
            created_at: chrono::DateTime<Utc>,
        ) -> OutgoingCommand {
            OutgoingCommand {
                id: 1,
                created_at,
                account_user_id: "user1".into(),
                event: event.into(),
                data: serde_json::json!({}),
                require_ack: true,
                status: status.into(),
                processed_at: None,
                ack_response: None,
                error_message: error.map(String::from),
                attempt_count,
                max_attempts: 6,
            }
        }

        #[test]
        fn test_pending_non_rate_limited_is_ready() {
            let now = Utc::now();
            let cmd = make_command(status::PENDING, "test:event", None, 0, now);
            assert!(is_ready_for_processing(&cmd, now));
        }

        #[test]
        fn test_non_pending_is_not_ready() {
            let now = Utc::now();
            let cmd = make_command(status::PROCESSING, "test:event", None, 0, now);
            assert!(!is_ready_for_processing(&cmd, now));
        }

        #[test]
        fn test_rate_limited_before_backoff_expires_is_not_ready() {
            let now = Utc::now();
            let cmd = make_command(status::PENDING, "message:send", Some("rate limit"), 1, now);
            assert!(!is_ready_for_processing(&cmd, now + Duration::seconds(2)));
        }

        #[test]
        fn test_rate_limited_after_backoff_expires_is_ready() {
            let now = Utc::now();
            let cmd = make_command(status::PENDING, "message:send", Some("rate limit"), 1, now);
            assert!(is_ready_for_processing(&cmd, now + Duration::seconds(10)));
        }
    }

    mod test_create_command {
        use super::*;

        #[tokio::test]
        async fn test_create_command_defaults() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let cmd = create_command(
                    tx,
                    "user1",
                    "message:send",
                    serde_json::json!({"room_id": "room1", "text": "hello"}),
                    true,
                    None,
                )
                .await
                .unwrap();
                assert_eq!(cmd.account_user_id, "user1");
                assert_eq!(cmd.event, "message:send");
                assert!(cmd.require_ack);
                assert_eq!(cmd.max_attempts, 6);
                assert_eq!(cmd.status, status::PENDING);
                assert_eq!(cmd.attempt_count, 0);
                Ok(())
            })
            .await
            .expect("test_create_command_defaults");
        }

        #[tokio::test]
        async fn test_create_command_non_message_send_uses_standard_retry_budget() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let cmd = create_command(
                    tx,
                    "user1",
                    "message:join-room",
                    serde_json::json!({}),
                    true,
                    None,
                )
                .await
                .unwrap();
                assert_eq!(cmd.max_attempts, 3);
                Ok(())
            })
            .await
            .expect("test_create_command_non_message_send_uses_standard_retry_budget");
        }

        #[tokio::test]
        async fn test_create_command_custom_params() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let cmd = create_command(
                    tx,
                    "user1",
                    "test:event",
                    serde_json::json!({}),
                    false,
                    Some(5),
                )
                .await
                .unwrap();
                assert!(!cmd.require_ack);
                assert_eq!(cmd.max_attempts, 5);
                Ok(())
            })
            .await
            .expect("test_create_command_custom_params");
        }

        #[tokio::test]
        async fn test_create_command_returns_with_id() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let cmd =
                    create_command(tx, "user1", "test:event", serde_json::json!({}), true, None)
                        .await
                        .unwrap();
                assert!(cmd.id > 0);
                Ok(())
            })
            .await
            .expect("test_create_command_returns_with_id");
        }

        #[tokio::test]
        async fn test_create_multiple_commands_increments_id() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let cmd1 = create_command(
                    tx,
                    "user1",
                    "event:a",
                    serde_json::json!({"n": 1}),
                    true,
                    None,
                )
                .await
                .unwrap();
                let cmd2 = create_command(
                    tx,
                    "user1",
                    "event:b",
                    serde_json::json!({"n": 2}),
                    true,
                    None,
                )
                .await
                .unwrap();
                let cmd3 = create_command(
                    tx,
                    "user1",
                    "event:c",
                    serde_json::json!({"n": 3}),
                    true,
                    None,
                )
                .await
                .unwrap();
                assert!(cmd1.id < cmd2.id);
                assert!(cmd2.id < cmd3.id);
                Ok(())
            })
            .await
            .expect("test_create_multiple_commands_increments_id");
        }
    }

    mod test_get_pending_commands {
        use super::*;

        #[tokio::test]
        async fn test_get_pending_commands_fifo_order() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let cmd1 = create_command(
                    tx,
                    "user1",
                    "event:a",
                    serde_json::json!({"n": 1}),
                    true,
                    None,
                )
                .await
                .unwrap();
                let cmd2 = create_command(
                    tx,
                    "user1",
                    "event:b",
                    serde_json::json!({"n": 2}),
                    true,
                    None,
                )
                .await
                .unwrap();
                let cmd3 = create_command(
                    tx,
                    "user1",
                    "event:c",
                    serde_json::json!({"n": 3}),
                    true,
                    None,
                )
                .await
                .unwrap();
                let pending = get_pending_commands(tx, "user1", 10).await.unwrap();
                assert_eq!(pending.len(), 3);
                assert_eq!(pending[0].id, cmd1.id);
                assert_eq!(pending[1].id, cmd2.id);
                assert_eq!(pending[2].id, cmd3.id);
                Ok(())
            })
            .await
            .expect("test_get_pending_commands_fifo_order");
        }

        #[tokio::test]
        async fn test_get_pending_commands_filters_by_account() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                create_command(tx, "user1", "event:a", serde_json::json!({}), true, None)
                    .await
                    .unwrap();
                create_command(tx, "user2", "event:b", serde_json::json!({}), true, None)
                    .await
                    .unwrap();
                create_command(tx, "user1", "event:c", serde_json::json!({}), true, None)
                    .await
                    .unwrap();
                let pending_user1 = get_pending_commands(tx, "user1", 10).await.unwrap();
                assert_eq!(pending_user1.len(), 2);
                assert!(pending_user1.iter().all(|c| c.account_user_id == "user1"));
                Ok(())
            })
            .await
            .expect("test_get_pending_commands_filters_by_account");
        }

        #[tokio::test]
        async fn test_get_pending_commands_respects_limit() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                for i in 0..5 {
                    create_command(
                        tx,
                        "user1",
                        &format!("event:{}", i),
                        serde_json::json!({}),
                        true,
                        None,
                    )
                    .await
                    .unwrap();
                }
                let pending = get_pending_commands(tx, "user1", 3).await.unwrap();
                assert_eq!(pending.len(), 3);
                Ok(())
            })
            .await
            .expect("test_get_pending_commands_respects_limit");
        }

        #[tokio::test]
        async fn test_get_pending_commands_empty() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let pending = get_pending_commands(tx, "user_nonexistent", 10)
                    .await
                    .unwrap();
                assert!(pending.is_empty());
                Ok(())
            })
            .await
            .expect("test_get_pending_commands_empty");
        }
    }

    mod test_get_command {
        use super::*;

        #[tokio::test]
        async fn test_get_command_missing() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let result = get_command(tx, 99999).await.unwrap();
                assert!(result.is_none());
                Ok(())
            })
            .await
            .expect("test_get_command_missing");
        }
    }

    mod test_mark_processing {
        use super::*;

        #[tokio::test]
        async fn test_mark_processing_nonexistent_command() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                mark_processing(tx, 99999).await.unwrap();
                Ok(())
            })
            .await
            .expect("test_mark_processing_nonexistent_command");
        }
    }

    mod test_mark_success {
        use super::*;

        #[tokio::test]
        async fn test_mark_success_nonexistent_command() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                mark_success(tx, 99999, None).await.unwrap();
                Ok(())
            })
            .await
            .expect("test_mark_success_nonexistent_command");
        }
    }

    mod test_mark_failed {
        use super::*;

        #[tokio::test]
        async fn test_mark_failed_nonexistent_command() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                mark_failed(tx, 99999, "some error").await.unwrap();
                Ok(())
            })
            .await
            .expect("test_mark_failed_nonexistent_command");
        }
    }

    mod test_mark_timeout {
        use super::*;

        #[tokio::test]
        async fn test_mark_timeout_nonexistent_command() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                mark_timeout(tx, 99999).await.unwrap();
                Ok(())
            })
            .await
            .expect("test_mark_timeout_nonexistent_command");
        }
    }

    mod test_retry_or_fail {
        use super::*;

        #[tokio::test]
        async fn test_retry_or_fail_nonexistent_command_returns_false() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let result = retry_or_fail(tx, 99999, "some error").await.unwrap();
                assert!(!result);
                Ok(())
            })
            .await
            .expect("test_retry_or_fail_nonexistent_command_returns_false");
        }
    }

    mod test_get_command_result {
        use super::*;

        #[tokio::test]
        async fn test_get_command_result_missing() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let result = get_command_result(tx, 99999).await.unwrap();
                assert!(result.is_none());
                Ok(())
            })
            .await
            .expect("test_get_command_result_missing");
        }
    }

    mod test_prune_processed_commands {
        use super::*;

        #[tokio::test]
        async fn test_prune_processed_before_returns_count() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::OutgoingCommand,
            )
            .await
            .expect("init outgoing command db");

            lilium_database::transaction!(test_db.database(), |tx| {
                let result = prune_processed_before(tx, Utc::now()).await.unwrap();
                assert_eq!(result, 0);
                Ok(())
            })
            .await
            .expect("test_prune_processed_before_returns_count");
        }
    }
}
