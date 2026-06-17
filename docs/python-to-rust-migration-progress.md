# Python To Rust Migration Progress

This file is the current progress tracker for Python-to-Rust parity work. Keep
entries scenario-based and update it in the same change set as the code and
architecture changes.

Do not use this file as source-of-truth for behavior. The Python source code is
still the source of truth. Each entry records what was scanned, what Rust now
covers, and what remains to migrate.

## Update Rules

- Add or update an entry for every functionality or logic scan.
- Record the Python repository commit hash used for the scan.
- Record exact Python files read and Rust files changed.
- Record scenarios, side effects, failure modes, and verification commands.
- Move completed gaps into the verified scenario list when code and tests land.
- Delete obsolete one-off notes after their content is represented here.

## 2026-06-17: lilium-cli send-command (Phase 1 of CLI port)

Python commit: `dzmm_archive@0efb507c6126a2638d3d38aca4018a804431291e`

Python sources read:

- `/Users/bearice/Working/github/dzmm_archive/cli/send_command.py`
- `/Users/bearice/Working/github/dzmm_archive/cli/explore.py`
- `/Users/bearice/Working/github/dzmm_archive/cli/sync_members.py`
- `/Users/bearice/Working/github/dzmm_archive/cli/sync_rooms.py`
- `/Users/bearice/Working/github/dzmm_archive/cli/CLAUDE.md`
- `/Users/bearice/Working/github/dzmm_archive/core/explore.py`
- `/Users/bearice/Working/github/dzmm_archive/core/sync.py`
- `/Users/bearice/Working/github/dzmm_archive/core/history.py`
- `/Users/bearice/Working/github/dzmm_archive/services/room_service.py`
- `/Users/bearice/Working/github/dzmm_archive/services/room_member_service.py`
- `/Users/bearice/Working/github/dzmm_archive/services/account_service.py`
- `/Users/bearice/Working/github/dzmm_archive/services/outgoing_command_service.py`
- `/Users/bearice/Working/github/dzmm_archive/models/dzmm/room.py`
- `/Users/bearice/Working/github/dzmm_archive/models/dzmm/room_member.py`

Scope decision: only `send_command.py`'s dependencies already exist in Rust
(`outgoing_command` service, `NotificationConnection`, `DzmmApi` uploads). The
other three CLIs depend on unmigrated core modules (`core.explore`,
`core.sync`, `core.history`) plus a missing `RoomService` and six missing
explore-content entities. The user opted to migrate the core modules too, in
phases. This entry covers Phase 1; later phases will add their own entries.

Rust files added/updated:

- `binaries/lilium-cli/Cargo.toml` (new binary)
- `binaries/lilium-cli/src/main.rs`
- `binaries/lilium-cli/src/config.rs`
- `binaries/lilium-cli/src/send_command.rs`
- `Cargo.toml` (workspace members + `uuid` dep)
- `crates/lilium-services/src/media.rs` (`extract_audio_duration` made `pub` for
  `send-voice` duration detection, reusing the lofty-based helper instead of
  ffprobe/mutagen)

Verified scenarios:

- `lilium-cli send-command` exposes all 18 Python click subcommands: `send`,
  `status`, `join-room`, `heartbeat`, `reconnect`, `list-pending`,
  `send-message`, `send-reply`, `leave-room`, `start-match`, `cancel-match`,
  `fetch-match-limit`, `edit-message`, `recall-message`, `delete-message`,
  `mark-read`, `send-image`, `send-voice`.
- JSON payloads are preserved verbatim from Python (`message:send` text/image/
  voice/reply, `message:edit`/`recall`/`delete`/`read`, `match:*`,
  `system:reconnect`, `heartbeat` with millisecond timestamp).
- `send` creates a command via `outgoing_command::create_command` with
  `require_ack = !no_ack` and default max-attempts per event.
- `--no-wait` queues and returns immediately; `--wait` (default) waits for a
  terminal status.
- `wait_for_result` LISTENs on `outgoing_command_updated` (the
  `outgoing_command_updated_trigger` fires `AFTER UPDATE` with payload
  `{"id","account_user_id","status"}`), filters by command id + terminal
  status, and falls back to a final `get_command_result` fetch on timeout
  (parity with Python `poll_timeout`).
- `status` and `list-pending` read from the DB; `list-pending` lists all
  pending regardless of rate-limit readiness (matching Python, unlike the
  spider's readiness-filtered `get_pending_commands`).
- `send-image`/`send-voice` resolve the account via `account::get_account` +
  `create_auth_client`, upload via `DzmmApi::upload_chat_image` /
  `upload_voice_message`, then send the `message:send` payload. Voice duration
  is detected via `media::extract_audio_duration` (lofty).
- Exit codes: success 0, command failure / poll-timeout / not-found 1.

Verification commands:

```bash
cargo build -p lilium-cli
cargo test -p lilium-cli
./target/debug/lilium-cli --help
./target/debug/lilium-cli send-command --help
```

Remaining gaps (tracked as phases 2–8):

- Phase 2: `RoomService` (`crates/lilium-services/src/room.rs`) — not yet
  ported; blocks sync CLIs and history.
- Phase 3: `core::sync` (`RoomSyncer`, `MemberSyncer`) + `room_member`
  `batch_upsert_members`/`clear_room_members`.
- Phase 4: `sync-members` + `sync-rooms` CLIs (incl. poll mode + reconnect).
- Phase 5: `core::history` (`HistoryFetcher`).
- Phase 6: `core::explore` + six explore-content entities/services
  (`tweet`/`card`/`gallery`/`checkpoint`/`book`/`chapter`) + schema bootstrap
  tables.
- Phase 7: `explore` CLI.
- Phase 8: parity comments + this doc + memory updates.

## 2026-06-16: Spider Worker Runtime And Notifications

Python commit: `dzmm_archive@6a92a9914602d633ff6fa3f5908fa68d00c36fcd`

Python sources read:

- `/Users/bearice/Working/github/dzmm_archive/spider/ws_runtime.py`
- `/Users/bearice/Working/github/dzmm_archive/services/notification_service.py`
- `/Users/bearice/Working/github/dzmm_archive/services/outgoing_command_service.py`

Rust files updated:

- `binaries/lilium-spider/src/config.rs`
- `binaries/lilium-spider/src/arbiter/mod.rs`
- `binaries/lilium-spider/src/worker/mod.rs`
- `binaries/lilium-spider/src/control.rs`
- `binaries/lilium-spider/src/ingestion.rs`
- `crates/lilium-api-client/src/websocket.rs`
- `binaries/lilium-event-processor/src/config.rs`
- `binaries/lilium-event-processor/src/main.rs`
- `binaries/lilium-event-processor/src/processor.rs`
- `crates/lilium-database/src/database.rs`
- `crates/lilium-services/src/websocket_connection.rs`
- `crates/lilium-test-fixtures/src/database.rs`
- `crates/lilium-test-fixtures/src/profile.rs`

Verified scenarios:

- Spider loads `DATABASE_NOTIFICATION_URL` and falls back to `DATABASE_URL` for
  a dedicated notification listener connection.
- Event processor loads the same notification URL contract and listens for
  `websocket_event_inserted` outside the normal application pool.
- Worker starts four runtime tasks matching Python `AccountWorker.run`: event
  writer, websocket runtime, outgoing command listener, and worker control
  socket.
- Worker listens to `outgoing_command_inserted`, runs an initial pending-command
  poll, and keeps a 30 second polling fallback.
- Pending outgoing commands are fetched by `account_user_id`, ordered by id,
  capped at 100, and processed only when ready according to service rules.
- Socket commands use the current live Socket.IO client. When no socket is
  connected, the command remains pending instead of being marked failed.
- Commands requiring ACK wait 10 seconds, mark timeout on ACK timeout, mark
  success on successful ACK, and retry/fail when ACK contains
  `{"success": false}`.
- Commands without ACK emit the Socket.IO event and mark success with no ACK
  response.
- `system:reconnect` marks processing, requests reconnect, waits for a newer
  connection generation, and writes `{"status": "reconnected"}` or
  `{"status": "failed"}`.
- Worker control socket uses `ws_worker_<account_user_id>.sock` and supports
  `status`, `reconnect`, and `stop`.
- Worker acquires a PostgreSQL advisory lock on one dedicated physical
  connection before running websocket tasks.
- Worker releases the same advisory lock and deletes the
  `websocket_connections` row during shutdown, including the control-socket
  bind failure path after lock acquisition.
- Worker emits Socket.IO `heartbeat` periodically while the socket is connected,
  verifies the dedicated connection still owns the advisory lock, and updates
  `websocket_connections.last_heartbeat` on the same connection.
- Programmatic reconnect performs a hot-swap: connect a new Socket.IO client,
  switch the command executor to the new client, then disconnect the old client.
  If the new connection fails, the old socket remains active.

Remaining gaps:

- No remaining gap is currently recorded for the spider worker
  advisory-lock, heartbeat, outgoing-command, or worker-control slice. Future
  scans must open a new entry when they identify another Python behavior
  boundary.

Verification commands:

```bash
cargo fmt --all --check
cargo clippy -p lilium-database -p lilium-test-fixtures -p lilium-services -p lilium-api-client -p lilium-spider --all-targets --all-features
cargo test -p lilium-database -p lilium-test-fixtures -p lilium-services -p lilium-api-client -p lilium-spider --all-targets -- --include-ignored
```
