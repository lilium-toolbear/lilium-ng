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
