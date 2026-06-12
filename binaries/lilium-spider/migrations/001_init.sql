-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    user_id VARCHAR(255) PRIMARY KEY,
    is_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- WebSocket events table (partitioned by timestamp)
CREATE TABLE IF NOT EXISTS websocket_events (
    id BIGINT GENERATED ALWAYS AS IDENTITY,
    event VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, timestamp)
) PARTITION BY RANGE (timestamp);

-- Messages table (partitioned by sent_at)
CREATE TABLE IF NOT EXISTS messages (
    message_id VARCHAR(255) NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL,
    room_id VARCHAR(255) NOT NULL,
    sent_by VARCHAR(255),
    content_text TEXT,
    content_type VARCHAR(255),
    is_deleted BOOLEAN DEFAULT FALSE,
    is_recalled BOOLEAN DEFAULT FALSE,
    is_edited BOOLEAN DEFAULT FALSE,
    history JSONB,
    raw_data JSONB NOT NULL,
    source VARCHAR(50) NOT NULL,
    PRIMARY KEY (message_id, sent_at)
) PARTITION BY RANGE (sent_at);

-- Wallet table
CREATE TABLE IF NOT EXISTS wallet (
    user_id VARCHAR(255) PRIMARY KEY,
    allow_negative_balance BOOLEAN DEFAULT FALSE,
    snapshot_balance DECIMAL(38,2) DEFAULT 0,
    snapshot_escrow_balance DECIMAL(38,2) DEFAULT 0,
    snapshot_tx_id BIGINT DEFAULT 0,
    total_credited DECIMAL(38,2) DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Wallet transactions table
CREATE TABLE IF NOT EXISTS wallet_transaction (
    id BIGSERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    amount DECIMAL(38,2) NOT NULL,
    escrow_delta DECIMAL(38,2) DEFAULT 0,
    balance_after DECIMAL(38,2),
    tx_type VARCHAR(50) NOT NULL,
    description VARCHAR(200) NOT NULL,
    reference_id VARCHAR(100),
    memo VARCHAR(200),
    counterparty_id VARCHAR(100) NOT NULL,
    tx_group_id VARCHAR(100) NOT NULL,
    metadata JSONB,
    escrow_after DECIMAL(38,2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Event processor offsets table
CREATE TABLE IF NOT EXISTS event_processor_offsets (
    processor_id VARCHAR(255) PRIMARY KEY,
    last_processed_id BIGINT DEFAULT 0,
    last_processed_timestamp TIMESTAMPTZ,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
