# Database Layer Spec

## Final Architecture

The Rust rewrite uses a `Database` runtime as the production database entry
point. Binary configuration loads `DATABASE_URL` and `DATABASE_POOL_SIZE`, then
constructs `DatabaseConfig { url, max_connections }` and calls
`Database::create`.

`Database` owns one shared SQLx `PgPool`. SeaORM is constructed from that same
pool with `SqlxPostgresConnector::from_sqlx_postgres_pool`, so ORM and raw SQL
share a single connection budget.

Production API:

```rust
pub struct Database;
pub struct DbSession;
pub struct RawDbConnection;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Database {
    pub async fn create(config: DatabaseConfig) -> Result<Self>;
    pub fn orm(&self) -> &sea_orm::DatabaseConnection;
    pub async fn transaction<T, F>(&self, f: F) -> Result<T>;
    pub async fn raw_connection(&self) -> Result<RawDbConnection>;
}
```

Normal application work uses `transaction!(db, |session| { ... }).await`.
Rollback-only APIs are not part of production. `raw_connection` is the explicit
escape hatch for PostgreSQL protocol features such as `LISTEN/NOTIFY` and
maintenance SQL.

## Service Layer

Service structs do not exist only to hold database sessions. Session-only
services were converted to module functions that accept `&mut DbSession`
explicitly:

- account functions
- event and processor offset functions
- message functions
- outgoing command functions
- room member functions
- user functions
- websocket connection functions

Allowed service structs hold runtime dependencies, not execution state.
`MediaService` owns HTTP/data-path dependencies and accepts `&mut DbSession` for
database work. `NotificationService` owns subscriber state and no longer accepts
a database session.

## Raw SQL And ORM

SeaORM is present as the ORM layer. The project uses an ORM-first hybrid model:
ordinary entity work should move to entities/models, while PostgreSQL-specific
queries remain explicit raw SQL near the service/use case that owns the behavior.

The old `crates/lilium-database/src/queries` module has been deleted. Message
raw SQL helpers that still matter now live in `crates/lilium-services/src/message.rs`.

## Test Fixtures

Database-backed tests use guard-style fixtures:

```rust
let test_db = TestDb::acquire(FixtureProfile::Message).await?;
lilium_database::transaction!(test_db.database(), |session| {
    service_call(session).await
})
.await?;
```

Callback helpers `with_db_session` and `with_db_session_and_pool` were removed.
The fixture pool leases one database per active test unit, resets and seeds
before handing out a lease, returns the lease synchronously on drop, and resets
again on the next acquire.

## Current Compatibility Boundary

`lilium-database::pool::DbPool` remains as low-level infrastructure used by
`Database` and `lilium-test-fixtures`. It is not re-exported from the
`lilium_database` root and is not used by production binaries or services.

`DbSessionContext`, callback session helpers, lazy/env pool constructors, and
`queries` have been removed.

## Verification Commands

Use these checks after database-layer changes:

```bash
cargo check --workspace --all-targets
cargo test -p lilium-test-fixtures
cargo test -p lilium-services
cargo test -p lilium-event-processor
rg -n "DbSessionContext|with_db_session|with_session_context|lilium_database::queries|pub mod queries" crates binaries -g '*.rs'
```
