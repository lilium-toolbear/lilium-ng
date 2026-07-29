# Spider Migration Guide For Agents

This guide captures the practical lessons from migrating the websocket spider
runtime from Python to Rust. Use it together with
`docs/python-to-rust-migration-sop.md`. The SOP defines the required parity
workflow. This guide explains how to apply that workflow to spider-specific
runtime behavior.

The Python source remains the source of truth. Existing Rust code, progress
notes, commit messages, and this guide are evidence indexes. Before changing a
spider behavior, read the current Python source and record the Python commit
used for that scan.

## Scope

Treat spider migration as runtime parity, not file translation. The migrated
surface is the observable behavior of one account worker and the arbiter that
owns workers:

- configuration loading and environment variable names
- arbiter startup, worker spawning, worker rescan, and worker shutdown
- per-account websocket connection lifecycle
- PostgreSQL advisory-lock ownership and heartbeat updates
- Socket.IO event ingestion and durable event writing
- outgoing command LISTEN/NOTIFY plus polling fallback
- command execution, ACK handling, retry state, and reconnect commands
- Unix control socket protocol and socket-file lifecycle
- tracing, Sentry, error context, and shutdown logging

Do not count parity by matched filenames. Track parity by scenarios with input
state, action, output, side effects, failure mode, source files read, Rust files
touched, and verification commands.

## Source Files To Read First

For spider runtime work, start by reading the current versions of these Python
files:

- `spider/ws_runtime.py`
- `spider/ws_ingestion.py`
- `spider/ws_control.py`
- `services/notification_service.py`
- `services/outgoing_command_service.py`
- `services/websocket_connection_service.py`
- the model files for tables touched by the scenario
- the tests that exercise the selected behavior

Record the exact commit with:

```bash
git -C ../dzmm_archive rev-parse HEAD
```

Then read the Rust boundary that owns the same behavior:

- `binaries/lilium-spider/src/main.rs`
- `binaries/lilium-spider/src/config.rs`
- `binaries/lilium-spider/src/arbiter/mod.rs`
- `binaries/lilium-spider/src/worker/mod.rs`
- `binaries/lilium-spider/src/ingestion.rs`
- `binaries/lilium-spider/src/control.rs`
- `crates/lilium-api-client/src/websocket.rs`
- `crates/lilium-services/src/outgoing_command.rs`
- `crates/lilium-services/src/websocket_connection.rs`
- `crates/lilium-services/src/event.rs`
- `crates/lilium-database/src/database.rs`
- `crates/lilium-database/src/observability.rs`

Add a Rust comment next to migrated logic with every Python file and commit used
for the scan:

```rust
// Python parity source: dzmm_archive@<commit> spider/ws_runtime.py
```

When one Rust module covers multiple Python files, add one comment per Python
file. When Rust intentionally diverges, name the Python behavior and the reason.

## Migration Order

Migrate spider in vertical slices. Each slice starts at an executable boundary
and ends with a verified scenario in
`docs/python-to-rust-migration-progress.md`.

1. Define the runtime scenario.

   Use a concrete boundary such as worker startup, outgoing command processing,
   reconnect control, heartbeat, event ingestion, shutdown, arbiter rescan, or
   event processor wakeup. Write the scenario before editing Rust.

2. Map Python runtime ownership.

   Identify the Python object that owns the lifecycle. For example,
   `AccountWorker.run` owns worker tasks, `SocketRuntime.run` owns websocket
   connection state, `NotificationService.stream_with_polling` owns NOTIFY
   wakeups, and `EventWriter.drain_once` owns disk-first replay.

3. Map Rust ownership.

   Put runtime orchestration in the binary, pure calculations in
   `lilium-core`, API transport in `lilium-api-client`, database/session
   primitives in `lilium-database`, and domain state transitions in
   `lilium-services`. Do not hide PostgreSQL session-state behavior behind a
   normal pooled transaction.

4. Implement the smallest complete behavior.

   A complete spider slice includes the happy path, the main failure path, the
   shutdown path, observability fields, source comments, progress notes, and
   targeted verification.

5. Update durable docs in the same change set.

   Update `docs/python-to-rust-migration-progress.md` while the source evidence
   is fresh. Update `docs/database-layer-plan.md` when the slice changes
   connection ownership, connection lifetimes, notification rules, public DB
   APIs, or service boundaries.

## Hard Rules From The Spider Port

### Dedicated Connections Own Session State

PostgreSQL session state belongs to one physical connection. This affected two
spider surfaces:

- `LISTEN/NOTIFY` uses `NotificationConnection`, outside the shared
  application pool.
- websocket advisory locks use `DedicatedDbConnection`, held for the worker
  lifecycle.

Do not acquire advisory locks through a pooled transaction and then rely on the
lock later. A later heartbeat can run on a different connection and falsely
claim ownership. The heartbeat must verify the dedicated connection still owns
the lock and must update `websocket_connections.last_heartbeat` through that
same connection.

Production notification consumers load `DATABASE_NOTIFICATION_URL` and fall back
to `DATABASE_URL`. Test notification consumers load
`TEST_DATABASE_NOTIFICATION_URL` and fall back to `TEST_DATABASE_URL`.

### NOTIFY Is A Wakeup Signal

Python `NotificationService.stream_with_polling` treats NOTIFY as a wakeup
signal, then polls the database for the real data. Keep this contract in Rust:

- subscribe before the first poll
- run an initial poll after subscribing
- poll again after every NOTIFY
- keep a timed polling fallback, currently 30 seconds for outgoing commands
- fetch durable rows from the database rather than trusting the NOTIFY payload
- on LISTEN connection loss (broken pipe, server restart), reconnect the
  dedicated notification session and resume; do not tear down the worker
- treat poll/NOTIFY-triggered database errors as transient: log and continue,
  mirroring Python `stream_with_polling` / `NotificationListener` reconnect

This prevents missed work when a notification is lost, coalesced, delayed, or
received before a listener reaches the steady-state loop. It also keeps the
websocket + disk-spill path alive across database outages.

### Worker Runtime Is A Task Set

Python `AccountWorker.run` starts a writer task plus three background tasks:

- websocket runtime
- outgoing command listener
- worker control socket

Rust currently adds a heartbeat loop as its own task because advisory-lock
verification and heartbeat updates need dedicated-connection ownership. The
worker stops accepting events when any controlling task finishes, drains the
writer, releases the advisory lock, and unlinks only the socket it bound.

Keep shutdown as a first-class scenario. Verify the control-socket bind failure
path after the advisory lock is acquired, because that path must release the
lock before returning an error.

### Hot Reconnect Is A Swap

Programmatic reconnect is not a disconnect-then-connect sequence. Python creates
a new Socket.IO client, connects it, verifies it remains connected, installs it
as the active client, then closes the old client. Rust mirrors this through the
`SocketCommandExecutor` generation model.

The required behavior is:

- new connection succeeds before the active executor changes
- failed new connection leaves the old socket active
- reconnect command waits for a generation newer than the previous one
- success records `{"status": "reconnected"}`
- failure records `{"status": "failed"}`

### Outgoing Commands Preserve Queue Semantics

Commands are durable database rows. The websocket runtime only executes rows
that are ready for processing.

Required behavior:

- fetch by `account_user_id`
- filter `status = pending`
- order by ascending `id`
- cap the worker poll at 100 rows
- stop the ready list at the first pending row delayed by rate-limit backoff
- leave a command pending when no socket is connected
- mark processing before emitting to Socket.IO
- for commands requiring ACK, wait 10 seconds
- ACK `{"success": false}` triggers retry/fail with the server error
- ACK timeout marks the command `timeout`
- emit-without-ACK marks success with no ACK response

Do not move this state machine into the API client. The API client owns Socket.IO
transport. `lilium-services::outgoing_command` owns queue state transitions.
The worker owns the orchestration between both.

### Ingestion Is Loss-Aware

The event path is a bounded memory queue with disk spill fallback. The writer
replays disk before memory, then inserts a database batch. On insert failure,
memory events move to disk.

Required behavior:

- spill file records use `schema_version = 2`
- replayed events have source `disk_replay`
- legacy spill schemas fail clearly
- stop accepting sends new events to disk
- shutdown drains insertable work, then spills remaining memory queue
- control status reports queue depth, inserted count, accepted count, and
  spilled count

Do not replace the spill file with a best-effort in-memory retry loop. The disk
file is part of the runtime durability contract.

### Control Sockets Are Runtime API

Unix control sockets are observable API, not internal implementation detail.

Required behavior:

- arbiter socket path is `ws_arbiter.sock`
- worker socket path is `ws_worker_<account_user_id>.sock`
- `account_user_id` must be a canonical lowercase UUID string
- command and response payloads are newline-delimited JSON
- stale sockets are removed
- live sockets are refused
- shutdown unlinks only the socket inode bound by the current process

The Python control helper also uses adjacent lock files to avoid races between
processes. Preserve the live-socket refusal guarantee when touching Rust control
socket logic.

### Observability Is Part Of Parity

The spider binary initializes backend Sentry with service name `ws_arbiter` and
wraps the runtime in a root tracing span. Service-layer boundaries that perform
I/O should carry `#[instrument]` with compact fields.

Instrument:

- database calls and dedicated PostgreSQL sessions
- notification listeners
- Socket.IO connect, command execution, reconnect, and heartbeat paths
- filesystem spill reads and writes when needed for diagnosis
- worker startup, shutdown, and task failure boundaries

Skip large and sensitive values: database handles, clients, request bodies,
credentials, cookies, raw payloads, URLs, file contents, and callback closures.
Record IDs, counts, booleans, operation names, generation numbers, and durations.

## Testing Guidance

Write behavior tests, not migration-count tests. Avoid assertions that only
freeze constants, display text, or implementation labels. A useful spider test
proves a behavior that can break production.

Good targets:

- notification URL fallback and required env failures
- control socket refuses live listeners and removes stale sockets
- control payload validation and canonical UUID validation
- disk spill schema, replay order, and legacy schema failure
- event writer drains disk before memory
- outgoing command FIFO, retry, timeout, rate-limit backoff, and no-socket
  pending behavior
- advisory lock conflict, release, heartbeat update, and same-session ownership
- reconnect generation wait and old-socket preservation on failed reconnect

DB-backed tests must use `crates/lilium-test-fixtures`. Required test
infrastructure should fail clearly when unavailable. Do not gate required DB
tests with `#[ignore]` as a fixture switch. Do not add production test
backdoors, `#[cfg(test)]` behavior branches, magic test values, noop production
implementations, or test-only constructors in production modules.

Use the smallest meaningful command first:

```bash
cargo test -p lilium-spider --all-targets
cargo test -p lilium-services --all-targets
cargo test -p lilium-database --all-targets
```

Use broader checks when the slice crosses crate boundaries:

```bash
cargo fmt --all --check
cargo clippy -p lilium-database -p lilium-test-fixtures -p lilium-services -p lilium-api-client -p lilium-spider --all-targets --all-features
cargo test -p lilium-database -p lilium-test-fixtures -p lilium-services -p lilium-api-client -p lilium-spider --all-targets -- --include-ignored
```

Do not rerun unrelated commands without a new reason. Record the exact commands
that were run in the progress entry.

## Progress Entry Template

Add one scenario-based entry for each completed scan:

````markdown
## YYYY-MM-DD: Spider <Scenario Name>

Python commit: `dzmm_archive@<commit>`

Python sources read:

- `/Users/bearice/Working/github/dzmm_archive/spider/ws_runtime.py`

Rust files updated:

- `binaries/lilium-spider/src/worker/mod.rs`

Verified scenarios:

- <input state, action, side effect, failure mode>

Remaining gaps:

- <none, or a concrete unimplemented behavior with owner>

Verification commands:

```bash
<commands>
```
````

The entry is complete only when it names the behavior boundary, source files,
Rust files, scenarios, gaps, and commands.

## Agent Handoff Checklist

Before editing:

- identify the executable boundary and exact runtime scenario
- read current Python source and record the current Python commit
- read the owning Rust files
- write the behavior contract in scenario form
- decide the owning Rust layer

While editing:

- preserve external database, JSON, socket, environment, and observability shapes
- keep PostgreSQL session-state behavior on dedicated connections
- add Python parity source comments beside migrated logic
- update progress and architecture docs in the same change set

Before claiming completion:

- run targeted verification
- inspect failures for root cause before fixing
- avoid speculative fixes
- confirm no test backdoors were added
- confirm no stale progress notes contradict the new entry
- state remaining gaps explicitly

The spider port succeeded when agents stopped translating files and started
migrating runtime contracts. Keep that discipline for every future slice.
