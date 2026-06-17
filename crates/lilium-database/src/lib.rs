// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 database/__init__.py

//! Database infrastructure for Lilium.
//!
//! Application code should use [`Database::orm`] and SeaORM's
//! [`sea_orm::ConnectionTrait`] for ordinary service-layer CRUD. Dedicated
//! PostgreSQL connection state such as `LISTEN`/`NOTIFY` and session-level
//! advisory locks is exposed through specialized types instead of the normal
//! ORM pool.

pub mod database;
mod observability;
mod pool;
pub mod transaction;

pub use database::{
    Database, DatabaseConfig, DedicatedDatabaseConfig, DedicatedDbConnection,
    NotificationConnection, NotificationDatabaseConfig, RawDbConnection,
};
/// Transaction handle passed into service functions that need one atomic
/// database operation.
///
/// Prefer accepting `&impl sea_orm::ConnectionTrait` in services so the same
/// function can run on either a transaction or the shared ORM connection.
pub type DbTransaction = sea_orm::DatabaseTransaction;
pub use transaction::TransactionFuture;
