use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

pub type TransactionFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

#[macro_export]
macro_rules! transaction {
    ($database:expr, |$session:pat_param| $body:block $(,)?) => {{
        $database.transaction(|$session| Box::pin(async move $body))
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
