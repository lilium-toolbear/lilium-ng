# models/wallet/wallet_transaction.py

## 功能
钱包交易模型，用于审计日志和双重记账。

## 类

### TransactionType(StrEnum)
- 67 个枚举值，覆盖所有交易类型
- 每个枚举有 `label` (中文显示名) 和 `category` (income/expense/transfer) 属性

### WalletTransaction(SQLModel, table=True)

**表名**: `wallet_transaction`

**字段**:
- `id: Optional[int]` - 自增主键
- `user_id: str` - 用户 ID
- `amount: Decimal` - 交易金额 (+credit, -debit)
- `escrow_delta: Decimal` - 托管 delta (+freeze, -release)
- `balance_after: Optional[Decimal]` - 交易后余额
- `tx_type: str` - 交易类型代码
- `description: str` - 可读描述
- `reference_id: Optional[str]` - 引用 ID
- `memo: Optional[str]` - 备注
- `counterparty_id: str` - 对手方 ID (双重记账)
- `tx_group_id: str` - 交易组 ID (逻辑分组)
- `principal_id: Optional[UUID]` - 主体 ID
- `metadata_json: Optional[Dict]` - 结构化元数据
- `escrow_after: Optional[Decimal]` - 交易后托管余额
- `created_at: datetime` - 创建时间

**索引** (覆盖索引):
- `ix_wallet_transaction_user_id_cover_amount`
- `ix_wallet_transaction_user_id_id_snapshot_tail`
- `ix_wallet_transaction_user_tx_type_cover_amount`
- `ix_wallet_transaction_created_at_cover_user_amount`
- `ix_wallet_transaction_tx_type_created_at_cover_amount`

## 依赖模块
- `sqlalchemy.TIMESTAMP, Column, Index`
- `sqlalchemy.dialects.postgresql.JSONB`
- `sqlmodel.Field, SQLModel`
- `models.common.base.FixedDecimal2, utc_now`

## Rust 映射
- 位置: `crates/lilium-models/src/wallet/mod.rs`
- 状态: ⚠️ 简化实现（TransactionType 只有部分枚举值）
