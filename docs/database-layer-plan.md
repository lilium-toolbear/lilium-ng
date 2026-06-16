# Database Layer Spec

## Final Architecture

The Rust rewrite uses a `Database` runtime as the production database entry
point. Binary configuration loads `DATABASE_URL` and `DATABASE_POOL_SIZE`, then
constructs `DatabaseConfig { url, max_connections }` and calls
`Database::create`.

`Database` owns one shared SQLx `PgPool`. SeaORM is constructed from that same
pool with `SqlxPostgresConnector::from_sqlx_postgres_pool`, so ORM and raw SQL
share a single connection budget.

Table-shaped models have one source of truth in `lilium-models`. A database row
model is the same Rust type the service layer passes around for that table.
SeaORM metadata lives on the existing model modules such as
`lilium_models::dzmm::message`, `lilium_models::dzmm::user`, and
`lilium_models::ingestion::websocket_event`. `lilium-database` owns runtime,
transaction, and connection infrastructure only.

Production API:

```rust
pub struct Database;
pub struct RawDbConnection;
pub struct NotificationConnection;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct NotificationDatabaseConfig {
    pub url: String,
}

impl Database {
    pub async fn create(config: DatabaseConfig) -> Result<Self>;
    pub fn orm(&self) -> &sea_orm::DatabaseConnection;
    pub async fn transaction<T, F>(&self, f: F) -> Result<T>;
    pub async fn raw_connection(&self) -> Result<RawDbConnection>;
}

impl NotificationConnection {
    pub async fn connect(config: NotificationDatabaseConfig) -> Result<Self>;
}
```

Normal application work uses `transaction!(db, |tx| { ... }).await`.
Rollback-only APIs are not part of production. `raw_connection` is the explicit
escape hatch for short-lived maintenance SQL.

PostgreSQL `LISTEN/NOTIFY` uses `NotificationConnection`, a dedicated direct
`sqlx::postgres::PgListener` outside the ordinary application pool. Production
configuration loads `DATABASE_NOTIFICATION_URL` and falls back to
`DATABASE_URL`. Test configuration loads `TEST_DATABASE_NOTIFICATION_URL` and
falls back to `TEST_DATABASE_URL`. Notification listeners do not use SeaORM,
`Database::transaction`, `Database::raw_connection`, or shared `PgPool`
connections.

## Service Layer

Service structs do not exist only to hold database sessions. Session-only
services were converted to module functions. Functions that participate in the
transaction boundary accept a SeaORM connection context explicitly:

- account functions
- event and processor offset functions
- message functions
- outgoing command functions
- room member functions
- user mutations, search, and batch update internals
- websocket connection functions

User primary-key reads accept a SeaORM connection through `ConnectionTrait`.

Allowed service structs hold runtime dependencies, not execution state.
`MediaService` owns HTTP/data-path dependencies and accepts a `ConnectionTrait`
implementation for database work. `NotificationService` owns subscriber state
and no longer accepts a database session.

## Raw SQL And ORM

SeaORM is the ORM layer. The existing modules under `crates/lilium-models/src`
define table-shaped models and their SeaORM metadata in the same place. Each
table module uses SeaORM's required `Model` struct name and exports the business
type name as an alias, for example `pub type Message = Model`.

The project uses an ORM-first hybrid model. Ordinary user primary-key reads use
SeaORM entities in `lilium-services::user`. PostgreSQL-specific behavior remains
explicit raw SQL near the service/use case that owns the behavior:

- message full-text search and partition-aware queries
- websocket/event processor range scans
- advisory locks
- counters and conflict updates
- short-lived maintenance SQL behind `Database::raw_connection`
- notification listener connections behind `NotificationConnection`

The old `crates/lilium-database/src/queries` module has been deleted. Message
raw SQL helpers that still matter now live in `crates/lilium-services/src/message.rs`.

`lilium-database` does not define or re-export table models. Service code
imports table modules directly from `lilium_models::dzmm` and
`lilium_models::ingestion`, and uses local raw-query structs only for non-table
projections such as enriched search rows and aggregate stats.

## Test Fixtures

Database-backed tests use guard-style fixtures:

```rust
let test_db = TestDb::acquire(FixtureProfile::Message).await?;
lilium_database::transaction!(test_db.database(), |tx| {
    service_call(tx).await
})
.await?;
```

Callback helpers `with_db_session` and `with_db_session_and_pool` were removed.
The fixture pool leases one database per active test unit, resets and seeds
before handing out a lease, returns the lease synchronously on drop, and resets
again on the next acquire.

## Current Boundary

`DbPool` remains private low-level infrastructure inside `lilium-database`.
Production binaries, services, and test fixtures use `Database`,
`DatabaseTransaction`, `ConnectionTrait`, `RawDbConnection`, and `TestDb`.

`DbSessionContext`, callback session helpers, lazy/env pool constructors, and
`queries` have been removed.

SeaORM-backed service reads currently cover `user::get_by_id`,
`user::get_by_ids`, and `user::fetch_user_profile`. SQLx-backed service
functions remain where the function requires the current raw transaction
boundary or PostgreSQL-specific SQL.

## Verification Commands

Use these checks after database-layer changes:

```bash
cargo check --workspace --all-targets
cargo test -p lilium-test-fixtures
cargo test -p lilium-services
cargo test -p lilium-event-processor
rg -n "DbSessionContext|with_db_session|with_session_context|lilium_database::queries|pub mod queries" crates binaries -g '*.rs'
rg -n "pub mod entities|lilium_(database|models)::entities|lilium_database::entities" crates binaries -S
rg -n "pub struct Model|DeriveEntityModel" crates/lilium-database/src crates/lilium-models/src -S
rg -n "FromRow|sqlx" crates/lilium-models crates/lilium-database/Cargo.toml -S
```
