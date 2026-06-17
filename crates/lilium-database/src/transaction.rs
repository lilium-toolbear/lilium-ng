// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 database/async_engine.py

use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

/// Boxed future returned by transaction closures.
///
/// Use this type through [`crate::transaction!`] or [`crate::Database::transaction`]
/// when a closure must borrow a SeaORM transaction across `await` points.
pub type TransactionFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

/// Run an async block inside a [`crate::Database`] transaction.
///
/// Prefer this macro at call sites because it hides the boxed-future plumbing
/// required by async transaction closures:
///
/// ```ignore
/// lilium_database::transaction!(database, |tx| {
///     service_call(tx).await?;
///     Ok(())
/// })
/// .await?;
/// ```
#[macro_export]
macro_rules! transaction {
    ($database:expr, |$tx:pat_param| $body:block $(,)?) => {{
        $database.transaction(|$tx| Box::pin(async move $body))
    }};
}

#[cfg(test)]
mod tests {
    use super::TransactionFuture;

    fn accepts_future<'a, T>(_future: TransactionFuture<'a, T>) {}

    #[test]
    fn transaction_future_type_is_nameable() {
        let future: TransactionFuture<'static, ()> = Box::pin(async { Ok(()) });
        accepts_future(future);
    }
}
