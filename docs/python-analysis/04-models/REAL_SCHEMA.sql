-- Table: arena_session
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    mode VARCHAR NOT NULL DEFAULT ...,
    status VARCHAR NOT NULL,
    current_round INTEGER NOT NULL DEFAULT ...,
    turn_no INTEGER NOT NULL DEFAULT ...,
    round_seed INTEGER NOT NULL,
    state_json JSONB NOT NULL,
    result_summary JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...,
    ended_at TIMESTAMP
);

-- Table: auth_session
    id INTEGER PRIMARY KEY NOT NULL,
    session_id VARCHAR(64) NOT NULL,
    code VARCHAR(16) NOT NULL,
    user_id VARCHAR,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    authenticated_at TIMESTAMP
);

-- Table: battle_records
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    mode VARCHAR(10) NOT NULL,
    rounds_cleared INTEGER NOT NULL,
    pal_ids JSONB NOT NULL,
    pal_levels JSONB NOT NULL,
    exp_earned INTEGER NOT NULL DEFAULT ...,
    credits_earned INTEGER NOT NULL DEFAULT ...,
    battle_log JSONB NOT NULL,
    active_state JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: blackjack_sessions
    id VARCHAR PRIMARY KEY NOT NULL,
    user_id VARCHAR,
    room_id VARCHAR,
    status VARCHAR NOT NULL,
    state_json JSONB NOT NULL,
    result_json JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL,
    finished_at TIMESTAMP
);

-- Table: books
    book_id VARCHAR PRIMARY KEY NOT NULL,
    title VARCHAR,
    description TEXT,
    slug VARCHAR,
    is_nsfw BOOLEAN NOT NULL DEFAULT ...,
    is_public BOOLEAN NOT NULL DEFAULT ...,
    cover_image_url VARCHAR,
    local_cover_path VARCHAR,
    user_id VARCHAR,
    author JSONB,
    chapter_count INTEGER NOT NULL DEFAULT ...,
    total_word_count INTEGER NOT NULL DEFAULT ...,
    latest_chapter JSONB,
    likes_count INTEGER NOT NULL DEFAULT ...,
    comments_count INTEGER NOT NULL DEFAULT ...,
    top_comments JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP,
    published_at TIMESTAMP,
    fetched_at TIMESTAMP NOT NULL DEFAULT ...,
    raw_data JSONB
);

-- Table: bot_memory
    id INTEGER PRIMARY KEY NOT NULL,
    namespace VARCHAR NOT NULL,
    room_id VARCHAR,
    user_id VARCHAR,
    key VARCHAR NOT NULL DEFAULT ...,
    value JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP
);

-- Table: cards
    card_id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR,
    card_filename VARCHAR,
    original_filename VARCHAR,
    creator VARCHAR,
    creator_notes TEXT,
    user_id VARCHAR,
    creator_full_name VARCHAR,
    creator_avatar_url VARCHAR,
    tags ARRAY,
    is_public BOOLEAN NOT NULL DEFAULT ...,
    is_sensitive BOOLEAN NOT NULL DEFAULT ...,
    is_image_blur BOOLEAN NOT NULL DEFAULT ...,
    is_gamefy BOOLEAN NOT NULL DEFAULT ...,
    image_info JSONB,
    weighted_rating VARCHAR,
    popularity_score VARCHAR,
    likes_count INTEGER NOT NULL DEFAULT ...,
    comments_count INTEGER NOT NULL DEFAULT ...,
    top_comments JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    published_at TIMESTAMP,
    fetched_at TIMESTAMP NOT NULL DEFAULT ...,
    raw_data JSONB
);

-- Table: chapters
    chapter_id VARCHAR PRIMARY KEY NOT NULL,
    title VARCHAR,
    content TEXT,
    is_adult BOOLEAN NOT NULL DEFAULT ...,
    is_nsfw BOOLEAN NOT NULL DEFAULT ...,
    user_id VARCHAR,
    author JSONB,
    likes_count INTEGER NOT NULL DEFAULT ...,
    comments_count INTEGER NOT NULL DEFAULT ...,
    top_comments JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP,
    published_at TIMESTAMP,
    fetched_at TIMESTAMP NOT NULL DEFAULT ...,
    raw_data JSONB
);

-- Table: checkpoints
    checkpoint_id VARCHAR PRIMARY KEY NOT NULL,
    name VARCHAR,
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT ...,
    user_id VARCHAR,
    user_name VARCHAR,
    user_avatar_url VARCHAR,
    creator JSONB,
    rating_avg VARCHAR,
    rating_count INTEGER NOT NULL DEFAULT ...,
    review_status VARCHAR,
    share_code VARCHAR,
    character_cards JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP,
    fetched_at TIMESTAMP NOT NULL DEFAULT ...,
    raw_data JSONB
);

-- Table: clearing_batch
    batch_id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    partner_user_id VARCHAR NOT NULL,
    partner_client_id CHAR(32) NOT NULL,
    operation VARCHAR(64) NOT NULL,
    account_code VARCHAR(255) NOT NULL,
    asset_code VARCHAR(32) NOT NULL DEFAULT ...,
    batch_reference_id VARCHAR(255) NOT NULL,
    status VARCHAR(64) NOT NULL DEFAULT ...,
    item_count INTEGER NOT NULL DEFAULT ...,
    success_count INTEGER NOT NULL DEFAULT ...,
    failed_count INTEGER NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    completed_at TIMESTAMP
);

-- Table: clearing_batch_item
    batch_item_id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    batch_id CHAR(32) NOT NULL,
    intent_id CHAR(32),
    user_id VARCHAR NOT NULL,
    amount DECIMAL(38, 2),
    partner_reference_id VARCHAR(255) NOT NULL,
    note VARCHAR(2000),
    status VARCHAR(64) NOT NULL DEFAULT ...,
    error_code VARCHAR(64),
    error_message VARCHAR(2000)
);

-- Table: clearing_instruction
    instruction_id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    partner_user_id VARCHAR NOT NULL,
    partner_client_id CHAR(32) NOT NULL,
    intent_id CHAR(32),
    user_id VARCHAR NOT NULL,
    operation VARCHAR(64) NOT NULL,
    account_code VARCHAR(255) NOT NULL,
    amount DECIMAL(38, 2),
    asset_code VARCHAR(32) NOT NULL DEFAULT ...,
    partner_reference_id VARCHAR(255) NOT NULL,
    reason VARCHAR(255) NOT NULL,
    note VARCHAR(2000),
    status VARCHAR(64) NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    reverse_of_instruction_id CHAR(32),
    executed_at TIMESTAMP,
    error_code VARCHAR(64),
    error_message VARCHAR(2000)
);

-- Table: crop_insurance_policy
    id INTEGER PRIMARY KEY NOT NULL,
    seed_id INTEGER NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    mode VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL,
    strike DECIMAL(38, 2) NOT NULL,
    coverage_qty DECIMAL(38, 0) NOT NULL,
    premium_paid DECIMAL(38, 2) NOT NULL,
    spot_at_purchase DECIMAL(38, 2) NOT NULL,
    coverage_level DECIMAL(38, 6) NOT NULL,
    purchased_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    settlement_twap DECIMAL(38, 2),
    payout DECIMAL(38, 2),
    settled_at TIMESTAMP,
    cancelled_at TIMESTAMP,
    cancel_refund DECIMAL(38, 2),
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: currency_exchange_order
    id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    user_id VARCHAR(64) NOT NULL,
    currency VARCHAR(32) NOT NULL DEFAULT ...,
    direction VARCHAR(24) NOT NULL,
    input_amount DECIMAL(38, 8) NOT NULL,
    output_amount DECIMAL(38, 8) NOT NULL,
    rate DECIMAL(38, 8) NOT NULL,
    fee_amount DECIMAL(38, 8) NOT NULL DEFAULT ...,
    wallet_delta DECIMAL(38, 2) NOT NULL,
    game_delta DECIMAL(38, 6) NOT NULL,
    provider VARCHAR(32) NOT NULL DEFAULT ...,
    provider_revision INTEGER NOT NULL DEFAULT ...,
    wallet_inflation DECIMAL(38, 8) NOT NULL DEFAULT ...,
    game_inflation DECIMAL(38, 8) NOT NULL DEFAULT ...,
    status VARCHAR(16) NOT NULL DEFAULT ...,
    expires_at TIMESTAMP,
    idempotency_key VARCHAR(64),
    executed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: currency_rate_state
    currency VARCHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    current_rate DECIMAL(38, 8) NOT NULL,
    revision INTEGER NOT NULL DEFAULT ...,
    baseline_wallet_supply DECIMAL(38, 2) NOT NULL DEFAULT ...,
    baseline_game_supply DECIMAL(38, 6) NOT NULL DEFAULT ...,
    last_wallet_supply DECIMAL(38, 2) NOT NULL DEFAULT ...,
    last_game_supply DECIMAL(38, 6) NOT NULL DEFAULT ...,
    macro_anchor_rate DECIMAL(38, 8) NOT NULL DEFAULT ...,
    macro_anchor_updated_at TIMESTAMP NOT NULL DEFAULT ...,
    short_term_premium DECIMAL(38, 8) NOT NULL DEFAULT ...,
    bank_backstop_hour_start TIMESTAMP NOT NULL DEFAULT ...,
    bank_backstop_hour_used DECIMAL(38, 2) NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: discord_account_binding
    id INTEGER PRIMARY KEY NOT NULL,
    discord_user_id VARCHAR(32) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    discord_username VARCHAR(128),
    discord_global_name VARCHAR(128),
    discord_avatar_hash VARCHAR(128),
    discord_avatar_url VARCHAR(512),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    last_login_at TIMESTAMP
);

-- Table: dzmm_account
    user_id VARCHAR PRIMARY KEY NOT NULL,
    user_profile JSONB NOT NULL,
    email VARCHAR,
    password VARCHAR,
    signin_code VARCHAR,
    signin_code_image BLOB,
    signin_code_image_mime VARCHAR,
    cookies VARCHAR,
    is_enabled BOOLEAN NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: economy_config_version
    id INTEGER PRIMARY KEY NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT ...,
    config_payload JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    created_by VARCHAR,
    note VARCHAR,
    parent_version_id INTEGER
);

-- Table: economy_control_plane_state
    id INTEGER PRIMARY KEY NOT NULL DEFAULT ...,
    active_config_version_id INTEGER,
    active_guardrail_policy_version_id INTEGER,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: economy_control_state
    id INTEGER PRIMARY KEY NOT NULL DEFAULT ...,
    seed_weight DECIMAL(38, 6) NOT NULL DEFAULT ...,
    inventory_conservation_enabled BOOLEAN NOT NULL DEFAULT ...,
    amm_trade_fee_bps INTEGER NOT NULL DEFAULT ...,
    cash_injection_multiplier DECIMAL(38, 6) NOT NULL DEFAULT ...,
    treasury_policy_enabled BOOLEAN NOT NULL DEFAULT ...,
    treasury_policy_k DECIMAL(38, 6),
    treasury_policy_beta DECIMAL(38, 6),
    stored_decay_bps_per_day INTEGER NOT NULL DEFAULT ...,
    turnip_runtime_overrides JSONB,
    coverage_ratio DECIMAL(38, 8) NOT NULL DEFAULT ...,
    coverage_ema DECIMAL(38, 8) NOT NULL DEFAULT ...,
    bank_liability DECIMAL(38, 2) NOT NULL DEFAULT ...,
    runtime_metrics_updated_at TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT ...,
    futures_payout_bucket_level DECIMAL(38, 2) NOT NULL DEFAULT ...,
    futures_payout_bucket_max DECIMAL(38, 2) NOT NULL DEFAULT ...,
    futures_payout_bucket_refill_rate DECIMAL(38, 2) NOT NULL DEFAULT ...,
    last_refill_at TIMESTAMP
);

-- Table: economy_overview_component_snapshot
    component_key VARCHAR(64) PRIMARY KEY NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT ...,
    payload JSONB,
    generated_at TIMESTAMP NOT NULL DEFAULT ...,
    expires_at TIMESTAMP NOT NULL DEFAULT ...,
    refresh_started_at TIMESTAMP,
    refresh_owner VARCHAR(128),
    refresh_duration_ms INTEGER,
    refresh_error VARCHAR(1000),
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: economy_snapshot
    id INTEGER PRIMARY KEY NOT NULL,
    config_version_id INTEGER,
    liquidity DECIMAL(38, 2) NOT NULL DEFAULT ...,
    effective_supply DECIMAL(38, 2) NOT NULL DEFAULT ...,
    coverage_ratio DECIMAL(38, 8) NOT NULL DEFAULT ...,
    bank_liability DECIMAL(38, 2) NOT NULL DEFAULT ...,
    captured_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: event_processor_offsets
    processor_id VARCHAR PRIMARY KEY NOT NULL,
    last_processed_id INTEGER NOT NULL DEFAULT ...,
    last_processed_timestamp TIMESTAMP,
    last_processed_at TIMESTAMP,
    updated_at TIMESTAMP NOT NULL
);

-- Table: external_command_effect
    id INTEGER PRIMARY KEY NOT NULL,
    invocation_id VARCHAR(128) NOT NULL,
    effect_index INTEGER NOT NULL,
    effect_id VARCHAR(128) NOT NULL,
    effect_type VARCHAR(64) NOT NULL,
    status VARCHAR(64) NOT NULL,
    executed_at TIMESTAMP,
    error_code VARCHAR(64),
    error_message VARCHAR(1024)
);

-- Table: external_command_invocation
    invocation_id VARCHAR(128) PRIMARY KEY NOT NULL,
    config_id VARCHAR(128) NOT NULL,
    command_name VARCHAR(256) NOT NULL,
    matched_name VARCHAR(256) NOT NULL,
    room_id VARCHAR(128) NOT NULL,
    sender_id VARCHAR(128) NOT NULL,
    message_id VARCHAR(128) NOT NULL,
    status VARCHAR(64) NOT NULL,
    started_at TIMESTAMP NOT NULL DEFAULT ...,
    completed_at TIMESTAMP,
    error_code VARCHAR(64),
    error_message VARCHAR(1024)
);

-- Table: external_command_session
    local_session_id VARCHAR(128) PRIMARY KEY NOT NULL,
    config_id VARCHAR(128) NOT NULL,
    command_name VARCHAR(256) NOT NULL,
    room_id VARCHAR(128) NOT NULL,
    room_type VARCHAR(64) NOT NULL,
    sender_id VARCHAR(128) NOT NULL,
    invocation_id VARCHAR(128) NOT NULL,
    endpoint VARCHAR(1024) NOT NULL,
    external_session_id VARCHAR(128),
    status VARCHAR(64) NOT NULL,
    listen_rules JSONB NOT NULL,
    timers JSONB NOT NULL,
    start_message JSONB NOT NULL,
    last_acked_sequence INTEGER NOT NULL DEFAULT ...,
    last_effect_id VARCHAR(128),
    last_ack_status VARCHAR(64),
    last_ack_error_code VARCHAR(64),
    last_ack_error_message VARCHAR(1024),
    connected_at TIMESTAMP,
    disconnected_at TIMESTAMP,
    expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...,
    error_code VARCHAR(64),
    error_message VARCHAR(1024)
);

-- Table: farm_weather_hour
    id INTEGER PRIMARY KEY NOT NULL,
    starts_at TIMESTAMP NOT NULL,
    weather_type VARCHAR(20) NOT NULL,
    yield_factor DECIMAL(38, 6) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: futures_engine_state
    id INTEGER PRIMARY KEY NOT NULL,
    npc_net_position DECIMAL(38, 0) NOT NULL DEFAULT ...,
    hedge_pending_delta DECIMAL(38, 0) NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: futures_order
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    side VARCHAR(10) NOT NULL,
    order_type VARCHAR(10) NOT NULL DEFAULT ...,
    intent VARCHAR(10) NOT NULL,
    close_position_id INTEGER,
    price DECIMAL(38, 2),
    quantity DECIMAL(38, 0) NOT NULL,
    filled_quantity DECIMAL(38, 0) NOT NULL DEFAULT ...,
    margin_frozen DECIMAL(38, 2) NOT NULL DEFAULT ...,
    status VARCHAR(20) NOT NULL DEFAULT ...,
    cancel_requested BOOLEAN NOT NULL DEFAULT ...,
    expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: futures_order_fill
    id INTEGER PRIMARY KEY NOT NULL,
    buy_order_id INTEGER,
    sell_order_id INTEGER,
    price DECIMAL(38, 2) NOT NULL,
    quantity DECIMAL(38, 0) NOT NULL,
    buyer_id VARCHAR NOT NULL,
    seller_id VARCHAR NOT NULL,
    settled BOOLEAN NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: futures_position
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    side VARCHAR(5) NOT NULL,
    quantity DECIMAL(38, 0) NOT NULL,
    entry_price DECIMAL(38, 2) NOT NULL,
    margin DECIMAL(38, 2) NOT NULL,
    liquidation_price DECIMAL(38, 2) NOT NULL,
    take_profit_price DECIMAL(38, 2),
    stop_loss_price DECIMAL(38, 2),
    pending_close_quantity DECIMAL(38, 0) NOT NULL DEFAULT ...,
    status VARCHAR(10) NOT NULL DEFAULT ...,
    realized_pnl DECIMAL(38, 2) NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    closed_at TIMESTAMP
);

-- Table: futures_price_snapshot
    id INTEGER PRIMARY KEY NOT NULL,
    mid_price DECIMAL(38, 2) NOT NULL,
    spot_price DECIMAL(38, 2) NOT NULL,
    funding_rate DECIMAL(38, 8) NOT NULL DEFAULT ...,
    futures_price DECIMAL(38, 2) NOT NULL DEFAULT ...,
    open DECIMAL(38, 2),
    high DECIMAL(38, 2),
    low DECIMAL(38, 2),
    volume DECIMAL(38, 0),
    trade_count INTEGER,
    open_interest DECIMAL(38, 0) NOT NULL DEFAULT ...,
    total_long DECIMAL(38, 0) NOT NULL DEFAULT ...,
    total_short DECIMAL(38, 0) NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: futures_transaction
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    position_id INTEGER,
    tx_type VARCHAR(11) NOT NULL,
    quantity DECIMAL(38, 0) NOT NULL DEFAULT ...,
    price DECIMAL(38, 2) NOT NULL DEFAULT ...,
    pnl DECIMAL(38, 2) NOT NULL DEFAULT ...,
    raw_pnl DECIMAL(38, 2) NOT NULL DEFAULT ...,
    paid_pnl DECIMAL(38, 2) NOT NULL DEFAULT ...,
    unpaid_pnl DECIMAL(38, 2) NOT NULL DEFAULT ...,
    fee DECIMAL(38, 2) NOT NULL DEFAULT ...,
    margin_change DECIMAL(38, 2) NOT NULL DEFAULT ...,
    spot_price DECIMAL(38, 2),
    description VARCHAR NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: galleries
    gallery_id VARCHAR PRIMARY KEY NOT NULL,
    title VARCHAR,
    user_id VARCHAR,
    user_full_name VARCHAR,
    user_avatar_url VARCHAR,
    images ARRAY,
    local_image_paths ARRAY,
    likes_count INTEGER NOT NULL DEFAULT ...,
    dislikes_count INTEGER NOT NULL DEFAULT ...,
    comments_count INTEGER NOT NULL DEFAULT ...,
    top_comments JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    fetched_at TIMESTAMP NOT NULL DEFAULT ...,
    raw_data JSONB
);

-- Table: game_currency_account
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    currency VARCHAR(32) NOT NULL DEFAULT ...,
    balance DECIMAL(38, 6) NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: game_currency_transaction
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    currency VARCHAR(32) NOT NULL DEFAULT ...,
    amount DECIMAL(38, 6) NOT NULL,
    balance_after DECIMAL(38, 6) NOT NULL,
    tx_type VARCHAR(32) NOT NULL,
    description VARCHAR(255) NOT NULL DEFAULT ...,
    reference_id VARCHAR(128),
    order_id VARCHAR(64),
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: game_escrow
    escrow_token CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    game_id VARCHAR NOT NULL,
    game_type VARCHAR NOT NULL,
    status VARCHAR NOT NULL DEFAULT ...,
    locked_funds JSONB NOT NULL,
    total_locked FLOAT NOT NULL DEFAULT ...,
    settled_payouts JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    settled_at TIMESTAMP
);

-- Table: games
    id VARCHAR PRIMARY KEY NOT NULL,
    game_type VARCHAR NOT NULL,
    room_id VARCHAR NOT NULL,
    creator_id VARCHAR NOT NULL,
    status VARCHAR NOT NULL,
    config JSONB NOT NULL,
    players JSONB NOT NULL,
    result JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    started_at TIMESTAMP,
    finished_at TIMESTAMP
);

-- Table: games.lottery_draw
    id INTEGER PRIMARY KEY NOT NULL,
    draw_date DATE NOT NULL,
    draw_at TIMESTAMP,
    sales_close_at TIMESTAMP,
    status VARCHAR NOT NULL DEFAULT ...,
    ticket_count INTEGER NOT NULL DEFAULT ...,
    ticket_unit_count INTEGER NOT NULL DEFAULT ...,
    ticket_price DECIMAL(38, 2) NOT NULL DEFAULT ...,
    prize_contribution_rate DECIMAL(38, 2) NOT NULL DEFAULT ...,
    house_retention_rate DECIMAL(38, 2) NOT NULL DEFAULT ...,
    pool_before_sales DECIMAL(38, 2) NOT NULL DEFAULT ...,
    sales_amount DECIMAL(38, 2) NOT NULL DEFAULT ...,
    prize_contribution DECIMAL(38, 2) NOT NULL DEFAULT ...,
    house_retention DECIMAL(38, 2) NOT NULL DEFAULT ...,
    pool_before_draw DECIMAL(38, 2) NOT NULL DEFAULT ...,
    allocated_prize_amount DECIMAL(38, 2) NOT NULL DEFAULT ...,
    paid_prize_amount DECIMAL(38, 2) NOT NULL DEFAULT ...,
    carryover_amount DECIMAL(38, 2) NOT NULL DEFAULT ...,
    winning_numbers ARRAY,
    draw_hash VARCHAR(64),
    previous_draw_hash VARCHAR(64),
    algorithm_version VARCHAR(64),
    transcript_json JSONB,
    ticket_commitment_input TEXT,
    settlement_stats_json JSONB,
    failure_code VARCHAR(64),
    failure_message VARCHAR,
    closed_at TIMESTAMP,
    draw_started_at TIMESTAMP,
    drawn_at TIMESTAMP,
    settlement_started_at TIMESTAMP,
    settled_at TIMESTAMP,
    settlement_cursor_ticket_id INTEGER,
    settlement_completed_at TIMESTAMP,
    notification_started_at TIMESTAMP,
    notification_cursor_user_id VARCHAR(255),
    notification_completed_at TIMESTAMP,
    retry_after TIMESTAMP,
    retry_count INTEGER NOT NULL DEFAULT ...,
    stage_lease_expires_at TIMESTAMP,
    stage_owner VARCHAR(128),
    notifications_sent_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL
);

-- Table: games.lottery_payout
    id INTEGER PRIMARY KEY NOT NULL,
    draw_id INTEGER NOT NULL,
    ticket_id INTEGER NOT NULL,
    user_id VARCHAR NOT NULL,
    prize_tier VARCHAR(16) NOT NULL,
    gross_amount DECIMAL(38, 2) NOT NULL,
    tax_amount DECIMAL(38, 2) NOT NULL DEFAULT ...,
    net_amount DECIMAL(38, 2) NOT NULL,
    wallet_transaction_id INTEGER,
    tax_transaction_id INTEGER,
    reference_id VARCHAR(128) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: games.lottery_ticket
    id INTEGER PRIMARY KEY NOT NULL,
    draw_id INTEGER NOT NULL,
    user_id VARCHAR NOT NULL,
    numbers ARRAY NOT NULL,
    multiplier INTEGER NOT NULL DEFAULT ...,
    unit_price DECIMAL(38, 2) NOT NULL DEFAULT ...,
    total_price DECIMAL(38, 2) NOT NULL DEFAULT ...,
    status VARCHAR NOT NULL DEFAULT ...,
    wallet_transaction_id INTEGER,
    purchased_at TIMESTAMP NOT NULL DEFAULT ...,
    match_count INTEGER,
    prize_tier VARCHAR(16),
    idempotency_key VARCHAR(128) NOT NULL,
    line_index INTEGER NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: games.ponzi_events
    id INTEGER PRIMARY KEY NOT NULL,
    session_id UUID NOT NULL,
    seq INTEGER NOT NULL,
    event_type VARCHAR(64) NOT NULL,
    actor_user_id VARCHAR(128),
    public_payload JSONB NOT NULL,
    private_payload JSONB NOT NULL,
    public_message_status VARCHAR(16) NOT NULL DEFAULT ...,
    public_message_attempts INTEGER NOT NULL DEFAULT ...,
    public_message_sent_at TIMESTAMP,
    public_message_error TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: games.ponzi_sessions
    id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    room_id VARCHAR(128) NOT NULL,
    status VARCHAR(24) NOT NULL DEFAULT ...,
    phase VARCHAR(32) NOT NULL DEFAULT ...,
    players JSONB NOT NULL,
    state JSONB NOT NULL,
    pending_trade JSONB,
    stop_votes JSONB NOT NULL,
    result JSONB,
    latest_event_seq INTEGER NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    started_at TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT ...,
    finished_at TIMESTAMP
);

-- Table: idempotency_record
    record_id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    key VARCHAR(128) NOT NULL,
    partner_user_id VARCHAR NOT NULL,
    partner_client_id VARCHAR(128) NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    request_hash VARCHAR(64) NOT NULL,
    response_status INTEGER,
    response_body JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    expires_at TIMESTAMP NOT NULL
);

-- Table: image_gps
    message_id VARCHAR PRIMARY KEY NOT NULL,
    latitude FLOAT NOT NULL,
    longitude FLOAT NOT NULL,
    altitude FLOAT,
    timestamp TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: issued_principal
    principal_id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    token_kind VARCHAR(32) NOT NULL,
    owner_user_id VARCHAR(255) NOT NULL,
    subject_user_id VARCHAR(255) NOT NULL,
    effective_account_user_id VARCHAR(255) NOT NULL,
    actor_user_id VARCHAR(255) NOT NULL,
    client_id CHAR(32),
    scope_snapshot JSONB NOT NULL,
    issued_via VARCHAR(32) NOT NULL,
    source_principal_id CHAR(32),
    expires_at TIMESTAMP,
    revoked_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: land
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    permit_item_id VARCHAR(128),
    land_type VARCHAR(20) NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    upgrade_level INTEGER NOT NULL DEFAULT ...,
    upgrade_work_total BIGINT NOT NULL DEFAULT ...,
    upgrade_work_done DOUBLE PRECISION NOT NULL DEFAULT ...,
    upgrade_status VARCHAR(20) NOT NULL DEFAULT ...,
    upgrade_started_at TIMESTAMP,
    upgrade_completed_at TIMESTAMP
);

-- Table: land_assignment
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    pal_id BIGINT NOT NULL,
    assignment_type VARCHAR(20) NOT NULL,
    assigned_at TIMESTAMP NOT NULL DEFAULT ...,
    released_at TIMESTAMP,
    last_tick_at TIMESTAMP,
    consumption_remainder FLOAT NOT NULL DEFAULT ...
);

-- Table: llm_usage_log
    id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    user_id VARCHAR NOT NULL,
    model VARCHAR,
    request_type VARCHAR,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cached_content_tokens INTEGER,
    total_tokens INTEGER,
    latency_ms INTEGER,
    provider VARCHAR,
    provider_request_id VARCHAR,
    source VARCHAR,
    context JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: market_maker_state
    id INTEGER PRIMARY KEY NOT NULL DEFAULT ...,
    mid_price DECIMAL(38, 2) NOT NULL,
    inventory DECIMAL(38, 0) NOT NULL DEFAULT ...,
    fair_value DECIMAL(38, 2) NOT NULL,
    user_pressure DECIMAL(38, 0) NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...,
    amm_book JSONB,
    recent_trades JSONB,
    futures_vamm_x DECIMAL(48, 2) NOT NULL DEFAULT ...,
    futures_vamm_y DECIMAL(48, 2) NOT NULL DEFAULT ...,
    futures_vamm_k DECIMAL(78, 2) NOT NULL DEFAULT ...,
    futures_insurance_fund DECIMAL(38, 2) NOT NULL DEFAULT ...,
    futures_open_interest DECIMAL(38, 0) NOT NULL DEFAULT ...,
    futures_total_short DECIMAL(38, 0) NOT NULL DEFAULT ...,
    futures_anchor_spot DECIMAL(38, 2) NOT NULL DEFAULT ...,
    last_funding_at TIMESTAMP,
    is_paused BOOLEAN NOT NULL DEFAULT ...,
    futures_paused BOOLEAN NOT NULL DEFAULT ...,
    fv_noise_offset FLOAT NOT NULL DEFAULT ...,
    sub_tick INTEGER NOT NULL DEFAULT ...,
    cached_effective_depth DECIMAL(38, 2),
    cached_market_center DECIMAL(38, 2)
);

-- Table: market_order
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    side VARCHAR(10) NOT NULL,
    item_category VARCHAR(20) NOT NULL,
    item_key VARCHAR(100) NOT NULL,
    item_quality VARCHAR(30) NOT NULL DEFAULT ...,
    price DECIMAL(38, 2) NOT NULL,
    quantity DECIMAL(38, 0) NOT NULL DEFAULT ...,
    filled_quantity DECIMAL(38, 0) NOT NULL DEFAULT ...,
    escrow_amount DECIMAL(38, 2) NOT NULL DEFAULT ...,
    item_snapshot JSONB,
    pal_min_level INTEGER,
    status VARCHAR(20) NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    expires_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: market_order_fill
    id INTEGER PRIMARY KEY NOT NULL,
    buy_order_id INTEGER NOT NULL,
    sell_order_id INTEGER NOT NULL,
    quantity DECIMAL(38, 0) NOT NULL,
    price DECIMAL(38, 2) NOT NULL,
    total DECIMAL(38, 2) NOT NULL,
    fee DECIMAL(38, 2) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: messages
    message_id VARCHAR PRIMARY KEY NOT NULL,
    room_id VARCHAR NOT NULL,
    sent_at TIMESTAMP PRIMARY KEY NOT NULL,
    sent_by VARCHAR NOT NULL,
    content_type VARCHAR NOT NULL,
    content_text VARCHAR,
    content_tsv TSVECTOR,
    attachment_url VARCHAR,
    attachment_file VARCHAR,
    sticker_id VARCHAR,
    alt_text VARCHAR,
    metadata JSONB,
    raw_data JSONB NOT NULL,
    source VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP,
    is_deleted BOOLEAN NOT NULL DEFAULT ...,
    deleted_at TIMESTAMP,
    deleted_by VARCHAR,
    is_recalled BOOLEAN NOT NULL DEFAULT ...,
    is_edited BOOLEAN NOT NULL DEFAULT ...,
    history JSONB,
    reference_message_id VARCHAR,
    reference_data JSONB
);

-- Table: oidc_refresh_token
    id INTEGER PRIMARY KEY NOT NULL,
    token_id CHAR(32) NOT NULL DEFAULT ...,
    partner_user_id VARCHAR NOT NULL,
    end_user_id VARCHAR NOT NULL,
    client_id CHAR(32) NOT NULL,
    token_hash VARCHAR(128) NOT NULL,
    scope VARCHAR(2048) NOT NULL DEFAULT ...,
    status VARCHAR(32) NOT NULL DEFAULT ...,
    expires_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP,
    rotated_from_token_id CHAR(32),
    revoked_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: outgoing_commands
    id INTEGER PRIMARY KEY NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    account_user_id VARCHAR NOT NULL,
    event VARCHAR NOT NULL,
    data JSONB NOT NULL,
    require_ack BOOLEAN NOT NULL DEFAULT ...,
    status VARCHAR NOT NULL DEFAULT ...,
    processed_at TIMESTAMP,
    ack_response JSONB,
    error_message VARCHAR,
    attempt_count INTEGER NOT NULL DEFAULT ...,
    max_attempts INTEGER NOT NULL DEFAULT ...
);

-- Table: pal
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    species_code VARCHAR(100) NOT NULL,
    custom_name VARCHAR(100),
    rarity INTEGER NOT NULL,
    gender VARCHAR(10) NOT NULL,
    breeding_cooldown_until TIMESTAMP,
    current_breeding_egg_id BIGINT,
    current_breeding_until TIMESTAMP,
    revival_until TIMESTAMP,
    hatched_from_egg_id BIGINT,
    archived_source_season INTEGER,
    archived_source_pal_id BIGINT,
    level INTEGER NOT NULL DEFAULT ...,
    exp INTEGER NOT NULL DEFAULT ...,
    elite_tier INTEGER NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    locked_for_order_id INTEGER,
    asset_id VARCHAR(128) NOT NULL,
    pending_gift_id CHAR(32)
);

-- Table: pal_adoption_record
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    pal_id INTEGER NOT NULL,
    adoption_date DATE NOT NULL,
    cost BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: pal_egg
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    egg_tier INTEGER NOT NULL,
    element VARCHAR(20),
    status VARCHAR(20) NOT NULL DEFAULT ...,
    price_paid DECIMAL(38, 2) NOT NULL DEFAULT ...,
    hatching_started_at TIMESTAMP,
    hatches_at TIMESTAMP,
    breeding_started_at TIMESTAMP,
    breeding_ready_at TIMESTAMP,
    parent1_id BIGINT,
    parent2_id BIGINT,
    offspring_species VARCHAR(100),
    is_special_combo BOOLEAN NOT NULL DEFAULT ...,
    hatched_pal_id BIGINT,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    locked_for_order_id INTEGER,
    asset_id VARCHAR(128) NOT NULL,
    pending_gift_id CHAR(32)
);

-- Table: partner_client
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    name VARCHAR(128) NOT NULL,
    client_id CHAR(32) NOT NULL DEFAULT ...,
    client_type VARCHAR(32) NOT NULL DEFAULT ...,
    client_secret_encrypted VARCHAR(4096),
    status VARCHAR(32) NOT NULL DEFAULT ...,
    client_scopes JSONB NOT NULL,
    user_scopes JSONB NOT NULL,
    allowed_redirect_uris JSONB NOT NULL,
    webhook_url VARCHAR(2048),
    webhook_secret_encrypted VARCHAR(4096),
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL
);

-- Table: partner_managed_account
    id INTEGER PRIMARY KEY NOT NULL,
    owner_user_id VARCHAR(255) NOT NULL,
    managed_user_id VARCHAR(255) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT ...,
    can_login BOOLEAN NOT NULL DEFAULT ...,
    created_by_user_id VARCHAR(255),
    updated_by_user_id VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL
);

-- Table: partner_refresh_token
    id INTEGER PRIMARY KEY NOT NULL,
    token_id CHAR(32) NOT NULL DEFAULT ...,
    partner_user_id VARCHAR NOT NULL,
    client_id CHAR(32) NOT NULL,
    principal_id CHAR(32),
    token_hash VARCHAR(128) NOT NULL,
    scope VARCHAR(2048) NOT NULL DEFAULT ...,
    status VARCHAR(32) NOT NULL DEFAULT ...,
    expires_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP,
    rotated_from_token_id CHAR(32),
    revoked_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: payment_intent
    intent_id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    partner_user_id VARCHAR NOT NULL,
    partner_client_id CHAR(32) NOT NULL,
    user_id VARCHAR NOT NULL,
    operation VARCHAR(32) NOT NULL DEFAULT ...,
    account_code VARCHAR(255) NOT NULL,
    amount DECIMAL(38, 2) NOT NULL,
    asset_code VARCHAR(32) NOT NULL DEFAULT ...,
    title VARCHAR(255) NOT NULL,
    summary VARCHAR(2000) NOT NULL,
    partner_reference_id VARCHAR(255) NOT NULL,
    return_url VARCHAR(2048) NOT NULL,
    cancel_url VARCHAR(2048) NOT NULL,
    checkout_token CHAR(32) NOT NULL DEFAULT ...,
    status VARCHAR(64) NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    expires_at TIMESTAMP NOT NULL,
    authorized_at TIMESTAMP,
    completed_at TIMESTAMP,
    cancelled_at TIMESTAMP,
    error_code VARCHAR(64),
    error_message VARCHAR(2000)
);

-- Table: pending_gift
    gift_id CHAR(32) PRIMARY KEY NOT NULL,
    asset_id VARCHAR(128) PRIMARY KEY NOT NULL,
    asset_family VARCHAR(32) NOT NULL,
    from_user_id VARCHAR(100) NOT NULL,
    to_user_id VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    resolved_at TIMESTAMP
);

-- Table: poll_comment_reactions
    id CHAR(32) PRIMARY KEY NOT NULL,
    comment_id CHAR(32) NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    emoji VARCHAR(16) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: poll_comments
    id CHAR(32) PRIMARY KEY NOT NULL,
    poll_id CHAR(32) NOT NULL,
    author_id VARCHAR(100) NOT NULL,
    content VARCHAR(1000) NOT NULL,
    quote_comment_id CHAR(32),
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: poll_options
    id CHAR(32) PRIMARY KEY NOT NULL,
    poll_id CHAR(32) NOT NULL,
    position INTEGER NOT NULL,
    label VARCHAR(120) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: poll_votes
    id CHAR(32) PRIMARY KEY NOT NULL,
    poll_id CHAR(32) NOT NULL,
    option_id CHAR(32) NOT NULL,
    voter_user_id VARCHAR(100) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: polls
    id CHAR(32) PRIMARY KEY NOT NULL,
    scope VARCHAR(20) NOT NULL,
    room_id VARCHAR(100),
    creator_id VARCHAR(100) NOT NULL,
    title VARCHAR(120) NOT NULL,
    description VARCHAR(1000),
    vote_mode VARCHAR(20) NOT NULL,
    display_mode VARCHAR(20) NOT NULL,
    result_visibility VARCHAR(20) NOT NULL,
    max_choices INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    expires_at TIMESTAMP NOT NULL,
    closed_at TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: private_rooms
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    room_id VARCHAR NOT NULL,
    bot_user_id VARCHAR,
    invite_link VARCHAR,
    protected BOOLEAN NOT NULL DEFAULT ...,
    pending_message_id VARCHAR,
    pending_room_id VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: raid_action_log
    id BIGINT PRIMARY KEY NOT NULL,
    session_id VARCHAR(32) NOT NULL,
    seq INTEGER NOT NULL,
    action JSONB NOT NULL,
    effects JSONB NOT NULL DEFAULT ...,
    created_at TIMESTAMP PRIMARY KEY NOT NULL
);

-- Table: raid_map
    id BIGINT PRIMARY KEY NOT NULL,
    seed INTEGER NOT NULL,
    config JSONB NOT NULL DEFAULT ...,
    version INTEGER NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: raid_map_floor
    map_id BIGINT PRIMARY KEY NOT NULL,
    floor INTEGER PRIMARY KEY NOT NULL,
    tiles JSONB NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: raid_map_progress
    user_id VARCHAR(64) PRIMARY KEY NOT NULL,
    map_id BIGINT PRIMARY KEY NOT NULL,
    explored_rooms JSONB,
    activated_waystones JSONB,
    pressure INTEGER NOT NULL DEFAULT ...,
    run_count INTEGER NOT NULL DEFAULT ...,
    last_started_at TIMESTAMP,
    last_settled_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: raid_profile
    user_id VARCHAR(64) PRIMARY KEY NOT NULL,
    name VARCHAR(256) NOT NULL,
    str_stat INTEGER NOT NULL DEFAULT ...,
    dex INTEGER NOT NULL DEFAULT ...,
    wil INTEGER NOT NULL DEFAULT ...,
    per INTEGER NOT NULL DEFAULT ...,
    level INTEGER NOT NULL DEFAULT ...,
    experience INTEGER NOT NULL DEFAULT ...,
    max_willpower INTEGER NOT NULL DEFAULT ...,
    pending_attribute_points INTEGER NOT NULL DEFAULT ...,
    skill_search INTEGER NOT NULL DEFAULT ...,
    skill_combat INTEGER NOT NULL DEFAULT ...,
    skill_stealth INTEGER NOT NULL DEFAULT ...,
    skill_resist INTEGER NOT NULL DEFAULT ...,
    total_raids INTEGER NOT NULL DEFAULT ...,
    raids_survived INTEGER NOT NULL DEFAULT ...,
    total_loot_value DECIMAL(38, 0) NOT NULL DEFAULT ...,
    max_rooms_explored INTEGER NOT NULL DEFAULT ...,
    total_kills INTEGER NOT NULL DEFAULT ...,
    total_bosses_killed INTEGER NOT NULL DEFAULT ...,
    survival_streak INTEGER NOT NULL DEFAULT ...,
    best_survival_streak INTEGER NOT NULL DEFAULT ...,
    gifts_placed INTEGER NOT NULL DEFAULT ...,
    gifts_received INTEGER NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: raid_risk_control_state
    user_id VARCHAR(64) PRIMARY KEY NOT NULL,
    last_evaluated_at TIMESTAMP NOT NULL DEFAULT ...,
    precheck_band VARCHAR(16) NOT NULL DEFAULT ...,
    effective_shadow_band VARCHAR(16) NOT NULL DEFAULT ...,
    starts_6h INTEGER NOT NULL DEFAULT ...,
    starts_24h INTEGER NOT NULL DEFAULT ...,
    hot_30m_buckets_24h INTEGER NOT NULL DEFAULT ...,
    turnstile_passed_at TIMESTAMP,
    turnstile_exempt_nonce VARCHAR(128),
    signals_json JSONB NOT NULL DEFAULT ...
);

-- Table: raid_session
    id VARCHAR(32) PRIMARY KEY NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    map_id BIGINT,
    state_json JSONB NOT NULL,
    risk_snapshot_json JSONB,
    is_active BOOLEAN NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...,
    current_seq INTEGER NOT NULL DEFAULT ...,
    settled_player_action_count INTEGER,
    settled_action_interval_stddev_seconds DOUBLE PRECISION
);

-- Table: raid_warehouse_item
    item_id VARCHAR(128) PRIMARY KEY NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    template_id VARCHAR(64) NOT NULL,
    item_type VARCHAR(32) NOT NULL,
    equipped_slot VARCHAR(32),
    quality VARCHAR(32) NOT NULL DEFAULT ...,
    quantity INTEGER NOT NULL DEFAULT ...,
    item_data JSONB NOT NULL DEFAULT ...,
    market_locked_for_order_id BIGINT,
    pending_gift_id UUID,
    location VARCHAR(16) NOT NULL DEFAULT ...,
    carry_order INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: red_envelope
    id INTEGER PRIMARY KEY NOT NULL,
    sender_id VARCHAR NOT NULL,
    room_id VARCHAR NOT NULL,
    message_id VARCHAR NOT NULL,
    envelope_type VARCHAR NOT NULL,
    total_amount DECIMAL(38, 2) NOT NULL,
    remaining_amount DECIMAL(38, 2) NOT NULL,
    total_count INTEGER NOT NULL,
    remaining_count INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    expires_at TIMESTAMP NOT NULL DEFAULT ...,
    is_expired BOOLEAN NOT NULL DEFAULT ...
);

-- Table: red_envelope_claim
    id INTEGER PRIMARY KEY NOT NULL,
    envelope_id INTEGER NOT NULL,
    user_id VARCHAR NOT NULL,
    amount DECIMAL(38, 2) NOT NULL,
    claimed_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: resource_production
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    land_id BIGINT NOT NULL,
    accumulated_credits DECIMAL(38, 2) NOT NULL DEFAULT ...,
    last_tick_at TIMESTAMP NOT NULL DEFAULT ...,
    is_paused BOOLEAN NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: room_members
    room_id VARCHAR PRIMARY KEY NOT NULL,
    user_id VARCHAR PRIMARY KEY NOT NULL,
    role VARCHAR,
    joined_at TIMESTAMP,
    left_at TIMESTAMP,
    raw_data JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: rooms
    room_id VARCHAR PRIMARY KEY NOT NULL,
    title VARCHAR NOT NULL,
    chat_type VARCHAR,
    avatar_url VARCHAR,
    member_count INTEGER,
    tags ARRAY,
    is_public BOOLEAN,
    creator_id VARCHAR,
    account_ids ARRAY NOT NULL,
    last_message_at TIMESTAMP,
    first_message_at TIMESTAMP,
    backfill_until TIMESTAMP,
    history_complete BOOLEAN NOT NULL DEFAULT ...,
    message_count INTEGER NOT NULL DEFAULT ...,
    deleted_count INTEGER NOT NULL DEFAULT ...,
    recalled_count INTEGER NOT NULL DEFAULT ...,
    edited_count INTEGER NOT NULL DEFAULT ...,
    image_count INTEGER NOT NULL DEFAULT ...,
    is_active BOOLEAN NOT NULL DEFAULT ...,
    dissolved_at TIMESTAMP,
    raw_data JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: season_settlement
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    season INTEGER NOT NULL,
    rank INTEGER NOT NULL,
    cash_balance DECIMAL(38, 2) NOT NULL DEFAULT ...,
    escrow_total DECIMAL(38, 2) NOT NULL DEFAULT ...,
    resource_credits DECIMAL(38, 2) NOT NULL DEFAULT ...,
    turnip_value DECIMAL(38, 2) NOT NULL DEFAULT ...,
    excess_pal_value DECIMAL(38, 2) NOT NULL DEFAULT ...,
    land_value DECIMAL(38, 2) NOT NULL DEFAULT ...,
    tax_paid DECIMAL(38, 2) NOT NULL DEFAULT ...,
    total_settlement DECIMAL(38, 2) NOT NULL DEFAULT ...,
    starting_balance DECIMAL(38, 2) NOT NULL DEFAULT ...,
    pals_kept INTEGER NOT NULL DEFAULT ...,
    pals_settled INTEGER NOT NULL DEFAULT ...,
    lands_settled INTEGER NOT NULL DEFAULT ...,
    details JSONB,
    settled_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: security_audit_event
    event_id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    principal_id CHAR(32) NOT NULL,
    action VARCHAR(100) NOT NULL,
    result VARCHAR(32) NOT NULL,
    target_type VARCHAR(64) NOT NULL,
    target_id VARCHAR(255),
    error_code VARCHAR(64),
    metadata JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: shared_kv
    id INTEGER PRIMARY KEY NOT NULL,
    namespace VARCHAR NOT NULL,
    key VARCHAR NOT NULL,
    value JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP
);

-- Table: stock.consumer_cursor
    consumer_name VARCHAR(64) PRIMARY KEY NOT NULL,
    symbol VARCHAR(32) PRIMARY KEY NOT NULL,
    last_processed_candle_at TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: stock.finalized_candle
    symbol VARCHAR(32) PRIMARY KEY NOT NULL,
    candle_start TIMESTAMP PRIMARY KEY NOT NULL,
    open DECIMAL(38, 2) NOT NULL,
    high DECIMAL(38, 2) NOT NULL,
    low DECIMAL(38, 2) NOT NULL,
    close DECIMAL(38, 2) NOT NULL,
    volume DECIMAL(38, 2),
    source VARCHAR(32) NOT NULL DEFAULT ...,
    session_kind VARCHAR(16) NOT NULL,
    finalized_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: stock.producer_heartbeat
    producer_id VARCHAR(64) PRIMARY KEY NOT NULL,
    heartbeat_at TIMESTAMP NOT NULL DEFAULT ...,
    last_reconcile_started_at TIMESTAMP,
    last_reconcile_finished_at TIMESTAMP,
    mode VARCHAR(32),
    ws_connected BOOLEAN,
    ws_last_connected_at TIMESTAMP,
    ws_last_message_at TIMESTAMP,
    ws_subscription_count INTEGER,
    last_targeted_reconcile_started_at TIMESTAMP,
    last_targeted_reconcile_finished_at TIMESTAMP,
    last_targeted_reconcile_symbol_count INTEGER,
    last_gap_repair_started_at TIMESTAMP,
    last_gap_repair_finished_at TIMESTAMP,
    last_gap_repair_symbol_count INTEGER
);

-- Table: stock_account
    user_id VARCHAR PRIMARY KEY NOT NULL,
    total_realized_pnl DECIMAL(38, 2) NOT NULL DEFAULT ...,
    trade_count INTEGER NOT NULL DEFAULT ...,
    best_trade_pnl DECIMAL(38, 2) NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: stock_pending_order
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    idempotency_key VARCHAR(64) NOT NULL,
    symbol VARCHAR NOT NULL,
    action VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT ...,
    failure_reason VARCHAR(64),
    request_mode VARCHAR(16) NOT NULL,
    settlement_policy VARCHAR(48) NOT NULL DEFAULT ...,
    requested_shares DECIMAL(38, 6),
    requested_amount DECIMAL(38, 2),
    requested_leverage INTEGER NOT NULL,
    anchor_timestamp TIMESTAMP NOT NULL,
    anchor_market_minute_start TIMESTAMP NOT NULL,
    anchor_market_minute_end TIMESTAMP NOT NULL,
    settlement_deadline TIMESTAMP NOT NULL,
    acceptance_quote_price DECIMAL(38, 2) NOT NULL,
    acceptance_quote_time TIMESTAMP,
    acceptance_market_state VARCHAR(20) NOT NULL DEFAULT ...,
    acceptance_risk_snapshot_json JSONB,
    frozen_cash_amount DECIMAL(38, 2) NOT NULL DEFAULT ...,
    reserved_shares DECIMAL(38, 6) NOT NULL DEFAULT ...,
    settlement_price DECIMAL(38, 2),
    filled_shares DECIMAL(38, 6) NOT NULL DEFAULT ...,
    refunded_cash_amount DECIMAL(38, 2) NOT NULL DEFAULT ...,
    cancel_fee_cash_amount DECIMAL(38, 2),
    cancel_fee_shares DECIMAL(38, 6),
    settling_started_at TIMESTAMP,
    settlement_attempt_count INTEGER NOT NULL DEFAULT ...,
    settlement_worker_id VARCHAR(64),
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...,
    settled_at TIMESTAMP
);

-- Table: stock_portfolio
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    symbol VARCHAR NOT NULL,
    position_type VARCHAR(5) NOT NULL,
    shares DECIMAL(38, 6) NOT NULL,
    buy_price DECIMAL(38, 2) NOT NULL,
    leverage INTEGER NOT NULL DEFAULT ...,
    bought_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: stock_portfolio_adjustment
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    symbol VARCHAR NOT NULL,
    position_type VARCHAR(16) NOT NULL,
    adjustment_type VARCHAR(64) NOT NULL,
    shares_delta DECIMAL(38, 6) NOT NULL,
    pending_order_id INTEGER NOT NULL,
    metadata_json JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: stock_position_reservation
    id INTEGER PRIMARY KEY NOT NULL,
    pending_order_id INTEGER NOT NULL,
    user_id VARCHAR NOT NULL,
    symbol VARCHAR NOT NULL,
    position_type VARCHAR(16) NOT NULL,
    shares_reserved DECIMAL(38, 6) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    released_at TIMESTAMP
);

-- Table: stock_trade_history
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    pending_order_id INTEGER,
    symbol VARCHAR NOT NULL,
    action VARCHAR(11) NOT NULL,
    shares DECIMAL(38, 6) NOT NULL,
    price DECIMAL(38, 2) NOT NULL,
    leverage INTEGER NOT NULL DEFAULT ...,
    pnl DECIMAL(38, 2),
    executed_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: stock_trigger
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    symbol VARCHAR NOT NULL,
    position_type VARCHAR(5) NOT NULL,
    trigger_type VARCHAR(11) NOT NULL,
    trigger_price DECIMAL(38, 2) NOT NULL,
    shares DECIMAL(38, 6),
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: strand_object
    id BIGINT PRIMARY KEY NOT NULL,
    map_id BIGINT NOT NULL,
    type VARCHAR(16) NOT NULL,
    floor INTEGER NOT NULL,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    owner_user_id VARCHAR(64) NOT NULL,
    data JSONB NOT NULL,
    likes INTEGER NOT NULL DEFAULT ...,
    picked_up_by VARCHAR(64),
    expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: sudoku_puzzle
    id INTEGER PRIMARY KEY NOT NULL,
    room_id VARCHAR NOT NULL,
    creator_id VARCHAR NOT NULL,
    request_message_id VARCHAR NOT NULL,
    announcement_message_id VARCHAR,
    difficulty VARCHAR NOT NULL DEFAULT ...,
    puzzle VARCHAR NOT NULL,
    solution VARCHAR NOT NULL,
    status VARCHAR NOT NULL DEFAULT ...,
    solver_id VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    finished_at TIMESTAMP
);

-- Table: trpg_checkpoints
    id INTEGER PRIMARY KEY NOT NULL,
    room_id VARCHAR NOT NULL,
    game_id INTEGER,
    name VARCHAR NOT NULL,
    state JSONB NOT NULL,
    saved_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: trpg_games
    id INTEGER PRIMARY KEY NOT NULL,
    room_id VARCHAR NOT NULL,
    title VARCHAR NOT NULL DEFAULT ...,
    started_at TIMESTAMP NOT NULL DEFAULT ...,
    ended_at TIMESTAMP,
    background VARCHAR NOT NULL DEFAULT ...,
    mission VARCHAR NOT NULL DEFAULT ...,
    rule VARCHAR NOT NULL DEFAULT ...,
    scene VARCHAR NOT NULL DEFAULT ...,
    summary VARCHAR NOT NULL DEFAULT ...,
    summarized_turn_count INTEGER NOT NULL DEFAULT ...,
    turns JSONB NOT NULL,
    pcs JSONB NOT NULL,
    npcs JSONB NOT NULL,
    bags JSONB NOT NULL,
    bag_logs JSONB NOT NULL,
    undo_stack JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: turnip_inventory
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    quantity DECIMAL(38, 0) NOT NULL,
    buy_price DECIMAL(38, 2) NOT NULL,
    purchased_at TIMESTAMP NOT NULL DEFAULT ...,
    settles_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP,
    is_harvested BOOLEAN NOT NULL DEFAULT ...,
    is_stored BOOLEAN NOT NULL DEFAULT ...,
    stored_shelf_life_seconds DECIMAL(38, 2),
    locked_for_order_id INTEGER,
    market_locked_for_order_id INTEGER,
    pending_gift_id UUID
);

-- Table: turnip_market_event_composition
    id INTEGER PRIMARY KEY NOT NULL,
    payload_hash VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: turnip_market_event_instance
    id INTEGER PRIMARY KEY NOT NULL,
    event_version_id INTEGER NOT NULL,
    starts_at TIMESTAMP NOT NULL,
    ends_at TIMESTAMP NOT NULL,
    priority INTEGER NOT NULL DEFAULT ...,
    weight NUMERIC NOT NULL DEFAULT ...,
    paused BOOLEAN NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: turnip_market_event_version
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR(120) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    created_by VARCHAR,
    note VARCHAR,
    parent_version_id INTEGER
);

-- Table: turnip_market_guardrail_policy_version
    id INTEGER PRIMARY KEY NOT NULL,
    mode VARCHAR(32) NOT NULL,
    oracle_band_pct NUMERIC NOT NULL DEFAULT ...,
    admission_band_pct NUMERIC NOT NULL DEFAULT ...,
    execution_hard_band_pct NUMERIC NOT NULL DEFAULT ...,
    max_limit_order_notional_vs_nav_ratio NUMERIC NOT NULL DEFAULT ...,
    max_market_order_notional_vs_turnover_ratio NUMERIC NOT NULL DEFAULT ...,
    max_taking_order_quantity_vs_visible_depth_ratio NUMERIC NOT NULL DEFAULT ...,
    sink_quote_budget_cash_ratio NUMERIC NOT NULL DEFAULT ...,
    source_quote_budget_inventory_ratio NUMERIC NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    created_by VARCHAR,
    note VARCHAR,
    parent_version_id INTEGER
);

-- Table: turnip_market_snapshot
    id INTEGER PRIMARY KEY NOT NULL,
    prev_snapshot_id INTEGER,
    guardrail_policy_version_id INTEGER,
    config_version_id INTEGER,
    event_composition_id INTEGER,
    active_overlay_revision_id INTEGER,
    last_trade_price DECIMAL(38, 2) NOT NULL,
    last_raw_trade_price DECIMAL(38, 2),
    last_qualified_trade_price DECIMAL(38, 2),
    qualified_fill_seen_tick BOOLEAN NOT NULL DEFAULT ...,
    ema_fair_value DECIMAL(38, 2) NOT NULL,
    best_bid DECIMAL(38, 2),
    best_ask DECIMAL(38, 2),
    bid_depth VARCHAR(4096) NOT NULL,
    ask_depth VARCHAR(4096) NOT NULL,
    spread DECIMAL(38, 2) NOT NULL,
    treasury_cash_balance DECIMAL(38, 2) NOT NULL,
    treasury_turnip_balance DECIMAL(38, 0) NOT NULL,
    tick_number INTEGER NOT NULL,
    sub_tick_number INTEGER NOT NULL,
    farm_output_raw DECIMAL(38, 0),
    farm_output_ema DECIMAL(38, 2),
    raw_turnip_allocation_shadow DECIMAL(38, 0),
    turnip_allocation_total DECIMAL(38, 0),
    reference_price DECIMAL(38, 2),
    oracle_mode VARCHAR(32),
    guardrail_mode VARCHAR(32),
    oracle_guardrail_hit BOOLEAN,
    admission_guardrail_hit BOOLEAN,
    execution_guardrail_hit BOOLEAN,
    live_sink_bid_bias_pct DECIMAL(38, 6),
    live_source_ask_bias_pct DECIMAL(38, 6),
    live_sink_budget_multiplier DECIMAL(38, 6),
    clamp_flags_json JSONB,
    treasury_imbalance DECIMAL(38, 6),
    policy_sell_rate DECIMAL(38, 6),
    policy_buy_rate DECIMAL(38, 6),
    noise_drift DECIMAL(38, 6),
    noise_regime_raw_signal DECIMAL(38, 6),
    noise_vol DECIMAL(38, 6),
    noise_jump DECIMAL(38, 6),
    noise_jump_residual DECIMAL(38, 6),
    scenario_supply_mult DECIMAL(38, 6),
    scenario_demand_mult DECIMAL(38, 6),
    scenario_npc_direction DECIMAL(38, 6),
    noise_seed_hash VARCHAR(64),
    noise_exec_drift DECIMAL(38, 6),
    noise_exec_state DECIMAL(38, 6),
    noise_anchor_price DECIMAL(38, 6),
    noise_price_gap DECIMAL(38, 6),
    noise_delta_regime DECIMAL(38, 6),
    scenario_regime_bias DECIMAL(38, 6),
    effective_regime DECIMAL(38, 6),
    support_slow_reference_anchor DECIMAL(38, 6),
    support_bucket_started_at TIMESTAMP,
    support_slow_discount DECIMAL(38, 6),
    support_sink_capacity_turnips DECIMAL(38, 6),
    support_sink_quota_base_notional DECIMAL(38, 6),
    support_flow_pressure DECIMAL(38, 6),
    support_user_taker_sell_into_sink_notional DECIMAL(38, 6),
    support_flow_pressure_ema DECIMAL(38, 6),
    support_feedback_target_ratio DECIMAL(38, 6),
    support_actual_support_ratio DECIMAL(38, 6),
    support_gap DECIMAL(38, 6),
    support_gap_ema DECIMAL(38, 6),
    support_pressure_integral DECIMAL(38, 6),
    support_bid_bias_boost_raw DECIMAL(38, 6),
    support_bid_bias_boost DECIMAL(38, 6),
    support_budget_boost_raw DECIMAL(38, 6),
    support_budget_boost DECIMAL(38, 6),
    support_integral_decay_mode VARCHAR(32),
    raw_sink_bid_bias_pct DECIMAL(38, 6),
    raw_source_ask_bias_pct DECIMAL(38, 6),
    raw_sink_budget_multiplier DECIMAL(38, 6),
    smoothed_drift_bias DECIMAL(38, 6),
    smoothed_vol_bias DECIMAL(38, 6),
    source_quota DECIMAL(38, 0),
    sink_quota DECIMAL(38, 2),
    turnip_injection DECIMAL(38, 0),
    cash_injection DECIMAL(38, 2),
    scenario_npc_activity_mult DECIMAL(38, 6),
    scenario_npc_spread_tolerance DECIMAL(38, 6),
    scenario_random_trader_boost DECIMAL(38, 6),
    scenario_cash_alloc_drift DECIMAL(38, 6),
    scenario_panic_severity DECIMAL(38, 6),
    scenario_euphoria_severity DECIMAL(38, 6),
    npc_reference_notional DECIMAL(38, 2),
    applied_jump_source VARCHAR(32),
    applied_jump_kind VARCHAR(32),
    applied_jump_size DECIMAL(38, 6),
    applied_jump_persistence_ratio DECIMAL(38, 6),
    applied_jump_persistent_shift DECIMAL(38, 6),
    applied_jump_transient_shift DECIMAL(38, 6),
    applied_jump_oracle_anchor_before DECIMAL(38, 2),
    applied_jump_oracle_anchor_after DECIMAL(38, 2),
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: turnip_order
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    order_type VARCHAR(10) NOT NULL DEFAULT ...,
    side VARCHAR(10) NOT NULL,
    quantity DECIMAL(38, 0) NOT NULL,
    filled_quantity DECIMAL(38, 0) NOT NULL DEFAULT ...,
    limit_price DECIMAL(38, 2) NOT NULL,
    escrow_amount DECIMAL(38, 2) NOT NULL DEFAULT ...,
    quote_price DECIMAL(38, 2),
    execution_price DECIMAL(38, 2),
    slippage_pct FLOAT,
    status VARCHAR(20) NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    filled_at TIMESTAMP,
    cancelled_at TIMESTAMP,
    expires_at TIMESTAMP NOT NULL
);

-- Table: turnip_order_fill
    id INTEGER PRIMARY KEY NOT NULL,
    order_id INTEGER NOT NULL,
    fill_type VARCHAR(10) NOT NULL,
    counterparty_order_id INTEGER,
    quantity DECIMAL(38, 0) NOT NULL,
    price DECIMAL(38, 2) NOT NULL,
    total DECIMAL(38, 2) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: turnip_price
    id INTEGER PRIMARY KEY NOT NULL,
    price DECIMAL(38, 2) NOT NULL,
    open DECIMAL(38, 2),
    high DECIMAL(38, 2),
    low DECIMAL(38, 2),
    volume DECIMAL(38, 0) NOT NULL DEFAULT ...,
    trade_count DECIMAL(38, 0) NOT NULL DEFAULT ...,
    trend VARCHAR(20) NOT NULL,
    trend_tick INTEGER NOT NULL,
    base_price DECIMAL(38, 2) NOT NULL,
    cycle_ticks INTEGER,
    cycle_context JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: turnip_scenario_overlay_revision
    id INTEGER PRIMARY KEY NOT NULL,
    stage_id INTEGER NOT NULL,
    reason VARCHAR(32) NOT NULL,
    target_now DECIMAL(38, 6) NOT NULL,
    observed_price DECIMAL(38, 2) NOT NULL,
    observed_fv DECIMAL(38, 2) NOT NULL,
    observed_reference_price DECIMAL(38, 2) NOT NULL,
    error DECIMAL(38, 6) NOT NULL,
    runtime_patch_json JSONB NOT NULL,
    event_patch_json JSONB NOT NULL,
    scenario_patch_json JSONB NOT NULL DEFAULT ...,
    entry_patch_json JSONB NOT NULL DEFAULT ...,
    neutralize_progress_ratio DECIMAL(38, 6),
    effective_overlay_hash VARCHAR(64) NOT NULL,
    supervisor_version VARCHAR(120) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: turnip_scenario_run
    id INTEGER PRIMARY KEY NOT NULL,
    template_id INTEGER,
    template_snapshot_json JSONB NOT NULL,
    status VARCHAR(16) NOT NULL,
    start_mode VARCHAR(16) NOT NULL,
    scheduled_at TIMESTAMP,
    started_at TIMESTAMP,
    current_stage_started_at TIMESTAMP,
    paused_at TIMESTAMP,
    ended_at TIMESTAMP,
    baseline_config_version_id INTEGER,
    current_stage_index INTEGER NOT NULL DEFAULT ...,
    heartbeat_interval_sec INTEGER NOT NULL DEFAULT ...,
    last_heartbeat_at TIMESTAMP,
    lease_expires_at TIMESTAMP,
    created_by VARCHAR(100),
    abort_reason VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: turnip_scenario_stage
    id INTEGER PRIMARY KEY NOT NULL,
    run_id INTEGER NOT NULL,
    stage_index INTEGER NOT NULL,
    name VARCHAR(120) NOT NULL,
    stage_type VARCHAR(32) NOT NULL,
    mode VARCHAR(16) NOT NULL,
    duration_sec INTEGER NOT NULL,
    regime_target DECIMAL(38, 6),
    target_ref VARCHAR(32),
    target_value DECIMAL(38, 6),
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: turnip_scenario_template
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR(120) NOT NULL,
    description VARCHAR,
    default_start_mode VARCHAR(16) NOT NULL,
    stage_definition_json JSONB NOT NULL,
    created_by VARCHAR(100),
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...,
    deleted_at TIMESTAMP
);

-- Table: turnip_seed
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    quantity DECIMAL(38, 0) NOT NULL,
    seed_price DECIMAL(38, 2) NOT NULL,
    planted_at TIMESTAMP NOT NULL DEFAULT ...,
    fertilize_count INTEGER NOT NULL DEFAULT ...,
    fertilize_cost DECIMAL(38, 2) NOT NULL DEFAULT ...,
    weather_score DECIMAL(38, 6) NOT NULL DEFAULT ...,
    growth_required_hours DECIMAL(38, 6) NOT NULL DEFAULT ...,
    growth_progress_hours DECIMAL(38, 6) NOT NULL DEFAULT ...,
    last_growth_accounted_at TIMESTAMP NOT NULL DEFAULT ...,
    latest_effective_time_bonus DECIMAL(38, 6) NOT NULL DEFAULT ...,
    latest_effective_harvest_bonus DECIMAL(38, 6) NOT NULL DEFAULT ...,
    locked_weather_yield_factor DECIMAL(38, 6) NOT NULL DEFAULT ...,
    pal_harvest_bonus_score DECIMAL(38, 6) NOT NULL DEFAULT ...,
    locked_pal_harvest_bonus DECIMAL(38, 6) NOT NULL DEFAULT ...,
    pal_harvest_bonus_locked BOOLEAN NOT NULL DEFAULT ...,
    batch_yield_factor DECIMAL(38, 6) NOT NULL DEFAULT ...,
    status VARCHAR(20) NOT NULL DEFAULT ...,
    harvested_at TIMESTAMP,
    matured_at TIMESTAMP
);

-- Table: turnip_trade
    id INTEGER PRIMARY KEY NOT NULL,
    side VARCHAR(10) NOT NULL,
    quantity DECIMAL(38, 0) NOT NULL,
    price DECIMAL(38, 2) NOT NULL,
    total DECIMAL(38, 2) NOT NULL,
    maker_actor VARCHAR(100) NOT NULL,
    taker_actor VARCHAR(100) NOT NULL,
    maker_order_id INTEGER,
    taker_order_id INTEGER,
    trade_type VARCHAR(16) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: turnip_transaction
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    quantity DECIMAL(38, 0) NOT NULL,
    balance_after DECIMAL(38, 0) NOT NULL,
    tx_type VARCHAR(20) NOT NULL,
    unit_price DECIMAL(38, 2) NOT NULL,
    description VARCHAR(200) NOT NULL,
    mid_price DECIMAL(38, 2),
    inventory_ids ARRAY,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: tweets
    tweet_id VARCHAR PRIMARY KEY NOT NULL,
    user_id VARCHAR,
    content VARCHAR,
    media_urls ARRAY,
    local_media_paths ARRAY,
    source VARCHAR,
    tweet_type VARCHAR,
    parent_tweet_id VARCHAR,
    reply_to_tweet_id VARCHAR,
    reply_to_username VARCHAR,
    is_edited BOOLEAN NOT NULL DEFAULT ...,
    edit_history JSONB,
    post_id VARCHAR,
    draw_id VARCHAR,
    likes_count INTEGER NOT NULL DEFAULT ...,
    comments_count INTEGER NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP,
    fetched_at TIMESTAMP NOT NULL DEFAULT ...,
    is_deleted BOOLEAN NOT NULL DEFAULT ...,
    raw_data JSONB
);

-- Table: undercover_event
    id INTEGER PRIMARY KEY NOT NULL,
    session_id UUID NOT NULL,
    seq INTEGER NOT NULL,
    event_type VARCHAR(64) NOT NULL,
    actor_user_id VARCHAR(128),
    public_payload JSONB NOT NULL,
    private_payload JSONB NOT NULL,
    public_message_status VARCHAR(16) NOT NULL DEFAULT ...,
    public_message_attempts INTEGER NOT NULL DEFAULT ...,
    public_message_sent_at TIMESTAMP,
    public_message_error TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: undercover_session
    id CHAR(32) PRIMARY KEY NOT NULL,
    room_id VARCHAR NOT NULL,
    creator_id VARCHAR NOT NULL,
    status VARCHAR NOT NULL DEFAULT ...,
    phase VARCHAR NOT NULL DEFAULT ...,
    config JSONB NOT NULL,
    state_payload JSONB NOT NULL,
    phase_deadline_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    started_at TIMESTAMP,
    finished_at TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT ...,
    last_activity_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: undercover_word_pair
    id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    word_a VARCHAR(32) NOT NULL,
    word_b VARCHAR(32) NOT NULL,
    canonical_word_a VARCHAR(128) NOT NULL,
    canonical_word_b VARCHAR(128) NOT NULL,
    submitter_user_id VARCHAR NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: user_achievement
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    achievement_key VARCHAR(100) NOT NULL,
    unlocked_at TIMESTAMP NOT NULL DEFAULT ...,
    context JSONB
);

-- Table: user_achievement_progress
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    stat_key VARCHAR(100) NOT NULL,
    value DECIMAL(38, 0) NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: user_credential
    id INTEGER PRIMARY KEY NOT NULL,
    username VARCHAR(64) NOT NULL,
    password_hash VARCHAR(256) NOT NULL DEFAULT ...,
    user_id VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    last_login TIMESTAMP
);

-- Table: user_history
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    full_name VARCHAR,
    avatar_url VARCHAR,
    avatar_file VARCHAR,
    bio VARCHAR,
    birthday VARCHAR,
    birthday_public BOOLEAN,
    quirk VARCHAR,
    is_bot BOOLEAN,
    gender VARCHAR,
    metadata JSONB,
    raw_data JSONB,
    recorded_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: user_item
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    item_type VARCHAR(50) NOT NULL,
    display_name VARCHAR(200),
    quantity INTEGER NOT NULL DEFAULT ...,
    purchased_at TIMESTAMP NOT NULL DEFAULT ...,
    activated_at TIMESTAMP,
    expires_at TIMESTAMP,
    price_paid DECIMAL(38, 2) NOT NULL
);

-- Table: user_notification
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    category VARCHAR NOT NULL,
    content VARCHAR NOT NULL,
    reference_id VARCHAR(255),
    is_read BOOLEAN NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: user_passkey
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    name VARCHAR(128) NOT NULL DEFAULT ...,
    credential_id VARCHAR(512) NOT NULL,
    public_key VARCHAR(4096) NOT NULL,
    user_handle VARCHAR(128) NOT NULL,
    sign_count INTEGER NOT NULL DEFAULT ...,
    transports JSONB NOT NULL,
    backup_state VARCHAR(32),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP
);

-- Table: users
    user_id VARCHAR PRIMARY KEY NOT NULL,
    full_name VARCHAR,
    name_tsv TSVECTOR,
    avatar_url VARCHAR,
    avatar_file VARCHAR,
    bio VARCHAR,
    birthday VARCHAR,
    birthday_public BOOLEAN,
    quirk VARCHAR,
    is_bot BOOLEAN,
    gender VARCHAR,
    metadata JSONB,
    raw_data JSONB,
    last_seen TIMESTAMP,
    message_count INTEGER NOT NULL DEFAULT ...,
    deleted_count INTEGER NOT NULL DEFAULT ...,
    recalled_count INTEGER NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    updated_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: wallet
    user_id VARCHAR PRIMARY KEY NOT NULL,
    allow_negative_balance BOOLEAN NOT NULL DEFAULT ...,
    snapshot_balance DECIMAL(38, 2) NOT NULL DEFAULT ...,
    snapshot_escrow_balance DECIMAL(38, 2) NOT NULL DEFAULT ...,
    snapshot_tx_id BIGINT NOT NULL DEFAULT ...,
    last_daily_credit DATE,
    total_credited DECIMAL(38, 2) NOT NULL DEFAULT ...,
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: wallet_transaction
    id INTEGER PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    amount DECIMAL(38, 2) NOT NULL,
    escrow_delta DECIMAL(38, 2) NOT NULL DEFAULT ...,
    balance_after DECIMAL(38, 2),
    tx_type VARCHAR(50) NOT NULL,
    description VARCHAR(200) NOT NULL,
    reference_id VARCHAR(100),
    memo VARCHAR(200),
    counterparty_id VARCHAR(100) NOT NULL,
    tx_group_id VARCHAR(100) NOT NULL,
    principal_id CHAR(32),
    metadata JSONB,
    escrow_after DECIMAL(38, 2),
    created_at TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: webhook_delivery
    event_id CHAR(32) PRIMARY KEY NOT NULL DEFAULT ...,
    partner_user_id VARCHAR NOT NULL,
    partner_client_id CHAR(32) NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    resource_type VARCHAR(64) NOT NULL,
    resource_id VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT ...,
    qstash_message_id VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT ...,
    sent_at TIMESTAMP,
    delivered_at TIMESTAMP,
    dead_at TIMESTAMP,
    last_error VARCHAR(2000)
);

-- Table: websocket_connections
    lock_id BIGINT PRIMARY KEY NOT NULL,
    account_user_id VARCHAR NOT NULL,
    connected_at TIMESTAMP NOT NULL DEFAULT ...,
    last_heartbeat TIMESTAMP NOT NULL DEFAULT ...
);

-- Table: websocket_events
    id BIGINT PRIMARY KEY NOT NULL,
    timestamp TIMESTAMP PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL,
    event VARCHAR NOT NULL,
    data JSONB NOT NULL
);

