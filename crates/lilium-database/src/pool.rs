use sqlx::PgPool;

#[derive(Debug, Clone)]
/// Internal wrapper around the SQLx Postgres pool used by [`crate::Database`].
///
/// This type exists so the database crate can keep raw SQLx pool access behind
/// an explicit infrastructure boundary. Service code should use
/// [`crate::Database::orm`] and SeaORM instead of depending on this pool.
pub struct DbPool {
    inner: PgPool,
}

/// Normalize Postgres URLs for SQLx/SeaORM connection constructors.
///
/// The Python side and deployment configuration may use `postgresql://`; SQLx
/// accepts `postgres://` consistently, so the database layer normalizes it once.
pub(crate) fn normalize_database_url(url: &str) -> String {
    let url = url.trim();
    if let Some(rest) = url.strip_prefix("postgresql://") {
        format!("postgres://{rest}")
    } else {
        url.to_string()
    }
}

impl DbPool {
    /// Wrap an existing SQLx Postgres pool for internal database runtime use.
    pub(crate) fn from_pg_pool(inner: PgPool) -> Self {
        Self { inner }
    }

    /// Return the underlying SQLx pool for infrastructure-only helpers.
    ///
    /// Do not expose this to services for ordinary table CRUD. Prefer
    /// `ConnectionTrait` and SeaORM query builders.
    pub fn inner(&self) -> &PgPool {
        &self.inner
    }
}
