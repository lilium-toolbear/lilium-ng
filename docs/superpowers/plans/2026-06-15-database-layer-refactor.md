# Database Layer Refactor Implementation Record

## Objective

Replace the SQLx-pool/session-wrapper architecture with a `Database` runtime,
SeaORM-backed ORM access, explicit raw SQL escape hatches, guard-based database
fixtures, and stateless service functions.

## Implemented

- Added SeaORM dependency and live-schema entity modules in
  `crates/lilium-database/src/entities`.
- Added `Database`, `DatabaseConfig`, `RawDbConnection`, and `transaction!`.
- Built SeaORM and raw SQL access from one shared SQLx `PgPool`.
- Switched binaries to construct `Database` from env-loaded binary config.
- Migrated spider and event-processor transaction call sites to `transaction!`.
- Converted session-wrapper services to stateless functions. Raw transaction
  work accepts `&mut DbSession`; ORM-backed user reads accept a SeaORM
  connection:
  - account
  - event and processor offsets
  - message
  - outgoing command
  - room member
  - user
  - websocket connection
- Kept dependency-owning service structs only where they hold real runtime state:
  - `MediaService`
  - `NotificationService`
- Deleted `crates/lilium-database/src/queries`.
- Moved remaining message raw SQL helpers into `crates/lilium-services/src/message.rs`.
- Migrated ordinary user read operations to SeaORM:
  - `user::get_by_id`
  - `user::get_by_ids`
  - `user::fetch_user_profile`
- Kept user batch mutation internals on SQLx because they run inside the
  existing raw transaction boundary with related writes.
- Added guard-based `TestDb::acquire`.
- Migrated service and event-processor database tests off callback fixtures.
- Removed callback helpers `with_db_session` and `with_db_session_and_pool`.
- Removed `DbSessionContext`, context session helpers, boxed helper shims, and unused lazy/env pool constructors.
- Removed `DbPool` from the public API. It remains private infrastructure inside
  `lilium-database`.

## Verified

These commands passed after the implementation:

```bash
cargo check --workspace --all-targets
cargo test -p lilium-test-fixtures
cargo test -p lilium-services user_service_integration
cargo test -p lilium-event-processor
```

These searches have no Rust-code matches:

```bash
rg -n "DbSessionContext|with_db_session|with_db_session_and_pool|with_session_context|with_rollback_session_context|lilium_database::queries|pub mod queries" crates binaries -g '*.rs'
```

## Current Boundaries

- `Database::transaction` remains SQLx-backed through `DbSession`.
- SeaORM entity coverage now mirrors the live bootstrap schema for stable
  business columns.
- User primary-key read operations are SeaORM-backed service functions.
- Raw SQL remains in services for PostgreSQL-specific behavior, partition-aware
  message/event work, dynamic search SQL, advisory locks, counters, and
  multi-step mutations that share the existing raw transaction boundary.
- `DbPool` is private low-level infrastructure, not the production application
  entry point.
