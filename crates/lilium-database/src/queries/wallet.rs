use anyhow::Result;
use lilium_models::wallet::{Wallet, WalletLedgerPosition, WalletTransaction};
use rust_decimal::Decimal;
use sqlx::{Executor, Postgres};

pub async fn get_or_create_wallet<'e, E>(exec: &mut E, user_id: &str) -> Result<Wallet>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
    sqlx::query(
        r#"INSERT INTO wallet (user_id, allow_negative_balance, snapshot_balance,
               snapshot_escrow_balance, snapshot_tx_id, total_credited, created_at)
           VALUES ($1, false, 0, 0, 0, 0, NOW())
           ON CONFLICT (user_id) DO NOTHING"#,
    )
    .bind(user_id)
    .execute(&mut *exec)
    .await?;

    let wallet = sqlx::query_as::<_, Wallet>(
        "SELECT user_id, allow_negative_balance, snapshot_balance, snapshot_escrow_balance,
                snapshot_tx_id, last_daily_credit, total_credited, created_at
         FROM wallet WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&mut *exec)
    .await?;
    Ok(wallet)
}

pub async fn get_ledger_position<'e, E>(exec: &mut E, user_id: &str) -> Result<WalletLedgerPosition>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
    let wallet = get_or_create_wallet(exec, user_id).await?;

    let (tail_amount, tail_escrow): (Option<Decimal>, Option<Decimal>) =
        if wallet.snapshot_tx_id > 0 {
            sqlx::query_as(
                r#"SELECT COALESCE(SUM(amount), 0), COALESCE(SUM(escrow_delta), 0)
               FROM wallet_transaction
               WHERE user_id = $1 AND id > $2"#,
            )
            .bind(user_id)
            .bind(wallet.snapshot_tx_id)
            .fetch_one(&mut *exec)
            .await?
        } else {
            sqlx::query_as(
                r#"SELECT COALESCE(SUM(amount), 0), COALESCE(SUM(escrow_delta), 0)
               FROM wallet_transaction WHERE user_id = $1"#,
            )
            .bind(user_id)
            .fetch_one(&mut *exec)
            .await?
        };

    Ok(WalletLedgerPosition {
        balance: wallet.snapshot_balance + tail_amount.unwrap_or(Decimal::ZERO),
        escrow_balance: wallet.snapshot_escrow_balance + tail_escrow.unwrap_or(Decimal::ZERO),
        snapshot_tx_id: wallet.snapshot_tx_id,
    })
}

pub async fn get_balance<'e, E>(exec: &mut E, user_id: &str) -> Result<Decimal>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
    let pos = get_ledger_position(exec, user_id).await?;
    Ok(pos.balance)
}

pub async fn log_transaction<'e, E>(
    exec: &mut E,
    user_id: &str,
    amount: Decimal,
    escrow_delta: Decimal,
    tx_type: &str,
    description: &str,
    counterparty_id: &str,
    tx_group_id: &str,
) -> Result<WalletTransaction>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
    let tx = sqlx::query_as::<_, WalletTransaction>(
        r#"INSERT INTO wallet_transaction
           (user_id, amount, escrow_delta, tx_type, description, counterparty_id, tx_group_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, user_id, amount, escrow_delta, balance_after,
                     tx_type, description, reference_id, memo,
                     counterparty_id, tx_group_id, principal_id,
                     metadata, escrow_after, created_at"#,
    )
    .bind(user_id)
    .bind(amount)
    .bind(escrow_delta)
    .bind(tx_type)
    .bind(description)
    .bind(counterparty_id)
    .bind(tx_group_id)
    .fetch_one(&mut *exec)
    .await?;
    Ok(tx)
}
