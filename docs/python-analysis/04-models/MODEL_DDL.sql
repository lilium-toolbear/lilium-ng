
CREATE TABLE arena_session (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	mode VARCHAR NOT NULL, 
	status VARCHAR NOT NULL, 
	current_round INTEGER NOT NULL, 
	turn_no INTEGER NOT NULL, 
	round_seed INTEGER NOT NULL, 
	state_json JSONB NOT NULL, 
	result_summary JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	ended_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE auth_session (
	id SERIAL NOT NULL, 
	session_id VARCHAR(64) NOT NULL, 
	code VARCHAR(16) NOT NULL, 
	user_id VARCHAR, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	authenticated_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	UNIQUE (session_id), 
	UNIQUE (code), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE battle_records (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	mode VARCHAR(10) NOT NULL, 
	rounds_cleared INTEGER NOT NULL, 
	pal_ids JSONB DEFAULT '[]' NOT NULL, 
	pal_levels JSONB DEFAULT '[]' NOT NULL, 
	exp_earned INTEGER NOT NULL, 
	credits_earned INTEGER NOT NULL, 
	battle_log JSONB DEFAULT '{}' NOT NULL, 
	active_state JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE blackjack_sessions (
	id VARCHAR NOT NULL, 
	user_id VARCHAR, 
	room_id VARCHAR, 
	status VARCHAR NOT NULL, 
	state_json JSONB NOT NULL, 
	result_json JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	finished_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE books (
	book_id VARCHAR NOT NULL, 
	title VARCHAR, 
	description TEXT, 
	slug VARCHAR, 
	is_nsfw BOOLEAN NOT NULL, 
	is_public BOOLEAN NOT NULL, 
	cover_image_url VARCHAR, 
	local_cover_path VARCHAR, 
	user_id VARCHAR, 
	author JSONB, 
	chapter_count INTEGER NOT NULL, 
	total_word_count INTEGER NOT NULL, 
	latest_chapter JSONB, 
	likes_count INTEGER NOT NULL, 
	comments_count INTEGER NOT NULL, 
	top_comments JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE, 
	published_at TIMESTAMP WITH TIME ZONE, 
	fetched_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	raw_data JSONB, 
	PRIMARY KEY (book_id)
)


;


CREATE TABLE bot_memory (
	id SERIAL NOT NULL, 
	namespace VARCHAR NOT NULL, 
	room_id VARCHAR, 
	user_id VARCHAR, 
	key VARCHAR NOT NULL, 
	value JSONB NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id)
)


;


CREATE TABLE cards (
	card_id SERIAL NOT NULL, 
	name VARCHAR, 
	card_filename VARCHAR, 
	original_filename VARCHAR, 
	creator VARCHAR, 
	creator_notes TEXT, 
	user_id VARCHAR, 
	creator_full_name VARCHAR, 
	creator_avatar_url VARCHAR, 
	tags TEXT[], 
	is_public BOOLEAN NOT NULL, 
	is_sensitive BOOLEAN NOT NULL, 
	is_image_blur BOOLEAN NOT NULL, 
	is_gamefy BOOLEAN NOT NULL, 
	image_info JSONB, 
	weighted_rating VARCHAR, 
	popularity_score VARCHAR, 
	likes_count INTEGER NOT NULL, 
	comments_count INTEGER NOT NULL, 
	top_comments JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	published_at TIMESTAMP WITH TIME ZONE, 
	fetched_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	raw_data JSONB, 
	PRIMARY KEY (card_id)
)


;


CREATE TABLE chapters (
	chapter_id VARCHAR NOT NULL, 
	title VARCHAR, 
	content TEXT, 
	is_adult BOOLEAN NOT NULL, 
	is_nsfw BOOLEAN NOT NULL, 
	user_id VARCHAR, 
	author JSONB, 
	likes_count INTEGER NOT NULL, 
	comments_count INTEGER NOT NULL, 
	top_comments JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE, 
	published_at TIMESTAMP WITH TIME ZONE, 
	fetched_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	raw_data JSONB, 
	PRIMARY KEY (chapter_id)
)


;


CREATE TABLE checkpoints (
	checkpoint_id VARCHAR NOT NULL, 
	name VARCHAR, 
	description TEXT, 
	is_public BOOLEAN NOT NULL, 
	user_id VARCHAR, 
	user_name VARCHAR, 
	user_avatar_url VARCHAR, 
	creator JSONB, 
	rating_avg VARCHAR, 
	rating_count INTEGER NOT NULL, 
	review_status VARCHAR, 
	share_code VARCHAR, 
	character_cards JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE, 
	fetched_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	raw_data JSONB, 
	PRIMARY KEY (checkpoint_id)
)


;


CREATE TABLE clearing_batch (
	batch_id UUID NOT NULL, 
	partner_user_id VARCHAR NOT NULL, 
	partner_client_id UUID NOT NULL, 
	operation VARCHAR(64) NOT NULL, 
	account_code VARCHAR(255) NOT NULL, 
	asset_code VARCHAR(32) NOT NULL, 
	batch_reference_id VARCHAR(255) NOT NULL, 
	status VARCHAR(64) NOT NULL, 
	item_count INTEGER NOT NULL, 
	success_count INTEGER NOT NULL, 
	failed_count INTEGER NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	completed_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (batch_id), 
	FOREIGN KEY(partner_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE clearing_batch_item (
	batch_item_id UUID NOT NULL, 
	batch_id UUID NOT NULL, 
	intent_id UUID, 
	user_id VARCHAR NOT NULL, 
	amount DECIMAL(38, 2), 
	partner_reference_id VARCHAR(255) NOT NULL, 
	note VARCHAR(2000), 
	status VARCHAR(64) NOT NULL, 
	error_code VARCHAR(64), 
	error_message VARCHAR(2000), 
	PRIMARY KEY (batch_item_id), 
	FOREIGN KEY(batch_id) REFERENCES clearing_batch (batch_id), 
	FOREIGN KEY(intent_id) REFERENCES payment_intent (intent_id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE clearing_instruction (
	instruction_id UUID NOT NULL, 
	partner_user_id VARCHAR NOT NULL, 
	partner_client_id UUID NOT NULL, 
	intent_id UUID, 
	user_id VARCHAR NOT NULL, 
	operation VARCHAR(64) NOT NULL, 
	account_code VARCHAR(255) NOT NULL, 
	amount DECIMAL(38, 2), 
	asset_code VARCHAR(32) NOT NULL, 
	partner_reference_id VARCHAR(255) NOT NULL, 
	reason VARCHAR(255) NOT NULL, 
	note VARCHAR(2000), 
	status VARCHAR(64) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	reverse_of_instruction_id UUID, 
	executed_at TIMESTAMP WITH TIME ZONE, 
	error_code VARCHAR(64), 
	error_message VARCHAR(2000), 
	PRIMARY KEY (instruction_id), 
	FOREIGN KEY(partner_user_id) REFERENCES users (user_id), 
	FOREIGN KEY(intent_id) REFERENCES payment_intent (intent_id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE crop_insurance_policy (
	id SERIAL NOT NULL, 
	seed_id INTEGER NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	mode VARCHAR(20) NOT NULL, 
	status VARCHAR(20) NOT NULL, 
	strike DECIMAL(38, 2) NOT NULL, 
	coverage_qty DECIMAL(38, 0) NOT NULL, 
	premium_paid DECIMAL(38, 2) NOT NULL, 
	spot_at_purchase DECIMAL(38, 2) NOT NULL, 
	coverage_level DECIMAL(38, 6) NOT NULL, 
	purchased_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	settlement_twap DECIMAL(38, 2), 
	payout DECIMAL(38, 2), 
	settled_at TIMESTAMP WITH TIME ZONE, 
	cancelled_at TIMESTAMP WITH TIME ZONE, 
	cancel_refund DECIMAL(38, 2), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(seed_id) REFERENCES turnip_seed (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE currency_exchange_order (
	id UUID NOT NULL, 
	user_id VARCHAR(64) NOT NULL, 
	currency VARCHAR(32) NOT NULL, 
	direction VARCHAR(24) NOT NULL, 
	input_amount DECIMAL(38, 8) NOT NULL, 
	output_amount DECIMAL(38, 8) NOT NULL, 
	rate DECIMAL(38, 8) NOT NULL, 
	fee_amount DECIMAL(38, 8) NOT NULL, 
	wallet_delta DECIMAL(38, 2) NOT NULL, 
	game_delta DECIMAL(38, 6) NOT NULL, 
	provider VARCHAR(32) NOT NULL, 
	provider_revision INTEGER NOT NULL, 
	wallet_inflation DECIMAL(38, 8) NOT NULL, 
	game_inflation DECIMAL(38, 8) NOT NULL, 
	status VARCHAR(16) NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE, 
	idempotency_key VARCHAR(64), 
	executed_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE currency_rate_state (
	currency VARCHAR(32) NOT NULL, 
	current_rate DECIMAL(38, 8) NOT NULL, 
	revision INTEGER NOT NULL, 
	baseline_wallet_supply DECIMAL(38, 2) NOT NULL, 
	baseline_game_supply DECIMAL(38, 6) NOT NULL, 
	last_wallet_supply DECIMAL(38, 2) NOT NULL, 
	last_game_supply DECIMAL(38, 6) NOT NULL, 
	macro_anchor_rate DECIMAL(38, 8) NOT NULL, 
	macro_anchor_updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	short_term_premium DECIMAL(38, 8) NOT NULL, 
	bank_backstop_hour_start TIMESTAMP WITH TIME ZONE NOT NULL, 
	bank_backstop_hour_used DECIMAL(38, 2) NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (currency)
)


;


CREATE TABLE discord_account_binding (
	id SERIAL NOT NULL, 
	discord_user_id VARCHAR(32) NOT NULL, 
	user_id VARCHAR(255) NOT NULL, 
	discord_username VARCHAR(128), 
	discord_global_name VARCHAR(128), 
	discord_avatar_hash VARCHAR(128), 
	discord_avatar_url VARCHAR(512), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	last_login_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE dzmm_account (
	user_id VARCHAR NOT NULL, 
	user_profile JSONB NOT NULL, 
	email VARCHAR, 
	password VARCHAR, 
	signin_code VARCHAR, 
	signin_code_image BYTEA, 
	signin_code_image_mime VARCHAR, 
	cookies VARCHAR, 
	is_enabled BOOLEAN NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (user_id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE economy_config_version (
	id SERIAL NOT NULL, 
	schema_version INTEGER NOT NULL, 
	config_payload JSONB NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	created_by VARCHAR, 
	note VARCHAR, 
	parent_version_id INTEGER, 
	PRIMARY KEY (id), 
	FOREIGN KEY(parent_version_id) REFERENCES economy_config_version (id)
)


;


CREATE TABLE economy_control_plane_state (
	id INTEGER NOT NULL, 
	active_config_version_id INTEGER, 
	active_guardrail_policy_version_id INTEGER, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(active_config_version_id) REFERENCES economy_config_version (id), 
	FOREIGN KEY(active_guardrail_policy_version_id) REFERENCES turnip_market_guardrail_policy_version (id)
)


;


CREATE TABLE economy_control_state (
	id INTEGER NOT NULL, 
	seed_weight DECIMAL(38, 6) NOT NULL, 
	inventory_conservation_enabled BOOLEAN NOT NULL, 
	amm_trade_fee_bps INTEGER NOT NULL, 
	cash_injection_multiplier DECIMAL(38, 6) NOT NULL, 
	treasury_policy_enabled BOOLEAN NOT NULL, 
	treasury_policy_k DECIMAL(38, 6), 
	treasury_policy_beta DECIMAL(38, 6), 
	stored_decay_bps_per_day INTEGER NOT NULL, 
	turnip_runtime_overrides JSONB, 
	coverage_ratio DECIMAL(38, 8) NOT NULL, 
	coverage_ema DECIMAL(38, 8) NOT NULL, 
	bank_liability DECIMAL(38, 2) NOT NULL, 
	runtime_metrics_updated_at TIMESTAMP WITH TIME ZONE, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	futures_payout_bucket_level DECIMAL(38, 2) NOT NULL, 
	futures_payout_bucket_max DECIMAL(38, 2) NOT NULL, 
	futures_payout_bucket_refill_rate DECIMAL(38, 2) NOT NULL, 
	last_refill_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id)
)


;


CREATE TABLE economy_overview_component_snapshot (
	component_key VARCHAR(64) NOT NULL, 
	schema_version INTEGER NOT NULL, 
	payload JSONB, 
	generated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	refresh_started_at TIMESTAMP WITH TIME ZONE, 
	refresh_owner VARCHAR(128), 
	refresh_duration_ms INTEGER, 
	refresh_error VARCHAR(1000), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (component_key)
)


;


CREATE TABLE economy_snapshot (
	id SERIAL NOT NULL, 
	config_version_id INTEGER, 
	liquidity DECIMAL(38, 2) NOT NULL, 
	effective_supply DECIMAL(38, 2) NOT NULL, 
	coverage_ratio DECIMAL(38, 8) NOT NULL, 
	bank_liability DECIMAL(38, 2) NOT NULL, 
	captured_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	CONSTRAINT uq_economy_snapshot_captured_at UNIQUE (captured_at), 
	FOREIGN KEY(config_version_id) REFERENCES economy_config_version (id)
)


;


CREATE TABLE event_processor_offsets (
	processor_id VARCHAR NOT NULL, 
	last_processed_id INTEGER NOT NULL, 
	last_processed_timestamp TIMESTAMP WITH TIME ZONE, 
	last_processed_at TIMESTAMP WITH TIME ZONE, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (processor_id)
)


;


CREATE TABLE external_command_effect (
	id SERIAL NOT NULL, 
	invocation_id VARCHAR(128) NOT NULL, 
	effect_index INTEGER NOT NULL, 
	effect_id VARCHAR(128) NOT NULL, 
	effect_type VARCHAR(64) NOT NULL, 
	status VARCHAR(64) NOT NULL, 
	executed_at TIMESTAMP WITH TIME ZONE, 
	error_code VARCHAR(64), 
	error_message VARCHAR(1024), 
	PRIMARY KEY (id), 
	FOREIGN KEY(invocation_id) REFERENCES external_command_invocation (invocation_id) ON DELETE CASCADE
)


;


CREATE TABLE external_command_invocation (
	invocation_id VARCHAR(128) NOT NULL, 
	config_id VARCHAR(128) NOT NULL, 
	command_name VARCHAR(256) NOT NULL, 
	matched_name VARCHAR(256) NOT NULL, 
	room_id VARCHAR(128) NOT NULL, 
	sender_id VARCHAR(128) NOT NULL, 
	message_id VARCHAR(128) NOT NULL, 
	status VARCHAR(64) NOT NULL, 
	started_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	completed_at TIMESTAMP WITH TIME ZONE, 
	error_code VARCHAR(64), 
	error_message VARCHAR(1024), 
	PRIMARY KEY (invocation_id), 
	FOREIGN KEY(sender_id) REFERENCES users (user_id)
)


;


CREATE TABLE external_command_session (
	local_session_id VARCHAR(128) NOT NULL, 
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
	last_acked_sequence INTEGER NOT NULL, 
	last_effect_id VARCHAR(128), 
	last_ack_status VARCHAR(64), 
	last_ack_error_code VARCHAR(64), 
	last_ack_error_message VARCHAR(1024), 
	connected_at TIMESTAMP WITH TIME ZONE, 
	disconnected_at TIMESTAMP WITH TIME ZONE, 
	expires_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	error_code VARCHAR(64), 
	error_message VARCHAR(1024), 
	PRIMARY KEY (local_session_id), 
	FOREIGN KEY(sender_id) REFERENCES users (user_id), 
	FOREIGN KEY(invocation_id) REFERENCES external_command_invocation (invocation_id)
)


;


CREATE TABLE farm_weather_hour (
	id SERIAL NOT NULL, 
	starts_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	weather_type VARCHAR(20) NOT NULL, 
	yield_factor DECIMAL(38, 6) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id)
)


;


CREATE TABLE futures_engine_state (
	id SERIAL NOT NULL, 
	npc_net_position DECIMAL(38, 0) NOT NULL, 
	hedge_pending_delta DECIMAL(38, 0) NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id)
)


;


CREATE TABLE futures_order (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	side VARCHAR(10) NOT NULL, 
	order_type VARCHAR(10) NOT NULL, 
	intent VARCHAR(10) NOT NULL, 
	close_position_id INTEGER, 
	price DECIMAL(38, 2), 
	quantity DECIMAL(38, 0) NOT NULL, 
	filled_quantity DECIMAL(38, 0) NOT NULL, 
	margin_frozen DECIMAL(38, 2) NOT NULL, 
	status VARCHAR(20) NOT NULL, 
	cancel_requested BOOLEAN NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(close_position_id) REFERENCES futures_position (id)
)


;


CREATE TABLE futures_order_fill (
	id SERIAL NOT NULL, 
	buy_order_id INTEGER, 
	sell_order_id INTEGER, 
	price DECIMAL(38, 2) NOT NULL, 
	quantity DECIMAL(38, 0) NOT NULL, 
	buyer_id VARCHAR NOT NULL, 
	seller_id VARCHAR NOT NULL, 
	settled BOOLEAN NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(buy_order_id) REFERENCES futures_order (id), 
	FOREIGN KEY(sell_order_id) REFERENCES futures_order (id), 
	FOREIGN KEY(buyer_id) REFERENCES users (user_id), 
	FOREIGN KEY(seller_id) REFERENCES users (user_id)
)


;


CREATE TABLE futures_position (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	side positionside NOT NULL, 
	quantity DECIMAL(38, 0) NOT NULL, 
	entry_price DECIMAL(38, 2) NOT NULL, 
	margin DECIMAL(38, 2) NOT NULL, 
	liquidation_price DECIMAL(38, 2) NOT NULL, 
	take_profit_price DECIMAL(38, 2), 
	stop_loss_price DECIMAL(38, 2), 
	pending_close_quantity DECIMAL(38, 0) NOT NULL, 
	status positionstatus NOT NULL, 
	realized_pnl DECIMAL(38, 2) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	closed_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE futures_price_snapshot (
	id SERIAL NOT NULL, 
	mid_price DECIMAL(38, 2) NOT NULL, 
	spot_price DECIMAL(38, 2) NOT NULL, 
	funding_rate DECIMAL(38, 8) NOT NULL, 
	futures_price DECIMAL(38, 2) NOT NULL, 
	open DECIMAL(38, 2), 
	high DECIMAL(38, 2), 
	low DECIMAL(38, 2), 
	volume DECIMAL(38, 0), 
	trade_count INTEGER, 
	open_interest DECIMAL(38, 0) NOT NULL, 
	total_long DECIMAL(38, 0) NOT NULL, 
	total_short DECIMAL(38, 0) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id)
)


;


CREATE TABLE futures_transaction (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	position_id INTEGER, 
	tx_type futurestransactiontype NOT NULL, 
	quantity DECIMAL(38, 0) NOT NULL, 
	price DECIMAL(38, 2) NOT NULL, 
	pnl DECIMAL(38, 2) NOT NULL, 
	raw_pnl DECIMAL(38, 2) NOT NULL, 
	paid_pnl DECIMAL(38, 2) NOT NULL, 
	unpaid_pnl DECIMAL(38, 2) NOT NULL, 
	fee DECIMAL(38, 2) NOT NULL, 
	margin_change DECIMAL(38, 2) NOT NULL, 
	spot_price DECIMAL(38, 2), 
	description VARCHAR NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE galleries (
	gallery_id VARCHAR NOT NULL, 
	title VARCHAR, 
	user_id VARCHAR, 
	user_full_name VARCHAR, 
	user_avatar_url VARCHAR, 
	images TEXT[], 
	local_image_paths TEXT[], 
	likes_count INTEGER NOT NULL, 
	dislikes_count INTEGER NOT NULL, 
	comments_count INTEGER NOT NULL, 
	top_comments JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	fetched_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	raw_data JSONB, 
	PRIMARY KEY (gallery_id)
)


;


CREATE TABLE game_currency_account (
	id SERIAL NOT NULL, 
	user_id VARCHAR(64) NOT NULL, 
	currency VARCHAR(32) NOT NULL, 
	balance DECIMAL(38, 6) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	CONSTRAINT uq_game_currency_account_user_currency UNIQUE (user_id, currency), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE game_currency_transaction (
	id SERIAL NOT NULL, 
	user_id VARCHAR(64) NOT NULL, 
	currency VARCHAR(32) NOT NULL, 
	amount DECIMAL(38, 6) NOT NULL, 
	balance_after DECIMAL(38, 6) NOT NULL, 
	tx_type VARCHAR(32) NOT NULL, 
	description VARCHAR(255) NOT NULL, 
	reference_id VARCHAR(128), 
	order_id VARCHAR(64), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE game_escrow (
	escrow_token UUID NOT NULL, 
	game_id VARCHAR NOT NULL, 
	game_type VARCHAR NOT NULL, 
	status VARCHAR NOT NULL, 
	locked_funds JSONB NOT NULL, 
	total_locked FLOAT NOT NULL, 
	settled_payouts JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	settled_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (escrow_token)
)


;


CREATE TABLE games (
	id VARCHAR NOT NULL, 
	game_type VARCHAR NOT NULL, 
	room_id VARCHAR NOT NULL, 
	creator_id VARCHAR NOT NULL, 
	status VARCHAR NOT NULL, 
	config JSONB NOT NULL, 
	players JSONB NOT NULL, 
	result JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	started_at TIMESTAMP WITH TIME ZONE, 
	finished_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id)
)


;


CREATE TABLE games.lottery_draw (
	id SERIAL NOT NULL, 
	draw_date DATE NOT NULL, 
	draw_at TIMESTAMP WITH TIME ZONE, 
	sales_close_at TIMESTAMP WITH TIME ZONE, 
	status VARCHAR NOT NULL, 
	ticket_count INTEGER NOT NULL, 
	ticket_unit_count INTEGER NOT NULL, 
	ticket_price DECIMAL(38, 2) NOT NULL, 
	prize_contribution_rate DECIMAL(38, 2) NOT NULL, 
	house_retention_rate DECIMAL(38, 2) NOT NULL, 
	pool_before_sales DECIMAL(38, 2) NOT NULL, 
	sales_amount DECIMAL(38, 2) NOT NULL, 
	prize_contribution DECIMAL(38, 2) NOT NULL, 
	house_retention DECIMAL(38, 2) NOT NULL, 
	pool_before_draw DECIMAL(38, 2) NOT NULL, 
	allocated_prize_amount DECIMAL(38, 2) NOT NULL, 
	paid_prize_amount DECIMAL(38, 2) NOT NULL, 
	carryover_amount DECIMAL(38, 2) NOT NULL, 
	winning_numbers INTEGER[], 
	draw_hash VARCHAR(64), 
	previous_draw_hash VARCHAR(64), 
	algorithm_version VARCHAR(64), 
	transcript_json JSONB, 
	ticket_commitment_input TEXT, 
	settlement_stats_json JSONB, 
	failure_code VARCHAR(64), 
	failure_message VARCHAR, 
	closed_at TIMESTAMP WITH TIME ZONE, 
	draw_started_at TIMESTAMP WITH TIME ZONE, 
	drawn_at TIMESTAMP WITH TIME ZONE, 
	settlement_started_at TIMESTAMP WITH TIME ZONE, 
	settled_at TIMESTAMP WITH TIME ZONE, 
	settlement_cursor_ticket_id INTEGER, 
	settlement_completed_at TIMESTAMP WITH TIME ZONE, 
	notification_started_at TIMESTAMP WITH TIME ZONE, 
	notification_cursor_user_id VARCHAR(255), 
	notification_completed_at TIMESTAMP WITH TIME ZONE, 
	retry_after TIMESTAMP WITH TIME ZONE, 
	retry_count INTEGER NOT NULL, 
	stage_lease_expires_at TIMESTAMP WITH TIME ZONE, 
	stage_owner VARCHAR(128), 
	notifications_sent_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	CONSTRAINT uq_lottery_draw_draw_date UNIQUE (draw_date)
)


;


CREATE TABLE games.lottery_payout (
	id SERIAL NOT NULL, 
	draw_id INTEGER NOT NULL, 
	ticket_id INTEGER NOT NULL, 
	user_id VARCHAR NOT NULL, 
	prize_tier VARCHAR(16) NOT NULL, 
	gross_amount DECIMAL(38, 2) NOT NULL, 
	tax_amount DECIMAL(38, 2) NOT NULL, 
	net_amount DECIMAL(38, 2) NOT NULL, 
	wallet_transaction_id INTEGER, 
	tax_transaction_id INTEGER, 
	reference_id VARCHAR(128) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	CONSTRAINT uq_lottery_payout_ticket_id UNIQUE (ticket_id), 
	FOREIGN KEY(draw_id) REFERENCES games.lottery_draw (id), 
	FOREIGN KEY(ticket_id) REFERENCES games.lottery_ticket (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(wallet_transaction_id) REFERENCES wallet_transaction (id), 
	FOREIGN KEY(tax_transaction_id) REFERENCES wallet_transaction (id)
)


;


CREATE TABLE games.lottery_ticket (
	id SERIAL NOT NULL, 
	draw_id INTEGER NOT NULL, 
	user_id VARCHAR NOT NULL, 
	numbers INTEGER[] NOT NULL, 
	multiplier INTEGER NOT NULL, 
	unit_price DECIMAL(38, 2) NOT NULL, 
	total_price DECIMAL(38, 2) NOT NULL, 
	status VARCHAR NOT NULL, 
	wallet_transaction_id INTEGER, 
	purchased_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	match_count INTEGER, 
	prize_tier VARCHAR(16), 
	idempotency_key VARCHAR(128) NOT NULL, 
	line_index INTEGER NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	CONSTRAINT uq_lottery_ticket_draw_user_idempotency_line UNIQUE (draw_id, user_id, idempotency_key, line_index), 
	FOREIGN KEY(draw_id) REFERENCES games.lottery_draw (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(wallet_transaction_id) REFERENCES wallet_transaction (id)
)


;


CREATE TABLE games.ponzi_events (
	id SERIAL NOT NULL, 
	session_id UUID NOT NULL, 
	seq INTEGER NOT NULL, 
	event_type VARCHAR(64) NOT NULL, 
	actor_user_id VARCHAR(128), 
	public_payload JSONB NOT NULL, 
	private_payload JSONB NOT NULL, 
	public_message_status VARCHAR(16) NOT NULL, 
	public_message_attempts INTEGER NOT NULL, 
	public_message_sent_at TIMESTAMP WITH TIME ZONE, 
	public_message_error TEXT, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(session_id) REFERENCES games.ponzi_sessions (id) ON DELETE CASCADE, 
	FOREIGN KEY(actor_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE games.ponzi_sessions (
	id UUID NOT NULL, 
	room_id VARCHAR(128) NOT NULL, 
	status VARCHAR(24) NOT NULL, 
	phase VARCHAR(32) NOT NULL, 
	players JSONB NOT NULL, 
	state JSONB NOT NULL, 
	pending_trade JSONB, 
	stop_votes JSONB NOT NULL, 
	result JSONB, 
	latest_event_seq INTEGER NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	started_at TIMESTAMP WITH TIME ZONE, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	finished_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id)
)


;


CREATE TABLE idempotency_record (
	record_id UUID NOT NULL, 
	key VARCHAR(128) NOT NULL, 
	partner_user_id VARCHAR NOT NULL, 
	partner_client_id VARCHAR(128) NOT NULL, 
	endpoint VARCHAR(255) NOT NULL, 
	request_hash VARCHAR(64) NOT NULL, 
	response_status INTEGER, 
	response_body JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (record_id), 
	FOREIGN KEY(partner_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE image_gps (
	message_id VARCHAR NOT NULL, 
	latitude FLOAT NOT NULL, 
	longitude FLOAT NOT NULL, 
	altitude FLOAT, 
	timestamp TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (message_id)
)


;


CREATE TABLE issued_principal (
	principal_id UUID NOT NULL, 
	token_kind VARCHAR(32) NOT NULL, 
	owner_user_id VARCHAR(255) NOT NULL, 
	subject_user_id VARCHAR(255) NOT NULL, 
	effective_account_user_id VARCHAR(255) NOT NULL, 
	actor_user_id VARCHAR(255) NOT NULL, 
	client_id UUID, 
	scope_snapshot JSONB NOT NULL, 
	issued_via VARCHAR(32) NOT NULL, 
	source_principal_id UUID, 
	expires_at TIMESTAMP WITH TIME ZONE, 
	revoked_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (principal_id)
)


;


CREATE TABLE land (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	permit_item_id VARCHAR(128), 
	land_type VARCHAR(20) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	upgrade_level INTEGER NOT NULL, 
	upgrade_work_total BIGINT NOT NULL, 
	upgrade_work_done DOUBLE PRECISION NOT NULL, 
	upgrade_status VARCHAR(20) NOT NULL, 
	upgrade_started_at TIMESTAMP WITH TIME ZONE, 
	upgrade_completed_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE land_assignment (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	pal_id BIGINT NOT NULL, 
	assignment_type VARCHAR(20) NOT NULL, 
	assigned_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	released_at TIMESTAMP WITH TIME ZONE, 
	last_tick_at TIMESTAMP WITH TIME ZONE, 
	consumption_remainder FLOAT NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(pal_id) REFERENCES pal (id)
)


;


CREATE TABLE llm_usage_log (
	id UUID NOT NULL, 
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
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id)
)


;


CREATE TABLE market_maker_state (
	id INTEGER NOT NULL, 
	mid_price DECIMAL(38, 2) NOT NULL, 
	inventory DECIMAL(38, 0) NOT NULL, 
	fair_value DECIMAL(38, 2) NOT NULL, 
	user_pressure DECIMAL(38, 0) NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	amm_book JSONB, 
	recent_trades JSONB, 
	futures_vamm_x DECIMAL(48, 2) NOT NULL, 
	futures_vamm_y DECIMAL(48, 2) NOT NULL, 
	futures_vamm_k DECIMAL(78, 2) NOT NULL, 
	futures_insurance_fund DECIMAL(38, 2) NOT NULL, 
	futures_open_interest DECIMAL(38, 0) NOT NULL, 
	futures_total_short DECIMAL(38, 0) NOT NULL, 
	futures_anchor_spot DECIMAL(38, 2) NOT NULL, 
	last_funding_at TIMESTAMP WITH TIME ZONE, 
	is_paused BOOLEAN NOT NULL, 
	futures_paused BOOLEAN NOT NULL, 
	fv_noise_offset FLOAT NOT NULL, 
	sub_tick INTEGER NOT NULL, 
	cached_effective_depth DECIMAL(38, 2), 
	cached_market_center DECIMAL(38, 2), 
	PRIMARY KEY (id)
)


;


CREATE TABLE market_order (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	side VARCHAR(10) NOT NULL, 
	item_category VARCHAR(20) NOT NULL, 
	item_key VARCHAR(100) NOT NULL, 
	item_quality VARCHAR(30) NOT NULL, 
	price DECIMAL(38, 2) NOT NULL, 
	quantity DECIMAL(38, 0) NOT NULL, 
	filled_quantity DECIMAL(38, 0) NOT NULL, 
	escrow_amount DECIMAL(38, 2) NOT NULL, 
	item_snapshot JSONB, 
	pal_min_level INTEGER, 
	status VARCHAR(20) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE market_order_fill (
	id SERIAL NOT NULL, 
	buy_order_id INTEGER NOT NULL, 
	sell_order_id INTEGER NOT NULL, 
	quantity DECIMAL(38, 0) NOT NULL, 
	price DECIMAL(38, 2) NOT NULL, 
	total DECIMAL(38, 2) NOT NULL, 
	fee DECIMAL(38, 2) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(buy_order_id) REFERENCES market_order (id), 
	FOREIGN KEY(sell_order_id) REFERENCES market_order (id)
)


;


CREATE TABLE messages (
	message_id VARCHAR NOT NULL, 
	room_id VARCHAR NOT NULL, 
	sent_at TIMESTAMP WITH TIME ZONE NOT NULL, 
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
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE, 
	is_deleted BOOLEAN NOT NULL, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	deleted_by VARCHAR, 
	is_recalled BOOLEAN NOT NULL, 
	is_edited BOOLEAN NOT NULL, 
	history JSONB, 
	reference_message_id VARCHAR, 
	reference_data JSONB, 
	PRIMARY KEY (message_id, sent_at)
)
 PARTITION BY RANGE (sent_at)


;


CREATE TABLE oidc_refresh_token (
	id SERIAL NOT NULL, 
	token_id UUID NOT NULL, 
	partner_user_id VARCHAR NOT NULL, 
	end_user_id VARCHAR NOT NULL, 
	client_id UUID NOT NULL, 
	token_hash VARCHAR(128) NOT NULL, 
	scope VARCHAR(2048) NOT NULL, 
	status VARCHAR(32) NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	last_used_at TIMESTAMP WITH TIME ZONE, 
	rotated_from_token_id UUID, 
	revoked_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(partner_user_id) REFERENCES users (user_id), 
	FOREIGN KEY(end_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE outgoing_commands (
	id SERIAL NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	account_user_id VARCHAR NOT NULL, 
	event VARCHAR NOT NULL, 
	data JSONB NOT NULL, 
	require_ack BOOLEAN NOT NULL, 
	status VARCHAR NOT NULL, 
	processed_at TIMESTAMP WITH TIME ZONE, 
	ack_response JSONB, 
	error_message VARCHAR, 
	attempt_count INTEGER NOT NULL, 
	max_attempts INTEGER NOT NULL, 
	PRIMARY KEY (id)
)


;


CREATE TABLE pal (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	species_code VARCHAR(100) NOT NULL, 
	custom_name VARCHAR(100), 
	rarity INTEGER NOT NULL, 
	gender VARCHAR(10) NOT NULL, 
	breeding_cooldown_until TIMESTAMP WITH TIME ZONE, 
	current_breeding_egg_id BIGINT, 
	current_breeding_until TIMESTAMP WITH TIME ZONE, 
	revival_until TIMESTAMP WITH TIME ZONE, 
	hatched_from_egg_id BIGINT, 
	archived_source_season INTEGER, 
	archived_source_pal_id BIGINT, 
	level INTEGER NOT NULL, 
	exp INTEGER NOT NULL, 
	elite_tier INTEGER NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	locked_for_order_id INTEGER, 
	asset_id VARCHAR(128) GENERATED ALWAYS AS ('pal:' || id::text) STORED NOT NULL, 
	pending_gift_id UUID, 
	PRIMARY KEY (id), 
	CONSTRAINT ck_pal_asset_id_prefix CHECK (asset_id LIKE 'pal:%%'), 
	CONSTRAINT ck_pal_archived_source_pair CHECK ((archived_source_season IS NULL) = (archived_source_pal_id IS NULL)), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(current_breeding_egg_id) REFERENCES pal_egg (id), 
	FOREIGN KEY(locked_for_order_id) REFERENCES market_order (id)
)


;


CREATE TABLE pal_adoption_record (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	pal_id INTEGER NOT NULL, 
	adoption_date DATE NOT NULL, 
	cost BIGINT NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(pal_id) REFERENCES pal (id)
)


;


CREATE TABLE pal_egg (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	egg_tier INTEGER NOT NULL, 
	element VARCHAR(20), 
	status VARCHAR(20) NOT NULL, 
	price_paid DECIMAL(38, 2) NOT NULL, 
	hatching_started_at TIMESTAMP WITH TIME ZONE, 
	hatches_at TIMESTAMP WITH TIME ZONE, 
	breeding_started_at TIMESTAMP WITH TIME ZONE, 
	breeding_ready_at TIMESTAMP WITH TIME ZONE, 
	parent1_id BIGINT, 
	parent2_id BIGINT, 
	offspring_species VARCHAR(100), 
	is_special_combo BOOLEAN NOT NULL, 
	hatched_pal_id BIGINT, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	locked_for_order_id INTEGER, 
	asset_id VARCHAR(128) GENERATED ALWAYS AS ('egg:' || id::text) STORED NOT NULL, 
	pending_gift_id UUID, 
	PRIMARY KEY (id), 
	CONSTRAINT ck_pal_egg_asset_id_prefix CHECK (asset_id LIKE 'egg:%%'), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(locked_for_order_id) REFERENCES market_order (id)
)


;


CREATE TABLE partner_client (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	name VARCHAR(128) NOT NULL, 
	client_id UUID NOT NULL, 
	client_type VARCHAR(32) NOT NULL, 
	client_secret_encrypted VARCHAR(4096), 
	status VARCHAR(32) NOT NULL, 
	client_scopes JSONB NOT NULL, 
	user_scopes JSONB NOT NULL, 
	allowed_redirect_uris JSONB NOT NULL, 
	webhook_url VARCHAR(2048), 
	webhook_secret_encrypted VARCHAR(4096), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	CONSTRAINT ck_partner_client_status CHECK (status IN ('active', 'disabled', 'revoked')), 
	CONSTRAINT ck_partner_client_client_type CHECK (client_type IN ('confidential', 'public')), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE partner_managed_account (
	id SERIAL NOT NULL, 
	owner_user_id VARCHAR(255) NOT NULL, 
	managed_user_id VARCHAR(255) NOT NULL, 
	status VARCHAR(32) NOT NULL, 
	can_login BOOLEAN NOT NULL, 
	created_by_user_id VARCHAR(255), 
	updated_by_user_id VARCHAR(255), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(owner_user_id) REFERENCES users (user_id), 
	FOREIGN KEY(managed_user_id) REFERENCES users (user_id), 
	FOREIGN KEY(created_by_user_id) REFERENCES users (user_id), 
	FOREIGN KEY(updated_by_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE partner_refresh_token (
	id SERIAL NOT NULL, 
	token_id UUID NOT NULL, 
	partner_user_id VARCHAR NOT NULL, 
	client_id UUID NOT NULL, 
	principal_id UUID, 
	token_hash VARCHAR(128) NOT NULL, 
	scope VARCHAR(2048) NOT NULL, 
	status VARCHAR(32) NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	last_used_at TIMESTAMP WITH TIME ZONE, 
	rotated_from_token_id UUID, 
	revoked_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(partner_user_id) REFERENCES users (user_id), 
	FOREIGN KEY(principal_id) REFERENCES issued_principal (principal_id)
)


;


CREATE TABLE payment_intent (
	intent_id UUID NOT NULL, 
	partner_user_id VARCHAR NOT NULL, 
	partner_client_id UUID NOT NULL, 
	user_id VARCHAR NOT NULL, 
	operation VARCHAR(32) NOT NULL, 
	account_code VARCHAR(255) NOT NULL, 
	amount DECIMAL(38, 2) NOT NULL, 
	asset_code VARCHAR(32) NOT NULL, 
	title VARCHAR(255) NOT NULL, 
	summary VARCHAR(2000) NOT NULL, 
	partner_reference_id VARCHAR(255) NOT NULL, 
	return_url VARCHAR(2048) NOT NULL, 
	cancel_url VARCHAR(2048) NOT NULL, 
	checkout_token UUID NOT NULL, 
	status VARCHAR(64) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	authorized_at TIMESTAMP WITH TIME ZONE, 
	completed_at TIMESTAMP WITH TIME ZONE, 
	cancelled_at TIMESTAMP WITH TIME ZONE, 
	error_code VARCHAR(64), 
	error_message VARCHAR(2000), 
	PRIMARY KEY (intent_id), 
	FOREIGN KEY(partner_user_id) REFERENCES users (user_id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE pending_gift (
	gift_id UUID NOT NULL, 
	asset_id VARCHAR(128) NOT NULL, 
	asset_family VARCHAR(32) NOT NULL, 
	from_user_id VARCHAR(100) NOT NULL, 
	to_user_id VARCHAR(100) NOT NULL, 
	status VARCHAR(20) NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	resolved_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (gift_id, asset_id), 
	CONSTRAINT ck_pending_gift_asset_prefix CHECK (asset_id LIKE asset_family || ':%%'), 
	FOREIGN KEY(from_user_id) REFERENCES users (user_id), 
	FOREIGN KEY(to_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE poll_comment_reactions (
	id UUID NOT NULL, 
	comment_id UUID NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	emoji VARCHAR(16) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(comment_id) REFERENCES poll_comments (id) ON DELETE CASCADE, 
	CONSTRAINT ux_poll_comment_reactions_comment_user_emoji UNIQUE (comment_id, user_id, emoji), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE poll_comments (
	id UUID NOT NULL, 
	poll_id UUID NOT NULL, 
	author_id VARCHAR(100) NOT NULL, 
	content VARCHAR(1000) NOT NULL, 
	quote_comment_id UUID, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(poll_id) REFERENCES polls (id) ON DELETE CASCADE, 
	FOREIGN KEY(quote_comment_id) REFERENCES poll_comments (id) ON DELETE SET NULL
)


;


CREATE TABLE poll_options (
	id UUID NOT NULL, 
	poll_id UUID NOT NULL, 
	position INTEGER NOT NULL, 
	label VARCHAR(120) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(poll_id) REFERENCES polls (id) ON DELETE CASCADE, 
	CONSTRAINT ux_poll_options_poll_position UNIQUE (poll_id, position), 
	CONSTRAINT ux_poll_options_poll_id_id UNIQUE (poll_id, id)
)


;


CREATE TABLE poll_votes (
	id UUID NOT NULL, 
	poll_id UUID NOT NULL, 
	option_id UUID NOT NULL, 
	voter_user_id VARCHAR(100) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	CONSTRAINT ux_poll_votes_poll_voter_option UNIQUE (poll_id, voter_user_id, option_id), 
	FOREIGN KEY(poll_id) REFERENCES polls (id) ON DELETE CASCADE, 
	CONSTRAINT fk_poll_votes_poll_option_pair FOREIGN KEY(poll_id, option_id) REFERENCES poll_options (poll_id, id) ON DELETE CASCADE, 
	FOREIGN KEY(voter_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE polls (
	id UUID NOT NULL, 
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
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	closed_at TIMESTAMP WITH TIME ZONE, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	CONSTRAINT ck_polls_scope_room_id CHECK ((scope = 'global' AND room_id IS NULL) OR (scope = 'room' AND room_id IS NOT NULL))
)


;


CREATE TABLE private_rooms (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	room_id VARCHAR NOT NULL, 
	bot_user_id VARCHAR, 
	invite_link VARCHAR, 
	protected BOOLEAN NOT NULL, 
	pending_message_id VARCHAR, 
	pending_room_id VARCHAR, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(bot_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE raid_action_log (
	id BIGINT GENERATED BY DEFAULT AS IDENTITY, 
	session_id VARCHAR(32) NOT NULL, 
	seq INTEGER NOT NULL, 
	action JSONB NOT NULL, 
	effects JSONB NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id, created_at)
)
 PARTITION BY RANGE (created_at)


;


CREATE TABLE raid_map (
	id BIGSERIAL NOT NULL, 
	seed INTEGER NOT NULL, 
	config JSONB NOT NULL, 
	version INTEGER NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id)
)


;


CREATE TABLE raid_map_floor (
	map_id BIGINT NOT NULL, 
	floor INTEGER NOT NULL, 
	tiles JSONB NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (map_id, floor)
)


;


CREATE TABLE raid_map_progress (
	user_id VARCHAR(64) NOT NULL, 
	map_id BIGINT NOT NULL, 
	explored_rooms JSONB, 
	activated_waystones JSONB, 
	pressure INTEGER NOT NULL, 
	run_count INTEGER NOT NULL, 
	last_started_at TIMESTAMP WITH TIME ZONE, 
	last_settled_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (user_id, map_id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE raid_profile (
	user_id VARCHAR(64) NOT NULL, 
	name VARCHAR(256) NOT NULL, 
	str_stat INTEGER NOT NULL, 
	dex INTEGER NOT NULL, 
	wil INTEGER NOT NULL, 
	per INTEGER NOT NULL, 
	level INTEGER NOT NULL, 
	experience INTEGER NOT NULL, 
	max_willpower INTEGER NOT NULL, 
	pending_attribute_points INTEGER NOT NULL, 
	skill_search INTEGER NOT NULL, 
	skill_combat INTEGER NOT NULL, 
	skill_stealth INTEGER NOT NULL, 
	skill_resist INTEGER NOT NULL, 
	total_raids INTEGER NOT NULL, 
	raids_survived INTEGER NOT NULL, 
	total_loot_value DECIMAL(38, 0) NOT NULL, 
	max_rooms_explored INTEGER NOT NULL, 
	total_kills INTEGER NOT NULL, 
	total_bosses_killed INTEGER NOT NULL, 
	survival_streak INTEGER NOT NULL, 
	best_survival_streak INTEGER NOT NULL, 
	gifts_placed INTEGER NOT NULL, 
	gifts_received INTEGER NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (user_id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE raid_risk_control_state (
	user_id VARCHAR(64) NOT NULL, 
	last_evaluated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	precheck_band VARCHAR(16) NOT NULL, 
	effective_shadow_band VARCHAR(16) NOT NULL, 
	starts_6h INTEGER NOT NULL, 
	starts_24h INTEGER NOT NULL, 
	hot_30m_buckets_24h INTEGER NOT NULL, 
	turnstile_passed_at TIMESTAMP WITH TIME ZONE, 
	turnstile_exempt_nonce VARCHAR(128), 
	signals_json JSONB NOT NULL, 
	PRIMARY KEY (user_id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE raid_session (
	id VARCHAR(32) NOT NULL, 
	user_id VARCHAR(64) NOT NULL, 
	map_id BIGINT, 
	state_json JSONB NOT NULL, 
	risk_snapshot_json JSONB, 
	is_active BOOLEAN NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	current_seq INTEGER NOT NULL, 
	settled_player_action_count INTEGER, 
	settled_action_interval_stddev_seconds DOUBLE PRECISION, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE raid_warehouse_item (
	item_id VARCHAR(128) NOT NULL, 
	user_id VARCHAR(64) NOT NULL, 
	template_id VARCHAR(64) NOT NULL, 
	item_type VARCHAR(32) NOT NULL, 
	equipped_slot VARCHAR(32), 
	quality VARCHAR(32) NOT NULL, 
	quantity INTEGER NOT NULL, 
	item_data JSONB NOT NULL, 
	market_locked_for_order_id BIGINT, 
	pending_gift_id UUID, 
	location VARCHAR(16) NOT NULL, 
	carry_order INTEGER, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (item_id), 
	CONSTRAINT ck_raid_warehouse_item_equipped_slot_required CHECK ((location <> 'equipped') OR (equipped_slot IS NOT NULL)), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE red_envelope (
	id SERIAL NOT NULL, 
	sender_id VARCHAR NOT NULL, 
	room_id VARCHAR NOT NULL, 
	message_id VARCHAR NOT NULL, 
	envelope_type VARCHAR NOT NULL, 
	total_amount DECIMAL(38, 2) NOT NULL, 
	remaining_amount DECIMAL(38, 2) NOT NULL, 
	total_count INTEGER NOT NULL, 
	remaining_count INTEGER NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	is_expired BOOLEAN NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(sender_id) REFERENCES users (user_id)
)


;


CREATE TABLE red_envelope_claim (
	id SERIAL NOT NULL, 
	envelope_id INTEGER NOT NULL, 
	user_id VARCHAR NOT NULL, 
	amount DECIMAL(38, 2) NOT NULL, 
	claimed_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(envelope_id) REFERENCES red_envelope (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE resource_production (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	land_id BIGINT NOT NULL, 
	accumulated_credits DECIMAL(38, 2) NOT NULL, 
	last_tick_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	is_paused BOOLEAN NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(land_id) REFERENCES land (id)
)


;


CREATE TABLE room_members (
	room_id VARCHAR NOT NULL, 
	user_id VARCHAR NOT NULL, 
	role VARCHAR, 
	joined_at TIMESTAMP WITH TIME ZONE, 
	left_at TIMESTAMP WITH TIME ZONE, 
	raw_data JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (room_id, user_id)
)


;


CREATE TABLE rooms (
	room_id VARCHAR NOT NULL, 
	title VARCHAR NOT NULL, 
	chat_type VARCHAR, 
	avatar_url VARCHAR, 
	member_count INTEGER, 
	tags TEXT[], 
	is_public BOOLEAN, 
	creator_id VARCHAR, 
	account_ids TEXT[] DEFAULT '{}' NOT NULL, 
	last_message_at TIMESTAMP WITH TIME ZONE, 
	first_message_at TIMESTAMP WITH TIME ZONE, 
	backfill_until TIMESTAMP WITH TIME ZONE, 
	history_complete BOOLEAN NOT NULL, 
	message_count INTEGER NOT NULL, 
	deleted_count INTEGER NOT NULL, 
	recalled_count INTEGER NOT NULL, 
	edited_count INTEGER NOT NULL, 
	image_count INTEGER NOT NULL, 
	is_active BOOLEAN NOT NULL, 
	dissolved_at TIMESTAMP WITH TIME ZONE, 
	raw_data JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (room_id)
)


;


CREATE TABLE season_settlement (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	season INTEGER NOT NULL, 
	rank INTEGER NOT NULL, 
	cash_balance DECIMAL(38, 2) NOT NULL, 
	escrow_total DECIMAL(38, 2) NOT NULL, 
	resource_credits DECIMAL(38, 2) NOT NULL, 
	turnip_value DECIMAL(38, 2) NOT NULL, 
	excess_pal_value DECIMAL(38, 2) NOT NULL, 
	land_value DECIMAL(38, 2) NOT NULL, 
	tax_paid DECIMAL(38, 2) NOT NULL, 
	total_settlement DECIMAL(38, 2) NOT NULL, 
	starting_balance DECIMAL(38, 2) NOT NULL, 
	pals_kept INTEGER NOT NULL, 
	pals_settled INTEGER NOT NULL, 
	lands_settled INTEGER NOT NULL, 
	details JSONB, 
	settled_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE security_audit_event (
	event_id UUID NOT NULL, 
	principal_id UUID NOT NULL, 
	action VARCHAR(100) NOT NULL, 
	result VARCHAR(32) NOT NULL, 
	target_type VARCHAR(64) NOT NULL, 
	target_id VARCHAR(255), 
	error_code VARCHAR(64), 
	metadata JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (event_id), 
	FOREIGN KEY(principal_id) REFERENCES issued_principal (principal_id)
)


;


CREATE TABLE shared_kv (
	id SERIAL NOT NULL, 
	namespace VARCHAR NOT NULL, 
	key VARCHAR NOT NULL, 
	value JSONB NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	CONSTRAINT uq_shared_kv_namespace_key UNIQUE (namespace, key)
)


;


CREATE TABLE stock.consumer_cursor (
	consumer_name VARCHAR(64) NOT NULL, 
	symbol VARCHAR(32) NOT NULL, 
	last_processed_candle_at TIMESTAMP WITH TIME ZONE, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (consumer_name, symbol)
)


;


CREATE TABLE stock.finalized_candle (
	symbol VARCHAR(32) NOT NULL, 
	candle_start TIMESTAMP WITH TIME ZONE NOT NULL, 
	open DECIMAL(38, 2) NOT NULL, 
	high DECIMAL(38, 2) NOT NULL, 
	low DECIMAL(38, 2) NOT NULL, 
	close DECIMAL(38, 2) NOT NULL, 
	volume DECIMAL(38, 2), 
	source VARCHAR(32) NOT NULL, 
	session_kind VARCHAR(16) NOT NULL, 
	finalized_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (symbol, candle_start)
)


;


CREATE TABLE stock.producer_heartbeat (
	producer_id VARCHAR(64) NOT NULL, 
	heartbeat_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	last_reconcile_started_at TIMESTAMP WITH TIME ZONE, 
	last_reconcile_finished_at TIMESTAMP WITH TIME ZONE, 
	mode VARCHAR(32), 
	ws_connected BOOLEAN, 
	ws_last_connected_at TIMESTAMP WITH TIME ZONE, 
	ws_last_message_at TIMESTAMP WITH TIME ZONE, 
	ws_subscription_count INTEGER, 
	last_targeted_reconcile_started_at TIMESTAMP WITH TIME ZONE, 
	last_targeted_reconcile_finished_at TIMESTAMP WITH TIME ZONE, 
	last_targeted_reconcile_symbol_count INTEGER, 
	last_gap_repair_started_at TIMESTAMP WITH TIME ZONE, 
	last_gap_repair_finished_at TIMESTAMP WITH TIME ZONE, 
	last_gap_repair_symbol_count INTEGER, 
	PRIMARY KEY (producer_id)
)


;


CREATE TABLE stock_account (
	user_id VARCHAR NOT NULL, 
	total_realized_pnl DECIMAL(38, 2) NOT NULL, 
	trade_count INTEGER NOT NULL, 
	best_trade_pnl DECIMAL(38, 2) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (user_id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE stock_pending_order (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	idempotency_key VARCHAR(64) NOT NULL, 
	symbol VARCHAR NOT NULL, 
	action VARCHAR(32) NOT NULL, 
	status VARCHAR(32) NOT NULL, 
	failure_reason VARCHAR(64), 
	request_mode VARCHAR(16) NOT NULL, 
	settlement_policy VARCHAR(48) NOT NULL, 
	requested_shares DECIMAL(38, 6), 
	requested_amount DECIMAL(38, 2), 
	requested_leverage INTEGER NOT NULL, 
	anchor_timestamp TIMESTAMP WITH TIME ZONE NOT NULL, 
	anchor_market_minute_start TIMESTAMP WITH TIME ZONE NOT NULL, 
	anchor_market_minute_end TIMESTAMP WITH TIME ZONE NOT NULL, 
	settlement_deadline TIMESTAMP WITH TIME ZONE NOT NULL, 
	acceptance_quote_price DECIMAL(38, 2) NOT NULL, 
	acceptance_quote_time TIMESTAMP WITH TIME ZONE, 
	acceptance_market_state VARCHAR(20) NOT NULL, 
	acceptance_risk_snapshot_json JSONB, 
	frozen_cash_amount DECIMAL(38, 2) NOT NULL, 
	reserved_shares DECIMAL(38, 6) NOT NULL, 
	settlement_price DECIMAL(38, 2), 
	filled_shares DECIMAL(38, 6) NOT NULL, 
	refunded_cash_amount DECIMAL(38, 2) NOT NULL, 
	cancel_fee_cash_amount DECIMAL(38, 2), 
	cancel_fee_shares DECIMAL(38, 6), 
	settling_started_at TIMESTAMP WITH TIME ZONE, 
	settlement_attempt_count INTEGER NOT NULL, 
	settlement_worker_id VARCHAR(64), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	settled_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	CONSTRAINT ck_stock_pending_order_request_mode_exclusive CHECK ((request_mode = 'amount' AND requested_amount IS NOT NULL AND requested_shares IS NULL) OR (request_mode = 'shares' AND requested_shares IS NOT NULL AND requested_amount IS NULL)), 
	CONSTRAINT uq_stock_pending_order_user_idempotency_key UNIQUE (user_id, idempotency_key), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE stock_portfolio (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	symbol VARCHAR NOT NULL, 
	position_type positiontype NOT NULL, 
	shares DECIMAL(38, 6) NOT NULL, 
	buy_price DECIMAL(38, 2) NOT NULL, 
	leverage INTEGER NOT NULL, 
	bought_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE stock_portfolio_adjustment (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	symbol VARCHAR NOT NULL, 
	position_type VARCHAR(16) NOT NULL, 
	adjustment_type VARCHAR(64) NOT NULL, 
	shares_delta DECIMAL(38, 6) NOT NULL, 
	pending_order_id INTEGER NOT NULL, 
	metadata_json JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(pending_order_id) REFERENCES stock_pending_order (id)
)


;


CREATE TABLE stock_position_reservation (
	id SERIAL NOT NULL, 
	pending_order_id INTEGER NOT NULL, 
	user_id VARCHAR NOT NULL, 
	symbol VARCHAR NOT NULL, 
	position_type VARCHAR(16) NOT NULL, 
	shares_reserved DECIMAL(38, 6) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	released_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	CONSTRAINT ck_stock_position_reservation_positive_shares CHECK (shares_reserved > 0), 
	CONSTRAINT uq_stock_position_reservation_pending_order_id UNIQUE (pending_order_id), 
	FOREIGN KEY(pending_order_id) REFERENCES stock_pending_order (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE stock_trade_history (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	pending_order_id INTEGER, 
	symbol VARCHAR NOT NULL, 
	action tradeaction NOT NULL, 
	shares DECIMAL(38, 6) NOT NULL, 
	price DECIMAL(38, 2) NOT NULL, 
	leverage INTEGER NOT NULL, 
	pnl DECIMAL(38, 2), 
	executed_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(pending_order_id) REFERENCES stock_pending_order (id)
)


;


CREATE TABLE stock_trigger (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	symbol VARCHAR NOT NULL, 
	position_type positiontype NOT NULL, 
	trigger_type triggertype NOT NULL, 
	trigger_price DECIMAL(38, 2) NOT NULL, 
	shares DECIMAL(38, 6), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE strand_object (
	id BIGSERIAL NOT NULL, 
	map_id BIGINT NOT NULL, 
	type VARCHAR(16) NOT NULL, 
	floor INTEGER NOT NULL, 
	x INTEGER NOT NULL, 
	y INTEGER NOT NULL, 
	owner_user_id VARCHAR(64) NOT NULL, 
	data JSONB NOT NULL, 
	likes INTEGER NOT NULL, 
	picked_up_by VARCHAR(64), 
	expires_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(owner_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE sudoku_puzzle (
	id SERIAL NOT NULL, 
	room_id VARCHAR NOT NULL, 
	creator_id VARCHAR NOT NULL, 
	request_message_id VARCHAR NOT NULL, 
	announcement_message_id VARCHAR, 
	difficulty VARCHAR NOT NULL, 
	puzzle VARCHAR NOT NULL, 
	solution VARCHAR NOT NULL, 
	status VARCHAR NOT NULL, 
	solver_id VARCHAR, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	finished_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id)
)


;


CREATE TABLE trpg_checkpoints (
	id SERIAL NOT NULL, 
	room_id VARCHAR NOT NULL, 
	game_id INTEGER, 
	name VARCHAR NOT NULL, 
	state JSONB NOT NULL, 
	saved_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	UNIQUE (room_id, name)
)


;


CREATE TABLE trpg_games (
	id SERIAL NOT NULL, 
	room_id VARCHAR NOT NULL, 
	title VARCHAR NOT NULL, 
	started_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	ended_at TIMESTAMP WITH TIME ZONE, 
	background VARCHAR NOT NULL, 
	mission VARCHAR NOT NULL, 
	rule VARCHAR NOT NULL, 
	scene VARCHAR NOT NULL, 
	summary VARCHAR NOT NULL, 
	summarized_turn_count INTEGER NOT NULL, 
	turns JSONB DEFAULT '[]' NOT NULL, 
	pcs JSONB DEFAULT '{}' NOT NULL, 
	npcs JSONB DEFAULT '{}' NOT NULL, 
	bags JSONB DEFAULT '{}' NOT NULL, 
	bag_logs JSONB DEFAULT '[]' NOT NULL, 
	undo_stack JSONB DEFAULT '[]' NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id)
)


;


CREATE TABLE turnip_inventory (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	quantity DECIMAL(38, 0) NOT NULL, 
	buy_price DECIMAL(38, 2) NOT NULL, 
	purchased_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	settles_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	expires_at TIMESTAMP WITH TIME ZONE, 
	is_harvested BOOLEAN NOT NULL, 
	is_stored BOOLEAN NOT NULL, 
	stored_shelf_life_seconds DECIMAL(38, 2), 
	locked_for_order_id INTEGER, 
	market_locked_for_order_id INTEGER, 
	pending_gift_id UUID, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(locked_for_order_id) REFERENCES turnip_order (id), 
	FOREIGN KEY(market_locked_for_order_id) REFERENCES market_order (id)
)


;


CREATE TABLE turnip_market_event_composition (
	id SERIAL NOT NULL, 
	payload_hash VARCHAR(64) NOT NULL, 
	payload JSONB NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id)
)


;


CREATE TABLE turnip_market_event_instance (
	id SERIAL NOT NULL, 
	event_version_id INTEGER NOT NULL, 
	starts_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	ends_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	priority INTEGER NOT NULL, 
	weight NUMERIC NOT NULL, 
	paused BOOLEAN NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(event_version_id) REFERENCES turnip_market_event_version (id)
)


;


CREATE TABLE turnip_market_event_version (
	id SERIAL NOT NULL, 
	name VARCHAR(120) NOT NULL, 
	payload JSONB NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	created_by VARCHAR, 
	note VARCHAR, 
	parent_version_id INTEGER, 
	PRIMARY KEY (id), 
	FOREIGN KEY(parent_version_id) REFERENCES turnip_market_event_version (id)
)


;


CREATE TABLE turnip_market_guardrail_policy_version (
	id SERIAL NOT NULL, 
	mode VARCHAR(32) NOT NULL, 
	oracle_band_pct NUMERIC NOT NULL, 
	admission_band_pct NUMERIC NOT NULL, 
	execution_hard_band_pct NUMERIC NOT NULL, 
	max_limit_order_notional_vs_nav_ratio NUMERIC NOT NULL, 
	max_market_order_notional_vs_turnover_ratio NUMERIC NOT NULL, 
	max_taking_order_quantity_vs_visible_depth_ratio NUMERIC NOT NULL, 
	sink_quote_budget_cash_ratio NUMERIC NOT NULL, 
	source_quote_budget_inventory_ratio NUMERIC NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	created_by VARCHAR, 
	note VARCHAR, 
	parent_version_id INTEGER, 
	PRIMARY KEY (id), 
	FOREIGN KEY(parent_version_id) REFERENCES turnip_market_guardrail_policy_version (id)
)


;


CREATE TABLE turnip_market_snapshot (
	id SERIAL NOT NULL, 
	prev_snapshot_id INTEGER, 
	guardrail_policy_version_id INTEGER, 
	config_version_id INTEGER, 
	event_composition_id INTEGER, 
	active_overlay_revision_id INTEGER, 
	last_trade_price DECIMAL(38, 2) NOT NULL, 
	last_raw_trade_price DECIMAL(38, 2), 
	last_qualified_trade_price DECIMAL(38, 2), 
	qualified_fill_seen_tick BOOLEAN NOT NULL, 
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
	support_bucket_started_at TIMESTAMP WITH TIME ZONE, 
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
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(prev_snapshot_id) REFERENCES turnip_market_snapshot (id) ON DELETE RESTRICT, 
	FOREIGN KEY(guardrail_policy_version_id) REFERENCES turnip_market_guardrail_policy_version (id) ON DELETE RESTRICT, 
	FOREIGN KEY(config_version_id) REFERENCES economy_config_version (id), 
	FOREIGN KEY(active_overlay_revision_id) REFERENCES turnip_scenario_overlay_revision (id)
)


;


CREATE TABLE turnip_order (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	order_type VARCHAR(10) NOT NULL, 
	side VARCHAR(10) NOT NULL, 
	quantity DECIMAL(38, 0) NOT NULL, 
	filled_quantity DECIMAL(38, 0) NOT NULL, 
	limit_price DECIMAL(38, 2) NOT NULL, 
	escrow_amount DECIMAL(38, 2) NOT NULL, 
	quote_price DECIMAL(38, 2), 
	execution_price DECIMAL(38, 2), 
	slippage_pct FLOAT, 
	status VARCHAR(20) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	filled_at TIMESTAMP WITH TIME ZONE, 
	cancelled_at TIMESTAMP WITH TIME ZONE, 
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE turnip_order_fill (
	id SERIAL NOT NULL, 
	order_id INTEGER NOT NULL, 
	fill_type VARCHAR(10) NOT NULL, 
	counterparty_order_id INTEGER, 
	quantity DECIMAL(38, 0) NOT NULL, 
	price DECIMAL(38, 2) NOT NULL, 
	total DECIMAL(38, 2) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(order_id) REFERENCES turnip_order (id)
)


;


CREATE TABLE turnip_price (
	id SERIAL NOT NULL, 
	price DECIMAL(38, 2) NOT NULL, 
	open DECIMAL(38, 2), 
	high DECIMAL(38, 2), 
	low DECIMAL(38, 2), 
	volume DECIMAL(38, 0) NOT NULL, 
	trade_count DECIMAL(38, 0) NOT NULL, 
	trend VARCHAR(20) NOT NULL, 
	trend_tick INTEGER NOT NULL, 
	base_price DECIMAL(38, 2) NOT NULL, 
	cycle_ticks INTEGER, 
	cycle_context JSONB, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id)
)


;


CREATE TABLE turnip_scenario_overlay_revision (
	id SERIAL NOT NULL, 
	stage_id INTEGER NOT NULL, 
	reason VARCHAR(32) NOT NULL, 
	target_now DECIMAL(38, 6) NOT NULL, 
	observed_price DECIMAL(38, 2) NOT NULL, 
	observed_fv DECIMAL(38, 2) NOT NULL, 
	observed_reference_price DECIMAL(38, 2) NOT NULL, 
	error DECIMAL(38, 6) NOT NULL, 
	runtime_patch_json JSONB NOT NULL, 
	event_patch_json JSONB NOT NULL, 
	scenario_patch_json JSONB NOT NULL, 
	entry_patch_json JSONB NOT NULL, 
	neutralize_progress_ratio DECIMAL(38, 6), 
	effective_overlay_hash VARCHAR(64) NOT NULL, 
	supervisor_version VARCHAR(120) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(stage_id) REFERENCES turnip_scenario_stage (id)
)


;


CREATE TABLE turnip_scenario_run (
	id SERIAL NOT NULL, 
	template_id INTEGER, 
	template_snapshot_json JSONB NOT NULL, 
	status VARCHAR(16) NOT NULL, 
	start_mode VARCHAR(16) NOT NULL, 
	scheduled_at TIMESTAMP WITH TIME ZONE, 
	started_at TIMESTAMP WITH TIME ZONE, 
	current_stage_started_at TIMESTAMP WITH TIME ZONE, 
	paused_at TIMESTAMP WITH TIME ZONE, 
	ended_at TIMESTAMP WITH TIME ZONE, 
	baseline_config_version_id INTEGER, 
	current_stage_index INTEGER NOT NULL, 
	heartbeat_interval_sec INTEGER NOT NULL, 
	last_heartbeat_at TIMESTAMP WITH TIME ZONE, 
	lease_expires_at TIMESTAMP WITH TIME ZONE, 
	created_by VARCHAR(100), 
	abort_reason VARCHAR, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(template_id) REFERENCES turnip_scenario_template (id), 
	FOREIGN KEY(baseline_config_version_id) REFERENCES economy_config_version (id)
)


;


CREATE TABLE turnip_scenario_stage (
	id SERIAL NOT NULL, 
	run_id INTEGER NOT NULL, 
	stage_index INTEGER NOT NULL, 
	name VARCHAR(120) NOT NULL, 
	stage_type VARCHAR(32) NOT NULL, 
	mode VARCHAR(16) NOT NULL, 
	duration_sec INTEGER NOT NULL, 
	regime_target DECIMAL(38, 6), 
	target_ref VARCHAR(32), 
	target_value DECIMAL(38, 6), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(run_id) REFERENCES turnip_scenario_run (id)
)


;


CREATE TABLE turnip_scenario_template (
	id SERIAL NOT NULL, 
	name VARCHAR(120) NOT NULL, 
	description VARCHAR, 
	default_start_mode VARCHAR(16) NOT NULL, 
	stage_definition_json JSONB NOT NULL, 
	created_by VARCHAR(100), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	CONSTRAINT ck_turnip_scenario_template_stage_definition_nonempty CHECK (jsonb_typeof(stage_definition_json) = 'array' AND jsonb_array_length(stage_definition_json) > 0)
)


;


CREATE TABLE turnip_seed (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	quantity DECIMAL(38, 0) NOT NULL, 
	seed_price DECIMAL(38, 2) NOT NULL, 
	planted_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	fertilize_count INTEGER NOT NULL, 
	fertilize_cost DECIMAL(38, 2) NOT NULL, 
	weather_score DECIMAL(38, 6) NOT NULL, 
	growth_required_hours DECIMAL(38, 6) NOT NULL, 
	growth_progress_hours DECIMAL(38, 6) NOT NULL, 
	last_growth_accounted_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	latest_effective_time_bonus DECIMAL(38, 6) NOT NULL, 
	latest_effective_harvest_bonus DECIMAL(38, 6) NOT NULL, 
	locked_weather_yield_factor DECIMAL(38, 6) NOT NULL, 
	pal_harvest_bonus_score DECIMAL(38, 6) NOT NULL, 
	locked_pal_harvest_bonus DECIMAL(38, 6) NOT NULL, 
	pal_harvest_bonus_locked BOOLEAN NOT NULL, 
	batch_yield_factor DECIMAL(38, 6) NOT NULL, 
	status VARCHAR(20) NOT NULL, 
	harvested_at TIMESTAMP WITH TIME ZONE, 
	matured_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE turnip_trade (
	id SERIAL NOT NULL, 
	side VARCHAR(10) NOT NULL, 
	quantity DECIMAL(38, 0) NOT NULL, 
	price DECIMAL(38, 2) NOT NULL, 
	total DECIMAL(38, 2) NOT NULL, 
	maker_actor VARCHAR(100) NOT NULL, 
	taker_actor VARCHAR(100) NOT NULL, 
	maker_order_id INTEGER, 
	taker_order_id INTEGER, 
	trade_type VARCHAR(16) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(maker_order_id) REFERENCES turnip_order (id), 
	FOREIGN KEY(taker_order_id) REFERENCES turnip_order (id)
)


;


CREATE TABLE turnip_transaction (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	quantity DECIMAL(38, 0) NOT NULL, 
	balance_after DECIMAL(38, 0) NOT NULL, 
	tx_type VARCHAR(20) NOT NULL, 
	unit_price DECIMAL(38, 2) NOT NULL, 
	description VARCHAR(200) NOT NULL, 
	mid_price DECIMAL(38, 2), 
	inventory_ids INTEGER[], 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE tweets (
	tweet_id VARCHAR NOT NULL, 
	user_id VARCHAR, 
	content VARCHAR, 
	media_urls TEXT[], 
	local_media_paths TEXT[], 
	source VARCHAR, 
	tweet_type VARCHAR, 
	parent_tweet_id VARCHAR, 
	reply_to_tweet_id VARCHAR, 
	reply_to_username VARCHAR, 
	is_edited BOOLEAN NOT NULL, 
	edit_history JSONB, 
	post_id VARCHAR, 
	draw_id VARCHAR, 
	likes_count INTEGER NOT NULL, 
	comments_count INTEGER NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE, 
	fetched_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	is_deleted BOOLEAN NOT NULL, 
	raw_data JSONB, 
	PRIMARY KEY (tweet_id)
)


;


CREATE TABLE undercover_event (
	id SERIAL NOT NULL, 
	session_id UUID NOT NULL, 
	seq INTEGER NOT NULL, 
	event_type VARCHAR(64) NOT NULL, 
	actor_user_id VARCHAR(128), 
	public_payload JSONB NOT NULL, 
	private_payload JSONB NOT NULL, 
	public_message_status VARCHAR(16) NOT NULL, 
	public_message_attempts INTEGER NOT NULL, 
	public_message_sent_at TIMESTAMP WITH TIME ZONE, 
	public_message_error TEXT, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(session_id) REFERENCES undercover_session (id) ON DELETE CASCADE, 
	FOREIGN KEY(actor_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE undercover_session (
	id UUID NOT NULL, 
	room_id VARCHAR NOT NULL, 
	creator_id VARCHAR NOT NULL, 
	status VARCHAR NOT NULL, 
	phase VARCHAR NOT NULL, 
	config JSONB NOT NULL, 
	state_payload JSONB NOT NULL, 
	phase_deadline_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	started_at TIMESTAMP WITH TIME ZONE, 
	finished_at TIMESTAMP WITH TIME ZONE, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	last_activity_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id)
)


;


CREATE TABLE undercover_word_pair (
	id UUID NOT NULL, 
	word_a VARCHAR(32) NOT NULL, 
	word_b VARCHAR(32) NOT NULL, 
	canonical_word_a VARCHAR(128) NOT NULL, 
	canonical_word_b VARCHAR(128) NOT NULL, 
	submitter_user_id VARCHAR NOT NULL, 
	is_active BOOLEAN NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	CONSTRAINT ux_undercover_word_pair_canonical UNIQUE (canonical_word_a, canonical_word_b), 
	FOREIGN KEY(submitter_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE user_achievement (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	achievement_key VARCHAR(100) NOT NULL, 
	unlocked_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	context JSONB, 
	PRIMARY KEY (id), 
	CONSTRAINT uq_user_achievement UNIQUE (user_id, achievement_key), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE user_achievement_progress (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	stat_key VARCHAR(100) NOT NULL, 
	value DECIMAL(38, 0) NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	CONSTRAINT uq_user_stat UNIQUE (user_id, stat_key), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE user_credential (
	id SERIAL NOT NULL, 
	username VARCHAR(64) NOT NULL, 
	password_hash VARCHAR(256) NOT NULL, 
	user_id VARCHAR NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	last_login TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE user_history (
	id SERIAL NOT NULL, 
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
	recorded_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE user_item (
	id SERIAL NOT NULL, 
	user_id VARCHAR(100) NOT NULL, 
	item_type VARCHAR(50) NOT NULL, 
	display_name VARCHAR(200), 
	quantity INTEGER NOT NULL, 
	purchased_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	activated_at TIMESTAMP WITH TIME ZONE, 
	expires_at TIMESTAMP WITH TIME ZONE, 
	price_paid DECIMAL(38, 2) NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE user_notification (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	category VARCHAR NOT NULL, 
	content VARCHAR NOT NULL, 
	reference_id VARCHAR(255), 
	is_read BOOLEAN NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE user_passkey (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	name VARCHAR(128) NOT NULL, 
	credential_id VARCHAR(512) NOT NULL, 
	public_key VARCHAR(4096) NOT NULL, 
	user_handle VARCHAR(128) NOT NULL, 
	sign_count INTEGER NOT NULL, 
	transports JSONB NOT NULL, 
	backup_state VARCHAR(32), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	last_used_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE users (
	user_id VARCHAR NOT NULL, 
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
	last_seen TIMESTAMP WITH TIME ZONE, 
	message_count INTEGER NOT NULL, 
	deleted_count INTEGER NOT NULL, 
	recalled_count INTEGER NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (user_id)
)


;


CREATE TABLE wallet (
	user_id VARCHAR NOT NULL, 
	allow_negative_balance BOOLEAN NOT NULL, 
	snapshot_balance DECIMAL(38, 2) NOT NULL, 
	snapshot_escrow_balance DECIMAL(38, 2) NOT NULL, 
	snapshot_tx_id BIGINT NOT NULL, 
	last_daily_credit DATE, 
	total_credited DECIMAL(38, 2) NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (user_id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id)
)


;


CREATE TABLE wallet_transaction (
	id SERIAL NOT NULL, 
	user_id VARCHAR NOT NULL, 
	amount DECIMAL(38, 2) NOT NULL, 
	escrow_delta DECIMAL(38, 2) NOT NULL, 
	balance_after DECIMAL(38, 2), 
	tx_type VARCHAR(50) NOT NULL, 
	description VARCHAR(200) NOT NULL, 
	reference_id VARCHAR(100), 
	memo VARCHAR(200), 
	counterparty_id VARCHAR(100) NOT NULL, 
	tx_group_id VARCHAR(100) NOT NULL, 
	principal_id UUID, 
	metadata JSONB, 
	escrow_after DECIMAL(38, 2), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (user_id), 
	FOREIGN KEY(principal_id) REFERENCES issued_principal (principal_id)
)


;


CREATE TABLE webhook_delivery (
	event_id UUID NOT NULL, 
	partner_user_id VARCHAR NOT NULL, 
	partner_client_id UUID NOT NULL, 
	event_type VARCHAR(128) NOT NULL, 
	resource_type VARCHAR(64) NOT NULL, 
	resource_id VARCHAR(64) NOT NULL, 
	payload JSONB NOT NULL, 
	status VARCHAR(32) NOT NULL, 
	qstash_message_id VARCHAR(255), 
	created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	sent_at TIMESTAMP WITH TIME ZONE, 
	delivered_at TIMESTAMP WITH TIME ZONE, 
	dead_at TIMESTAMP WITH TIME ZONE, 
	last_error VARCHAR(2000), 
	PRIMARY KEY (event_id), 
	FOREIGN KEY(partner_user_id) REFERENCES users (user_id)
)


;


CREATE TABLE websocket_connections (
	lock_id BIGSERIAL NOT NULL, 
	account_user_id VARCHAR NOT NULL, 
	connected_at TIMESTAMP WITH TIME ZONE NOT NULL, 
	last_heartbeat TIMESTAMP WITH TIME ZONE NOT NULL, 
	PRIMARY KEY (lock_id), 
	FOREIGN KEY(account_user_id) REFERENCES dzmm_account (user_id)
)


;


CREATE TABLE websocket_events (
	id BIGINT GENERATED BY DEFAULT AS IDENTITY, 
	timestamp TIMESTAMP WITH TIME ZONE NOT NULL, 
	user_id VARCHAR NOT NULL, 
	event VARCHAR NOT NULL, 
	data JSONB NOT NULL, 
	PRIMARY KEY (id, timestamp)
)
 PARTITION BY RANGE (timestamp)


;

