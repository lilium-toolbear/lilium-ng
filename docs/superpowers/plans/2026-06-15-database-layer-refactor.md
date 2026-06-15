# Database Layer Refactor Implementation Record

## Objective

Replace the SQLx-pool/session-wrapper architecture with a `Database` runtime,
SeaORM-backed ORM access, explicit raw SQL escape hatches, guard-based database
fixtures, and stateless service functions.

## Implemented

- Added SeaORM dependency and `crates/lilium-database/src/entities`.
- Added `Database`, `DatabaseConfig`, `RawDbConnection`, and `transaction!`.
- Built SeaORM and raw SQL access from one shared SQLx `PgPool`.
- Switched binaries to construct `Database` from env-loaded binary config.
- Migrated spider and event-processor transaction call sites to `transaction!`.
- Converted session-wrapper services to stateless functions:
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
cargo test -p lilium-services
cargo test -p lilium-event-processor
```

These searches have no Rust-code matches:

```bash
rg -n "DbSessionContext|with_db_session|with_db_session_and_pool|with_session_context|with_rollback_session_context|lilium_database::queries|pub mod queries" crates binaries -g '*.rs'
```

## Current Boundaries

- `Database::transaction` remains SQLx-backed through `DbSession`.
- SeaORM entity coverage exists as infrastructure; service CRUD has not been
  mechanically rewritten to SeaORM entities in this implementation pass.
- Raw SQL remains in services where it owns PostgreSQL-specific behavior or
  where ORM migration is still pending.
- `DbPool` is private low-level infrastructure, not the production application
  entry point.
