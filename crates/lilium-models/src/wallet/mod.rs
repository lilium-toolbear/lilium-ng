use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Wallet {
    pub user_id: String,
    pub allow_negative_balance: bool,
    pub snapshot_balance: Decimal,
    pub snapshot_escrow_balance: Decimal,
    pub snapshot_tx_id: i64,
    pub last_daily_credit: Option<NaiveDate>,
    pub total_credited: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WalletTransaction {
    pub id: i64,
    pub user_id: String,
    pub amount: Decimal,
    pub escrow_delta: Decimal,
    pub balance_after: Option<Decimal>,
    pub tx_type: String,
    pub description: String,
    pub reference_id: Option<String>,
    pub memo: Option<String>,
    pub counterparty_id: String,
    pub tx_group_id: String,
    pub principal_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub escrow_after: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct WalletLedgerPosition {
    pub balance: Decimal,
    pub escrow_balance: Decimal,
    pub snapshot_tx_id: i64,
}

#[derive(Debug, Clone, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TransactionType {
    DailyCredit,
    BlackjackBet,
    BlackjackWin,
    StockBuy,
    StockSell,
    TransferIn,
    TransferOut,
    TurnipBuy,
    TurnipSell,
    LandBuy,
    LandSell,
    LandCollect,
    FarmPlant,
    FarmFertilize,
    GameEntry,
    GameWin,
    GameRefund,
    BattleWin,
    FuturesMargin,
    FuturesPnl,
    FuturesLiquidation,
    FuturesFunding,
    BalanceAdjustment,
}

pub mod ids {
    pub const FUTURES_MM_TREASURY: &str = "__futures_mm_treasury__";
    pub const FUTURES_INSURANCE_FUND: &str = "__futures_insurance_fund__";
    pub const FUTURES_HEDGE_TREASURY: &str = "__futures_hedge_treasury__";
    pub const RAID_EXCHANGE_TREASURY: &str = "__raid_exchange_treasury__";
    pub const TURNIP_AMM_TREASURY: &str = "__turnip_amm_treasury__";
    pub const TURNIP_SPOT_TREASURY: &str = "__turnip_spot_treasury__";
    pub const INSURANCE_TREASURY: &str = "__insurance_treasury__";
    pub const LOTTERY_TREASURY: &str = "__lottery_treasury__";
    pub const SYSTEM_ADJUSTMENT: &str = "__wallet_adjustment_offset__";
}
