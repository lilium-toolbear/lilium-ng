# Model 生成的实际 SQL 表结构

## 重要说明
以下是从 Python model 定义推导出的实际 SQL 表结构。这些是 Rust 代码中应该使用的正确 SQL，而不是我之前编造的。

---

## messages 表

```sql
CREATE TABLE messages (
    -- 复合主键 (message_id, sent_at) 用于分区
    message_id VARCHAR NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL,
    
    -- 核心字段
    room_id VARCHAR NOT NULL,
    sent_by VARCHAR NOT NULL,
    
    -- 内容
    content_type VARCHAR NOT NULL,
    content_text TEXT,
    content_tsv TSVECTOR,  -- 全文搜索向量
    
    -- 附件
    attachment_url TEXT,
    attachment_file TEXT,
    sticker_id VARCHAR,
    alt_text TEXT,
    
    -- 元数据
    metadata JSONB,
    raw_data JSONB NOT NULL,
    
    -- 来源
    source VARCHAR NOT NULL,
    
    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    
    -- 删除追踪
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at TIMESTAMPTZ,
    deleted_by VARCHAR,
    
    -- 撤回追踪
    is_recalled BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- 编辑追踪
    is_edited BOOLEAN NOT NULL DEFAULT FALSE,
    history JSONB,
    
    -- 引用
    reference_message_id VARCHAR,
    reference_data JSONB,
    
    PRIMARY KEY (message_id, sent_at)
) PARTITION BY RANGE (sent_at);

-- 索引
CREATE INDEX idx_messages_source_created_at_id ON messages (source, created_at, message_id);
CREATE INDEX idx_messages_room_id_sent_at ON messages (room_id, sent_at);
CREATE INDEX idx_messages_sent_by_sent_at_id ON messages (sent_by, sent_at, message_id);
CREATE INDEX ix_messages_sent_at ON messages (sent_at);
CREATE INDEX idx_messages_content_tsv ON messages USING GIN (content_tsv);
```

---

## wallet 表

```sql
CREATE TABLE wallet (
    user_id VARCHAR PRIMARY KEY,  -- 外键到 users 表
    allow_negative_balance BOOLEAN NOT NULL DEFAULT FALSE,
    snapshot_balance DECIMAL(38,2) NOT NULL DEFAULT 0,
    snapshot_escrow_balance DECIMAL(38,2) NOT NULL DEFAULT 0,
    snapshot_tx_id BIGINT NOT NULL DEFAULT 0,
    last_daily_credit DATE,
    total_credited DECIMAL(38,2) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## wallet_transaction 表

```sql
CREATE TABLE wallet_transaction (
    id BIGSERIAL PRIMARY KEY,
    user_id VARCHAR NOT NULL,
    amount DECIMAL(38,2) NOT NULL,
    escrow_delta DECIMAL(38,2) NOT NULL DEFAULT 0,
    balance_after DECIMAL(38,2),
    tx_type VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    reference_id VARCHAR,
    memo VARCHAR,
    counterparty_id VARCHAR NOT NULL,
    tx_group_id VARCHAR NOT NULL,
    principal_id UUID,
    metadata JSONB,
    escrow_after DECIMAL(38,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX ix_wallet_transaction_user_id_cover_amount ON wallet_transaction (user_id) INCLUDE (amount, escrow_delta);
CREATE INDEX ix_wallet_transaction_user_id_id_snapshot_tail ON wallet_transaction (user_id, id) INCLUDE (amount, escrow_delta);
CREATE INDEX ix_wallet_transaction_user_tx_type_cover_amount ON wallet_transaction (user_id, tx_type) INCLUDE (amount);
CREATE INDEX ix_wallet_transaction_created_at_cover_user_amount ON wallet_transaction (created_at) INCLUDE (user_id, amount);
CREATE INDEX ix_wallet_transaction_tx_type_created_at_cover_amount ON wallet_transaction (tx_type, created_at) INCLUDE (amount);
```

---

## websocket_events 表

```sql
CREATE TABLE websocket_events (
    id BIGINT GENERATED ALWAYS AS IDENTITY,
    timestamp TIMESTAMPTZ NOT NULL,
    user_id VARCHAR NOT NULL,
    event VARCHAR NOT NULL,
    data JSONB NOT NULL,
    PRIMARY KEY (id, timestamp)
) PARTITION BY RANGE (timestamp);

-- 索引
CREATE INDEX ix_websocket_events_timestamp_id ON websocket_events (timestamp, id);
CREATE INDEX ix_websocket_events_user_id_timestamp_id ON websocket_events (user_id, timestamp, id);
CREATE INDEX ix_websocket_events_event_timestamp_id ON websocket_events (event, timestamp, id);
```

---

## event_processor_offsets 表

```sql
CREATE TABLE event_processor_offsets (
    processor_id VARCHAR PRIMARY KEY,
    last_processed_id BIGINT NOT NULL DEFAULT 0,
    last_processed_timestamp TIMESTAMPTZ,
    last_processed_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## dzmm_account 表

```sql
CREATE TABLE dzmm_account (
    user_id VARCHAR PRIMARY KEY,  -- 外键到 users 表
    user_profile JSONB NOT NULL,
    email VARCHAR,
    password VARCHAR,
    signin_code VARCHAR,
    signin_code_image BYTEA,
    signin_code_image_mime VARCHAR,
    cookies TEXT,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## websocket_connections 表

```sql
CREATE TABLE websocket_connections (
    lock_id BIGINT PRIMARY KEY,
    account_user_id VARCHAR NOT NULL,  -- 外键到 dzmm_account
    connected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX ix_websocket_connections_account_user_id ON websocket_connections (account_user_id);
```

---

## outgoing_commands 表

```sql
CREATE TABLE outgoing_commands (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    account_user_id VARCHAR NOT NULL,
    event VARCHAR NOT NULL,
    data JSONB NOT NULL,
    require_ack BOOLEAN DEFAULT TRUE,
    status VARCHAR NOT NULL DEFAULT 'pending',
    processed_at TIMESTAMPTZ,
    ack_response JSONB,
    error_message TEXT,
    attempt_count INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3
);

CREATE INDEX ix_outgoing_commands_created_at ON outgoing_commands (created_at);
CREATE INDEX ix_outgoing_commands_account_user_id ON outgoing_commands (account_user_id);
CREATE INDEX ix_outgoing_commands_status ON outgoing_commands (status);
```

---

## 这些是 Rust 代码中应该使用的正确 SQL
- 不要编造 SQL
- 从 model 定义推导实际表结构
- 使用正确的字段类型和约束
