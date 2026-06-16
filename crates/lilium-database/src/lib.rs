//! Database infrastructure for Lilium.
//!
//! Application code should use [`Database::orm`] and SeaORM's
//! [`sea_orm::ConnectionTrait`] for ordinary service-layer CRUD. Dedicated
//! PostgreSQL connection state such as `LISTEN`/`NOTIFY` is exposed through
//! specialized types instead of the normal ORM pool.

pub mod database;
mod pool;
pub mod transaction;

pub use database::{
    Database, DatabaseConfig, NotificationConnection, NotificationDatabaseConfig, RawDbConnection,
};
/// Transaction handle passed into service functions that need one atomic
/// database operation.
///
/// Prefer accepting `&impl sea_orm::ConnectionTrait` in services so the same
/// function can run on either a transaction or the shared ORM connection.
pub type DbTransaction = sea_orm::DatabaseTransaction;
pub use transaction::TransactionFuture;
