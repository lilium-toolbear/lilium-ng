use crate::Result;
use chrono::{Duration, Utc};
use lilium_database::DbSessionContext;

use lilium_models::dzmm::outgoing_command::{status, OutgoingCommand};
use tracing::instrument;

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

pub struct OutgoingCommandService<'a> {
    session: DbSessionContext<'a>,
}

impl<'a> OutgoingCommandService<'a> {
    #[instrument(skip(session))]
    pub fn new(session: DbSessionContext<'a>) -> Self {
        Self { session }
    }

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
        command.event == "message:send"
            && Self::is_rate_limited_error(command.error_message.as_deref())
    }

    #[instrument(skip(command), fields(command_id = command.id))]
    pub(crate) fn rate_limited_retry_not_before(
        command: &OutgoingCommand,
    ) -> chrono::DateTime<Utc> {
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
        if !Self::is_rate_limited_message_send(command) {
            return true;
        }
        Self::rate_limited_retry_not_before(command) <= now
    }

    #[instrument(skip(self, data), fields(account_user_id = %account_user_id, event = %event, require_ack, has_max_attempts = max_attempts.is_some()))]
    pub async fn create_command(
        &mut self,
        account_user_id: &str,
        event: &str,
        data: serde_json::Value,
        require_ack: bool,
        max_attempts: Option<i32>,
    ) -> Result<OutgoingCommand> {
        let max_attempts =
            max_attempts.unwrap_or_else(|| Self::default_max_attempts_for_event(event));

        let command = sqlx::query_as::<_, OutgoingCommand>(
            r#"INSERT INTO outgoing_commands (account_user_id, event, data, require_ack, status, attempt_count, max_attempts, created_at)
             VALUES ($1, $2, $3, $4, $5, 0, $6, $7)
             RETURNING *"#,
        )
        .bind(account_user_id)
        .bind(event)
        .bind(data)
        .bind(require_ack)
        .bind(status::PENDING)
        .bind(max_attempts)
        .bind(Utc::now())
        .fetch_one(self.session.as_mut())
        .await?;

        Ok(command)
    }

    #[instrument(skip(self), fields(account_user_id = %account_user_id, limit))]
    pub async fn get_pending_commands(
        &mut self,
        account_user_id: &str,
        limit: i64,
    ) -> Result<Vec<OutgoingCommand>> {
        let rows = sqlx::query_as::<_, OutgoingCommand>(
            r#"SELECT * FROM outgoing_commands
             WHERE account_user_id = $1 AND status = $2
             ORDER BY id ASC
             LIMIT $3"#,
        )
        .bind(account_user_id)
        .bind(status::PENDING)
        .bind(limit)
        .fetch_all(self.session.as_mut())
        .await?;

        let now = Utc::now();
        let mut ready_commands = Vec::new();
        for command in rows {
            if !Self::is_ready_for_processing(&command, now) {
                break;
            }
            ready_commands.push(command);
        }
        Ok(ready_commands)
    }

    #[instrument(skip(self), fields(command_id))]
    pub async fn get_command(&mut self, command_id: i32) -> Result<Option<OutgoingCommand>> {
        let command =
            sqlx::query_as::<_, OutgoingCommand>("SELECT * FROM outgoing_commands WHERE id = $1")
                .bind(command_id)
                .fetch_optional(self.session.as_mut())
                .await?;

        Ok(command)
    }

    #[instrument(skip(self), fields(command_id))]
    pub async fn mark_processing(&mut self, command_id: i32) -> Result<()> {
        sqlx::query(
            "UPDATE outgoing_commands SET status = $1, attempt_count = attempt_count + 1 WHERE id = $2",
        )
        .bind(status::PROCESSING)
        .bind(command_id)
        .execute(self.session.as_mut())
        .await?;
        Ok(())
    }

    #[instrument(skip(self, ack_response), fields(command_id))]
    pub async fn mark_success(
        &mut self,
        command_id: i32,
        ack_response: Option<serde_json::Value>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE outgoing_commands SET status = $1, processed_at = $2, ack_response = $3 WHERE id = $4",
        )
        .bind(status::SUCCESS)
        .bind(Utc::now())
        .bind(ack_response)
        .bind(command_id)
        .execute(self.session.as_mut())
        .await?;
        Ok(())
    }

    #[instrument(skip(self, error_message), fields(command_id))]
    pub async fn mark_failed(&mut self, command_id: i32, error_message: &str) -> Result<()> {
        sqlx::query(
            "UPDATE outgoing_commands SET status = $1, processed_at = $2, error_message = $3 WHERE id = $4",
        )
        .bind(status::FAILED)
        .bind(Utc::now())
        .bind(error_message)
        .bind(command_id)
        .execute(self.session.as_mut())
        .await?;
        Ok(())
    }

    #[instrument(skip(self), fields(command_id))]
    pub async fn mark_timeout(&mut self, command_id: i32) -> Result<()> {
        sqlx::query(
            "UPDATE outgoing_commands SET status = $1, processed_at = $2, error_message = $3 WHERE id = $4",
        )
        .bind(status::TIMEOUT)
        .bind(Utc::now())
        .bind("Operation timed out")
        .bind(command_id)
        .execute(self.session.as_mut())
        .await?;
        Ok(())
    }

    #[instrument(skip(self, error_message), fields(command_id))]
    pub async fn retry_or_fail(&mut self, command_id: i32, error_message: &str) -> Result<bool> {
        let command = match self.get_command(command_id).await? {
            Some(c) => c,
            None => return Ok(false),
        };

        let is_rate_limited_send =
            Self::is_rate_limited_error(Some(error_message)) && command.event == "message:send";

        if is_rate_limited_send && command.max_attempts == STANDARD_MAX_ATTEMPTS {
            sqlx::query("UPDATE outgoing_commands SET max_attempts = $1 WHERE id = $2")
                .bind(MESSAGE_SEND_RATE_LIMIT_MAX_ATTEMPTS)
                .bind(command_id)
                .execute(self.session.as_mut())
                .await?;
        }

        let max_attempts = if is_rate_limited_send && command.max_attempts == STANDARD_MAX_ATTEMPTS
        {
            MESSAGE_SEND_RATE_LIMIT_MAX_ATTEMPTS
        } else {
            command.max_attempts
        };

        if command.attempt_count >= max_attempts {
            let msg = format!("Max attempts ({}) reached: {}", max_attempts, error_message);
            sqlx::query(
                "UPDATE outgoing_commands SET status = $1, processed_at = $2, error_message = $3 WHERE id = $4",
            )
            .bind(status::FAILED)
            .bind(Utc::now())
            .bind(&msg)
            .bind(command_id)
            .execute(self.session.as_mut())
            .await?;
            Ok(false)
        } else {
            sqlx::query(
                "UPDATE outgoing_commands SET status = $1, error_message = $2 WHERE id = $3",
            )
            .bind(status::PENDING)
            .bind(error_message)
            .bind(command_id)
            .execute(self.session.as_mut())
            .await?;
            Ok(true)
        }
    }

    #[instrument(skip(self), fields(command_id))]
    pub async fn get_command_result(&mut self, command_id: i32) -> Result<Option<OutgoingCommand>> {
        self.get_command(command_id).await
    }

    #[instrument(skip(self), fields(cutoff = %cutoff))]
    pub async fn prune_processed_before(&mut self, cutoff: chrono::DateTime<Utc>) -> Result<i64> {
        let result = sqlx::query(
            "DELETE FROM outgoing_commands WHERE status = ANY($1) AND processed_at IS NOT NULL AND processed_at <= $2",
        )
        .bind(status::TERMINAL_STATUSES)
        .bind(cutoff)
        .execute(self.session.as_mut())
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_default_max_attempts {
        use super::*;

        #[test]
        fn test_message_send_gets_rate_limit_max() {
            assert_eq!(
                OutgoingCommandService::default_max_attempts_for_event("message:send"),
                6
            );
        }

        #[test]
        fn test_non_send_gets_standard_max() {
            assert_eq!(
                OutgoingCommandService::default_max_attempts_for_event("message:join-room"),
                3
            );
        }

        #[test]
        fn test_arbitrary_event_gets_standard_max() {
            assert_eq!(
                OutgoingCommandService::default_max_attempts_for_event("test:event"),
                3
            );
        }
    }

    mod test_is_rate_limited_error {
        use super::*;

        #[test]
        fn test_none_is_not_rate_limited() {
            assert!(!OutgoingCommandService::is_rate_limited_error(None));
        }

        #[test]
        fn test_chinese_rate_limit_phrase() {
            assert!(OutgoingCommandService::is_rate_limited_error(Some(
                "发送消息过于频繁"
            )));
        }

        #[test]
        fn test_chinese_short_phrase() {
            assert!(OutgoingCommandService::is_rate_limited_error(Some(
                "过于频繁"
            )));
        }

        #[test]
        fn test_english_rate_limit() {
            assert!(OutgoingCommandService::is_rate_limited_error(Some(
                "rate limit exceeded"
            )));
        }

        #[test]
        fn test_http_429() {
            assert!(OutgoingCommandService::is_rate_limited_error(Some(
                "HTTP 429"
            )));
        }

        #[test]
        fn test_case_insensitive() {
            assert!(OutgoingCommandService::is_rate_limited_error(Some(
                "RATE LIMIT"
            )));
        }

        #[test]
        fn test_socket_failure_not_rate_limited() {
            assert!(!OutgoingCommandService::is_rate_limited_error(Some(
                "temporary socket failure"
            )));
        }

        #[test]
        fn test_connection_refused_not_rate_limited() {
            assert!(!OutgoingCommandService::is_rate_limited_error(Some(
                "connection refused"
            )));
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
            assert!(OutgoingCommandService::is_rate_limited_message_send(&cmd));
        }

        #[test]
        fn test_message_send_without_error_is_not_identified() {
            let cmd = make_command("message:send", None);
            assert!(!OutgoingCommandService::is_rate_limited_message_send(&cmd));
        }

        #[test]
        fn test_non_send_with_rate_limit_error_is_not_identified() {
            let cmd = make_command("message:join-room", Some("rate limit"));
            assert!(!OutgoingCommandService::is_rate_limited_message_send(&cmd));
        }

        #[test]
        fn test_non_send_without_error_is_not_identified() {
            let cmd = make_command("test:event", None);
            assert!(!OutgoingCommandService::is_rate_limited_message_send(&cmd));
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
            let not_before = OutgoingCommandService::rate_limited_retry_not_before(&cmd);
            assert_eq!(not_before, now + Duration::seconds(5));
        }

        #[test]
        fn test_attempt_2_has_15_second_delay() {
            let now = Utc::now();
            let cmd = make_command(2, now);
            let not_before = OutgoingCommandService::rate_limited_retry_not_before(&cmd);
            assert_eq!(not_before, now + Duration::seconds(15));
        }

        #[test]
        fn test_attempt_3_has_30_second_delay() {
            let now = Utc::now();
            let cmd = make_command(3, now);
            let not_before = OutgoingCommandService::rate_limited_retry_not_before(&cmd);
            assert_eq!(not_before, now + Duration::seconds(30));
        }

        #[test]
        fn test_attempt_beyond_delays_capped_at_80() {
            let now = Utc::now();
            let cmd = make_command(10, now);
            let not_before = OutgoingCommandService::rate_limited_retry_not_before(&cmd);
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
            assert!(OutgoingCommandService::is_ready_for_processing(&cmd, now));
        }

        #[test]
        fn test_non_pending_is_not_ready() {
            let now = Utc::now();
            let cmd = make_command(status::PROCESSING, "test:event", None, 0, now);
            assert!(!OutgoingCommandService::is_ready_for_processing(&cmd, now));
        }

        #[test]
        fn test_rate_limited_before_backoff_expires_is_not_ready() {
            let now = Utc::now();
            let cmd = make_command(status::PENDING, "message:send", Some("rate limit"), 1, now);
            assert!(!OutgoingCommandService::is_ready_for_processing(
                &cmd,
                now + Duration::seconds(2)
            ));
        }

        #[test]
        fn test_rate_limited_after_backoff_expires_is_ready() {
            let now = Utc::now();
            let cmd = make_command(status::PENDING, "message:send", Some("rate limit"), 1, now);
            assert!(OutgoingCommandService::is_ready_for_processing(
                &cmd,
                now + Duration::seconds(10)
            ));
        }
    }

    mod test_create_command {
        use super::*;

        #[tokio::test]
        async fn test_create_command_defaults() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let cmd = service
                            .create_command(
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
                },
            )
            .await
            .expect("test_create_command_defaults");
        }

        #[tokio::test]
        async fn test_create_command_non_message_send_uses_standard_retry_budget() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let cmd = service
                            .create_command(
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
                },
            )
            .await
            .expect("test_create_command_non_message_send_uses_standard_retry_budget");
        }

        #[tokio::test]
        async fn test_create_command_custom_params() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let cmd = service
                            .create_command(
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
                },
            )
            .await
            .expect("test_create_command_custom_params");
        }

        #[tokio::test]
        async fn test_create_command_returns_with_id() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let cmd = service
                            .create_command(
                                "user1",
                                "test:event",
                                serde_json::json!({}),
                                true,
                                None,
                            )
                            .await
                            .unwrap();
                        assert!(cmd.id > 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_create_command_returns_with_id");
        }

        #[tokio::test]
        async fn test_create_multiple_commands_increments_id() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let cmd1 = service
                            .create_command(
                                "user1",
                                "event:a",
                                serde_json::json!({"n": 1}),
                                true,
                                None,
                            )
                            .await
                            .unwrap();
                        let cmd2 = service
                            .create_command(
                                "user1",
                                "event:b",
                                serde_json::json!({"n": 2}),
                                true,
                                None,
                            )
                            .await
                            .unwrap();
                        let cmd3 = service
                            .create_command(
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
                },
            )
            .await
            .expect("test_create_multiple_commands_increments_id");
        }
    }

    mod test_get_pending_commands {
        use super::*;

        #[tokio::test]
        async fn test_get_pending_commands_fifo_order() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let cmd1 = service
                            .create_command(
                                "user1",
                                "event:a",
                                serde_json::json!({"n": 1}),
                                true,
                                None,
                            )
                            .await
                            .unwrap();
                        let cmd2 = service
                            .create_command(
                                "user1",
                                "event:b",
                                serde_json::json!({"n": 2}),
                                true,
                                None,
                            )
                            .await
                            .unwrap();
                        let cmd3 = service
                            .create_command(
                                "user1",
                                "event:c",
                                serde_json::json!({"n": 3}),
                                true,
                                None,
                            )
                            .await
                            .unwrap();
                        let pending = service.get_pending_commands("user1", 10).await.unwrap();
                        assert_eq!(pending.len(), 3);
                        assert_eq!(pending[0].id, cmd1.id);
                        assert_eq!(pending[1].id, cmd2.id);
                        assert_eq!(pending[2].id, cmd3.id);
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_get_pending_commands_fifo_order");
        }

        #[tokio::test]
        async fn test_get_pending_commands_filters_by_account() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        service
                            .create_command("user1", "event:a", serde_json::json!({}), true, None)
                            .await
                            .unwrap();
                        service
                            .create_command("user2", "event:b", serde_json::json!({}), true, None)
                            .await
                            .unwrap();
                        service
                            .create_command("user1", "event:c", serde_json::json!({}), true, None)
                            .await
                            .unwrap();
                        let pending_user1 =
                            service.get_pending_commands("user1", 10).await.unwrap();
                        assert_eq!(pending_user1.len(), 2);
                        assert!(pending_user1.iter().all(|c| c.account_user_id == "user1"));
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_get_pending_commands_filters_by_account");
        }

        #[tokio::test]
        async fn test_get_pending_commands_respects_limit() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        for i in 0..5 {
                            service
                                .create_command(
                                    "user1",
                                    &format!("event:{}", i),
                                    serde_json::json!({}),
                                    true,
                                    None,
                                )
                                .await
                                .unwrap();
                        }
                        let pending = service.get_pending_commands("user1", 3).await.unwrap();
                        assert_eq!(pending.len(), 3);
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_get_pending_commands_respects_limit");
        }

        #[tokio::test]
        async fn test_get_pending_commands_empty() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let pending = service
                            .get_pending_commands("user_nonexistent", 10)
                            .await
                            .unwrap();
                        assert!(pending.is_empty());
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_get_pending_commands_empty");
        }
    }

    mod test_get_command {
        use super::*;

        #[tokio::test]
        async fn test_get_command_missing() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let result = service.get_command(99999).await.unwrap();
                        assert!(result.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_get_command_missing");
        }
    }

    mod test_mark_processing {
        use super::*;

        #[tokio::test]
        async fn test_mark_processing_nonexistent_command() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        service.mark_processing(99999).await.unwrap();
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_mark_processing_nonexistent_command");
        }
    }

    mod test_mark_success {
        use super::*;

        #[tokio::test]
        async fn test_mark_success_nonexistent_command() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        service.mark_success(99999, None).await.unwrap();
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_mark_success_nonexistent_command");
        }
    }

    mod test_mark_failed {
        use super::*;

        #[tokio::test]
        async fn test_mark_failed_nonexistent_command() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        service.mark_failed(99999, "some error").await.unwrap();
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_mark_failed_nonexistent_command");
        }
    }

    mod test_mark_timeout {
        use super::*;

        #[tokio::test]
        async fn test_mark_timeout_nonexistent_command() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        service.mark_timeout(99999).await.unwrap();
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_mark_timeout_nonexistent_command");
        }
    }

    mod test_retry_or_fail {
        use super::*;

        #[tokio::test]
        async fn test_retry_or_fail_nonexistent_command_returns_false() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let result = service.retry_or_fail(99999, "some error").await.unwrap();
                        assert!(!result);
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_retry_or_fail_nonexistent_command_returns_false");
        }
    }

    mod test_get_command_result {
        use super::*;

        #[tokio::test]
        async fn test_get_command_result_missing() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let result = service.get_command_result(99999).await.unwrap();
                        assert!(result.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_get_command_result_missing");
        }
    }

    mod test_prune_processed_commands {
        use super::*;

        #[tokio::test]
        async fn test_prune_processed_before_returns_count() {
            lilium_test_fixtures::with_db_session(
                lilium_test_fixtures::TestServiceFixture::OutgoingCommand,
                |session| {
                    Box::pin(async move {
                        let mut service = OutgoingCommandService::new(session);
                        let result = service.prune_processed_before(Utc::now()).await.unwrap();
                        assert_eq!(result, 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("test_prune_processed_before_returns_count");
        }
    }
}
