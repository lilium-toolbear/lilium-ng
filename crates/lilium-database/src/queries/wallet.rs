use sqlx::PgPool;
use rust_decimal::Decimal;
use lilium_models::wallet::{Wallet, WalletLedgerPosition, WalletTransaction};
use anyhow::Result;

pub async fn get_or_create_wallet(pool: &PgPool, user_id: &str) -> Result<Wallet> {
    sqlx::query(
        r#"INSERT INTO wallet (user_id) VALUES ($1)
           ON CONFLICT (user_id) DO NOTHING"#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    let wallet = sqlx::query_as::<_, Wallet>(
        "SELECT * FROM wallet WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(wallet)
}

pub async fn get_ledger_position(pool: &PgPool, user_id: &str) -> Result<WalletLedgerPosition> {
    let wallet = get_or_create_wallet(pool, user_id).await?;

    let (tail_amount, tail_escrow): (Option<Decimal>, Option<Decimal>) = if wallet.snapshot_tx_id > 0 {
        sqlx::query_as(
            r#"SELECT COALESCE(SUM(amount), 0), COALESCE(SUM(escrow_delta), 0)
               FROM wallet_transaction
               WHERE user_id = $1 AND id > $2"#,
        )
        .bind(user_id)
        .bind(wallet.snapshot_tx_id)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_as(
            r#"SELECT COALESCE(SUM(amount), 0), COALESCE(SUM(escrow_delta), 0)
               FROM wallet_transaction WHERE user_id = $1"#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?
    };

    Ok(WalletLedgerPosition {
        balance: wallet.snapshot_balance + tail_amount.unwrap_or(Decimal::ZERO),
        escrow_balance: wallet.snapshot_escrow_balance + tail_escrow.unwrap_or(Decimal::ZERO),
        snapshot_tx_id: wallet.snapshot_tx_id,
    })
}

pub async fn get_balance(pool: &PgPool, user_id: &str) -> Result<Decimal> {
    let pos = get_ledger_position(pool, user_id).await?;
    Ok(pos.balance)
}

pub async fn log_transaction(
    pool: &PgPool,
    user_id: &str,
    amount: Decimal,
    escrow_delta: Decimal,
    tx_type: &str,
    description: &str,
    counterparty_id: &str,
    tx_group_id: &str,
) -> Result<WalletTransaction> {
    let tx = sqlx::query_as::<_, WalletTransaction>(
        r#"INSERT INTO wallet_transaction
           (user_id, amount, escrow_delta, tx_type, description, counterparty_id, tx_group_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING *"#,
    )
    .bind(user_id)
    .bind(amount)
    .bind(escrow_delta)
    .bind(tx_type)
    .bind(description)
    .bind(counterparty_id)
    .bind(tx_group_id)
    .fetch_one(pool)
    .await?;
    Ok(tx)
}
