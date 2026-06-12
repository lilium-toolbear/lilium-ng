BEGIN;

CREATE TABLE alembic_version (
    version_num VARCHAR(32) NOT NULL, 
    CONSTRAINT alembic_version_pkc PRIMARY KEY (version_num)
);

-- Running upgrade  -> 2ba57397eb68

ALTER TABLE image_gps ALTER COLUMN timestamp TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE image_gps ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE messages ALTER COLUMN sent_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE messages ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE messages ALTER COLUMN updated_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE messages ALTER COLUMN deleted_at TYPE TIMESTAMP WITHOUT TIME ZONE;

DROP INDEX idx_messages_content_tsv;

ALTER TABLE messages DROP COLUMN content_tsv;

ALTER TABLE room_members ALTER COLUMN joined_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE room_members ALTER COLUMN last_read_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE room_members ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE room_members ALTER COLUMN updated_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE rooms ALTER COLUMN last_message_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE rooms ALTER COLUMN first_message_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE rooms ALTER COLUMN backfill_until TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE rooms ALTER COLUMN dissolved_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE rooms ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE rooms ALTER COLUMN updated_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE tweet_image_gps ALTER COLUMN timestamp TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE tweet_image_gps ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE tweets ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE tweets ALTER COLUMN updated_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE tweets ALTER COLUMN fetched_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE user_history ALTER COLUMN recorded_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE users ALTER COLUMN last_seen TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE users ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE;

ALTER TABLE users ALTER COLUMN updated_at TYPE TIMESTAMP WITHOUT TIME ZONE;

DROP INDEX idx_users_name_tsv;

ALTER TABLE users DROP COLUMN name_tsv;

INSERT INTO alembic_version (version_num) VALUES ('2ba57397eb68') RETURNING alembic_version.version_num;

-- Running upgrade 2ba57397eb68 -> 8b879ef11d3f

UPDATE alembic_version SET version_num='8b879ef11d3f' WHERE alembic_version.version_num = '2ba57397eb68';

-- Running upgrade 8b879ef11d3f -> e84ca55fa647

ALTER TABLE tweets ALTER COLUMN created_at SET NOT NULL;

UPDATE alembic_version SET version_num='e84ca55fa647' WHERE alembic_version.version_num = '8b879ef11d3f';

-- Running upgrade e84ca55fa647 -> f88dc21517d2

ALTER TABLE messages ALTER COLUMN metadata TYPE JSONB USING metadata::jsonb;

ALTER TABLE messages ALTER COLUMN raw_data TYPE JSONB USING raw_data::jsonb;

ALTER TABLE messages ALTER COLUMN history TYPE JSONB USING history::jsonb;

ALTER TABLE messages ALTER COLUMN reference_data TYPE JSONB USING reference_data::jsonb;

ALTER TABLE rooms ALTER COLUMN tags TYPE JSONB USING tags::jsonb;

ALTER TABLE rooms ALTER COLUMN raw_data TYPE JSONB USING raw_data::jsonb;

ALTER TABLE tweets ALTER COLUMN media_urls TYPE JSONB USING media_urls::jsonb;

ALTER TABLE tweets ALTER COLUMN local_media_paths TYPE JSONB USING local_media_paths::jsonb;

ALTER TABLE tweets ALTER COLUMN raw_data TYPE JSONB USING raw_data::jsonb;

ALTER TABLE user_history ALTER COLUMN metadata TYPE JSONB USING metadata::jsonb;

ALTER TABLE user_history ALTER COLUMN raw_data TYPE JSONB USING raw_data::jsonb;

ALTER TABLE users ALTER COLUMN metadata TYPE JSONB USING metadata::jsonb;

ALTER TABLE users ALTER COLUMN raw_data TYPE JSONB USING raw_data::jsonb;

UPDATE alembic_version SET version_num='f88dc21517d2' WHERE alembic_version.version_num = 'e84ca55fa647';

-- Running upgrade e84ca55fa647 -> 3b041151a7f8

DROP INDEX IF EXISTS idx_tweet_image_gps_coords;

DROP TABLE tweet_image_gps;

INSERT INTO alembic_version (version_num) VALUES ('3b041151a7f8') RETURNING alembic_version.version_num;

-- Running upgrade 3b041151a7f8 -> 0328471ceaaf

ALTER TABLE room_members DROP COLUMN is_muted;

ALTER TABLE room_members DROP COLUMN member_nickname;

ALTER TABLE room_members DROP COLUMN last_read_at;

UPDATE alembic_version SET version_num='0328471ceaaf' WHERE alembic_version.version_num = '3b041151a7f8';

-- Running upgrade 0328471ceaaf -> 5a4d532d0314

ALTER TABLE tweets DROP COLUMN user_name;

ALTER TABLE tweets DROP COLUMN user_avatar;

UPDATE alembic_version SET version_num='5a4d532d0314' WHERE alembic_version.version_num = '0328471ceaaf';

-- Running upgrade 5a4d532d0314, f88dc21517d2 -> a0dbf5e236e5

DELETE FROM alembic_version WHERE alembic_version.version_num = '5a4d532d0314';

UPDATE alembic_version SET version_num='a0dbf5e236e5' WHERE alembic_version.version_num = 'f88dc21517d2';

-- Running upgrade a0dbf5e236e5 -> 34f84a69972a

UPDATE tweets
        SET media_urls = media_urls::text::jsonb
        WHERE media_urls IS NOT NULL
        AND jsonb_typeof(media_urls) = 'string';

UPDATE tweets
        SET local_media_paths = local_media_paths::text::jsonb
        WHERE local_media_paths IS NOT NULL
        AND jsonb_typeof(local_media_paths) = 'string';

UPDATE tweets
        SET raw_data = raw_data::text::jsonb
        WHERE raw_data IS NOT NULL
        AND jsonb_typeof(raw_data) = 'string';

UPDATE alembic_version SET version_num='34f84a69972a' WHERE alembic_version.version_num = 'a0dbf5e236e5';

-- Running upgrade 34f84a69972a -> 6b3c6b0967e1

ALTER TABLE tweets
        ALTER COLUMN media_urls TYPE text[]
        USING CASE
            WHEN media_urls IS NULL OR jsonb_typeof(media_urls) = 'null' THEN NULL
            ELSE translate(media_urls::text, '[]', '{}')::text[]
        END;

ALTER TABLE tweets
        ALTER COLUMN local_media_paths TYPE text[]
        USING CASE
            WHEN local_media_paths IS NULL OR jsonb_typeof(local_media_paths) = 'null' THEN NULL
            ELSE translate(local_media_paths::text, '[]', '{}')::text[]
        END;

ALTER TABLE rooms
        ALTER COLUMN tags TYPE text[]
        USING CASE
            WHEN tags IS NULL OR jsonb_typeof(tags) = 'null' THEN NULL
            ELSE translate(tags::text, '[]', '{}')::text[]
        END;

UPDATE tweets SET media_urls = NULL WHERE media_urls = '{}';

UPDATE tweets SET local_media_paths = NULL WHERE local_media_paths = '{}';

UPDATE rooms SET tags = NULL WHERE tags = '{}';

UPDATE alembic_version SET version_num='6b3c6b0967e1' WHERE alembic_version.version_num = '34f84a69972a';

-- Running upgrade 6b3c6b0967e1 -> 70f5dac2d835

ALTER TABLE messages DROP COLUMN status;

UPDATE alembic_version SET version_num='70f5dac2d835' WHERE alembic_version.version_num = '6b3c6b0967e1';

-- Running upgrade 70f5dac2d835 -> 88f551b68add

CREATE TABLE dzmm_account (
    user_id VARCHAR NOT NULL, 
    user_profile JSONB NOT NULL, 
    email VARCHAR, 
    password VARCHAR, 
    signin_code VARCHAR, 
    cookies VARCHAR, 
    active BOOLEAN NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (user_id)
);

CREATE TABLE websocket_events (
    id SERIAL NOT NULL, 
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL, 
    user_id VARCHAR NOT NULL, 
    event VARCHAR NOT NULL, 
    data JSONB NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_websocket_events_event ON websocket_events (event);

CREATE INDEX ix_websocket_events_timestamp ON websocket_events (timestamp);

CREATE INDEX ix_websocket_events_user_id ON websocket_events (user_id);

CREATE TABLE websocket_connections (
    lock_id SERIAL NOT NULL, 
    account_user_id VARCHAR NOT NULL, 
    connected_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    last_heartbeat TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (lock_id), 
    FOREIGN KEY(account_user_id) REFERENCES dzmm_account (user_id)
);

CREATE INDEX ix_websocket_connections_account_user_id ON websocket_connections (account_user_id);

UPDATE alembic_version SET version_num='88f551b68add' WHERE alembic_version.version_num = '70f5dac2d835';

-- Running upgrade 88f551b68add -> a213ef8bedf6

CREATE TABLE event_processor_offsets (
    processor_id VARCHAR NOT NULL, 
    last_processed_id INTEGER NOT NULL, 
    last_processed_at TIMESTAMP WITH TIME ZONE, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (processor_id)
);

UPDATE alembic_version SET version_num='a213ef8bedf6' WHERE alembic_version.version_num = '88f551b68add';

-- Running upgrade a213ef8bedf6 -> f14717fc063d

CREATE OR REPLACE FUNCTION notify_websocket_event_inserted()
        RETURNS trigger AS $$
        BEGIN
            -- Send notification on 'websocket_event_inserted' channel
            -- Payload is the event ID for potential filtering
            PERFORM pg_notify('websocket_event_inserted', NEW.id::text);
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;;

CREATE TRIGGER websocket_event_inserted_trigger
        AFTER INSERT ON websocket_events
        FOR EACH ROW
        EXECUTE FUNCTION notify_websocket_event_inserted();;

UPDATE alembic_version SET version_num='f14717fc063d' WHERE alembic_version.version_num = 'a213ef8bedf6';

-- Running upgrade f14717fc063d -> e0c8e3d60e69

ALTER TABLE dzmm_account RENAME active TO is_enabled;

UPDATE alembic_version SET version_num='e0c8e3d60e69' WHERE alembic_version.version_num = 'f14717fc063d';

-- Running upgrade e0c8e3d60e69 -> 19b9fadc1bcb

ALTER TABLE messages ALTER COLUMN sent_by SET NOT NULL;

UPDATE alembic_version SET version_num='19b9fadc1bcb' WHERE alembic_version.version_num = 'e0c8e3d60e69';

-- Running upgrade 19b9fadc1bcb -> b6358b6a393d

ALTER TABLE messages ADD COLUMN is_edited BOOLEAN DEFAULT 'false' NOT NULL;

CREATE INDEX ix_messages_is_edited ON messages (is_edited);

UPDATE messages
        SET is_edited = true
        WHERE history IS NOT NULL
          AND history::jsonb != 'null'
          AND history::jsonb != '[]';

ALTER TABLE messages ALTER COLUMN is_edited DROP DEFAULT;

UPDATE alembic_version SET version_num='b6358b6a393d' WHERE alembic_version.version_num = '19b9fadc1bcb';

-- Running upgrade b6358b6a393d -> b8e8a1898aed

ALTER TABLE websocket_connections ALTER COLUMN lock_id TYPE BIGINT;

UPDATE alembic_version SET version_num='b8e8a1898aed' WHERE alembic_version.version_num = 'b6358b6a393d';

-- Running upgrade b8e8a1898aed -> 83bfe479ca45

CREATE OR REPLACE FUNCTION notify_message_inserted()
        RETURNS trigger AS $$
        BEGIN
            -- Send notification on 'message_inserted' channel
            -- Payload is the message ID for potential filtering
            PERFORM pg_notify('message_inserted', NEW.message_id);
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;;

CREATE TRIGGER message_inserted_trigger
        AFTER INSERT ON messages
        FOR EACH ROW
        EXECUTE FUNCTION notify_message_inserted();;

UPDATE alembic_version SET version_num='83bfe479ca45' WHERE alembic_version.version_num = 'b8e8a1898aed';

-- Running upgrade 83bfe479ca45 -> 0ae495f127a1

CREATE INDEX ix_messages_created_at ON messages (created_at);

UPDATE alembic_version SET version_num='0ae495f127a1' WHERE alembic_version.version_num = '83bfe479ca45';

-- Running upgrade 0ae495f127a1 -> 1c8d959c91ed

DROP INDEX ix_messages_created_at;

ALTER TABLE rooms ADD COLUMN account_ids TEXT[] DEFAULT '{}' NOT NULL;

UPDATE alembic_version SET version_num='1c8d959c91ed' WHERE alembic_version.version_num = '0ae495f127a1';

-- Running upgrade 1c8d959c91ed -> f8f783ef401f

CREATE INDEX idx_messages_source_created_at_id ON messages (source, created_at, message_id);

UPDATE alembic_version SET version_num='f8f783ef401f' WHERE alembic_version.version_num = '1c8d959c91ed';

-- Running upgrade f8f783ef401f -> c80c7ce78a4d

ALTER TABLE users ADD COLUMN avatar_file VARCHAR;

ALTER TABLE user_history ADD COLUMN avatar_file VARCHAR;

UPDATE alembic_version SET version_num='c80c7ce78a4d' WHERE alembic_version.version_num = 'f8f783ef401f';

-- Running upgrade c80c7ce78a4d -> b924516ff0a2

ALTER TABLE tweets ADD COLUMN parent_tweet_id VARCHAR;

ALTER TABLE tweets ADD COLUMN reply_to_tweet_id VARCHAR;

ALTER TABLE tweets ADD COLUMN reply_to_username VARCHAR;

ALTER TABLE tweets ADD COLUMN is_edited BOOLEAN DEFAULT 'false' NOT NULL;

ALTER TABLE tweets ADD COLUMN edit_history JSONB;

ALTER TABLE tweets ADD COLUMN post_id VARCHAR;

ALTER TABLE tweets ADD COLUMN draw_id VARCHAR;

CREATE INDEX ix_tweets_draw_id ON tweets (draw_id);

CREATE INDEX ix_tweets_parent_tweet_id ON tweets (parent_tweet_id);

CREATE INDEX ix_tweets_post_id ON tweets (post_id);

CREATE INDEX ix_tweets_reply_to_tweet_id ON tweets (reply_to_tweet_id);

UPDATE alembic_version SET version_num='b924516ff0a2' WHERE alembic_version.version_num = 'c80c7ce78a4d';

-- Running upgrade b924516ff0a2 -> e60ad00ac553

ALTER TABLE room_members ADD COLUMN left_at TIMESTAMP WITH TIME ZONE;

UPDATE alembic_version SET version_num='e60ad00ac553' WHERE alembic_version.version_num = 'b924516ff0a2';

-- Running upgrade e60ad00ac553 -> 73a6fb8cf0bb

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
);

CREATE INDEX ix_outgoing_commands_account_user_id ON outgoing_commands (account_user_id);

CREATE INDEX ix_outgoing_commands_created_at ON outgoing_commands (created_at);

CREATE INDEX ix_outgoing_commands_status ON outgoing_commands (status);

UPDATE alembic_version SET version_num='73a6fb8cf0bb' WHERE alembic_version.version_num = 'e60ad00ac553';

-- Running upgrade 73a6fb8cf0bb -> 697c00c6ed7c

CREATE OR REPLACE FUNCTION notify_outgoing_command_inserted()
        RETURNS trigger AS $$
        BEGIN
            -- Send notification on 'outgoing_command_inserted' channel
            -- Payload is JSON with command ID and account_user_id for filtering
            PERFORM pg_notify(
                'outgoing_command_inserted',
                json_build_object('id', NEW.id, 'account_user_id', NEW.account_user_id)::text
            );
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;;

CREATE TRIGGER outgoing_command_inserted_trigger
        AFTER INSERT ON outgoing_commands
        FOR EACH ROW
        EXECUTE FUNCTION notify_outgoing_command_inserted();;

UPDATE alembic_version SET version_num='697c00c6ed7c' WHERE alembic_version.version_num = '73a6fb8cf0bb';

-- Running upgrade 697c00c6ed7c -> 33dd21d5b61a

CREATE OR REPLACE FUNCTION notify_outgoing_command_updated()
        RETURNS trigger AS $$
        BEGIN
            -- Only notify if status actually changed
            IF OLD.status IS DISTINCT FROM NEW.status THEN
                -- Send notification on 'outgoing_command_updated' channel
                -- Payload is JSON with command ID, account_user_id, and new status
                PERFORM pg_notify(
                    'outgoing_command_updated',
                    json_build_object(
                        'id', NEW.id,
                        'account_user_id', NEW.account_user_id,
                        'status', NEW.status
                    )::text
                );
            END IF;
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;;

CREATE TRIGGER outgoing_command_updated_trigger
        AFTER UPDATE ON outgoing_commands
        FOR EACH ROW
        EXECUTE FUNCTION notify_outgoing_command_updated();;

UPDATE alembic_version SET version_num='33dd21d5b61a' WHERE alembic_version.version_num = '697c00c6ed7c';

-- Running upgrade 33dd21d5b61a -> 784715c4d470

CREATE INDEX idx_messages_room_id_sent_at ON messages (room_id, sent_at);

UPDATE alembic_version SET version_num='784715c4d470' WHERE alembic_version.version_num = '33dd21d5b61a';

-- Running upgrade 784715c4d470 -> 94b7387a3a16

CREATE TABLE bot_memory (
    id SERIAL NOT NULL, 
    namespace VARCHAR NOT NULL, 
    room_id VARCHAR, 
    user_id VARCHAR, 
    key VARCHAR NOT NULL, 
    value JSONB NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_bot_memory_expires ON bot_memory (expires_at);

CREATE INDEX ix_bot_memory_namespace ON bot_memory (namespace);

CREATE INDEX ix_bot_memory_namespace_room ON bot_memory (namespace, room_id);

CREATE INDEX ix_bot_memory_namespace_room_user ON bot_memory (namespace, room_id, user_id);

CREATE INDEX ix_bot_memory_room_id ON bot_memory (room_id);

CREATE INDEX ix_bot_memory_user_id ON bot_memory (user_id);

UPDATE alembic_version SET version_num='94b7387a3a16' WHERE alembic_version.version_num = '784715c4d470';

-- Running upgrade 94b7387a3a16 -> d5e3a02026bd

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
);

CREATE INDEX ix_books_created_at ON books (created_at);

CREATE INDEX ix_books_is_nsfw ON books (is_nsfw);

CREATE INDEX ix_books_likes_count ON books (likes_count);

CREATE INDEX ix_books_slug ON books (slug);

CREATE INDEX ix_books_user_id ON books (user_id);

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
);

CREATE INDEX ix_cards_created_at ON cards (created_at);

CREATE INDEX ix_cards_is_gamefy ON cards (is_gamefy);

CREATE INDEX ix_cards_likes_count ON cards (likes_count);

CREATE INDEX ix_cards_user_id ON cards (user_id);

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
);

CREATE INDEX ix_chapters_created_at ON chapters (created_at);

CREATE INDEX ix_chapters_is_adult ON chapters (is_adult);

CREATE INDEX ix_chapters_is_nsfw ON chapters (is_nsfw);

CREATE INDEX ix_chapters_likes_count ON chapters (likes_count);

CREATE INDEX ix_chapters_user_id ON chapters (user_id);

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
);

CREATE INDEX ix_checkpoints_created_at ON checkpoints (created_at);

CREATE INDEX ix_checkpoints_review_status ON checkpoints (review_status);

CREATE INDEX ix_checkpoints_share_code ON checkpoints (share_code);

CREATE INDEX ix_checkpoints_user_id ON checkpoints (user_id);

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
);

CREATE INDEX ix_galleries_created_at ON galleries (created_at);

CREATE INDEX ix_galleries_likes_count ON galleries (likes_count);

CREATE INDEX ix_galleries_user_id ON galleries (user_id);

UPDATE alembic_version SET version_num='d5e3a02026bd' WHERE alembic_version.version_num = '94b7387a3a16';

-- Running upgrade d5e3a02026bd -> 6cdc8b7329b8

ALTER TABLE dzmm_account ADD COLUMN signin_code_image BYTEA;

ALTER TABLE dzmm_account ADD COLUMN signin_code_image_mime VARCHAR;

UPDATE alembic_version SET version_num='6cdc8b7329b8' WHERE alembic_version.version_num = 'd5e3a02026bd';

-- Running upgrade 6cdc8b7329b8 -> ea0b662e14d4

SET statement_timeout = 0;

CREATE INDEX IF NOT EXISTS idx_messages_sent_by_sent_at_id ON messages (sent_by, sent_at, message_id);

UPDATE alembic_version SET version_num='ea0b662e14d4' WHERE alembic_version.version_num = '6cdc8b7329b8';

-- Running upgrade ea0b662e14d4 -> a62a5b528c7a

CREATE TABLE stock_account (
    user_id VARCHAR NOT NULL, 
    cash_balance DECIMAL(15, 2) NOT NULL, 
    total_realized_pnl DECIMAL(15, 2) NOT NULL, 
    trade_count INTEGER NOT NULL, 
    best_trade_pnl DECIMAL(15, 2) NOT NULL, 
    last_daily_claim DATE, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (user_id)
);

CREATE TYPE positiontype AS ENUM ('LONG', 'SHORT');

CREATE TABLE stock_portfolio (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    symbol VARCHAR NOT NULL, 
    position_type positiontype NOT NULL, 
    shares DECIMAL(15, 6) NOT NULL, 
    buy_price DECIMAL(15, 2) NOT NULL, 
    leverage INTEGER NOT NULL, 
    bought_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_stock_portfolio_symbol ON stock_portfolio (symbol);

CREATE INDEX ix_stock_portfolio_user_id ON stock_portfolio (user_id);

CREATE TYPE tradeaction AS ENUM ('BUY', 'SELL', 'SHORT', 'COVER', 'LIQUIDATED', 'STOP_LOSS', 'TAKE_PROFIT');

CREATE TABLE stock_trade_history (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    symbol VARCHAR NOT NULL, 
    action tradeaction NOT NULL, 
    shares DECIMAL(15, 6) NOT NULL, 
    price DECIMAL(15, 2) NOT NULL, 
    leverage INTEGER NOT NULL, 
    pnl DECIMAL(15, 2), 
    executed_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_stock_trade_history_symbol ON stock_trade_history (symbol);

CREATE INDEX ix_stock_trade_history_user_id ON stock_trade_history (user_id);

CREATE TYPE triggertype AS ENUM ('STOP_LOSS', 'TAKE_PROFIT');

CREATE TABLE stock_trigger (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    symbol VARCHAR NOT NULL, 
    position_type positiontype NOT NULL, 
    trigger_type triggertype NOT NULL, 
    trigger_price DECIMAL(15, 2) NOT NULL, 
    shares DECIMAL(15, 6), 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_stock_trigger_symbol ON stock_trigger (symbol);

CREATE INDEX ix_stock_trigger_user_id ON stock_trigger (user_id);

UPDATE alembic_version SET version_num='a62a5b528c7a' WHERE alembic_version.version_num = 'ea0b662e14d4';

-- Running upgrade a62a5b528c7a -> a3213cf998f7

ALTER TABLE stock_account ADD COLUMN total_claimed DECIMAL(15, 2) DEFAULT '0' NOT NULL;

UPDATE alembic_version SET version_num='a3213cf998f7' WHERE alembic_version.version_num = 'a62a5b528c7a';

-- Running upgrade a3213cf998f7 -> a93410803701

CREATE TABLE wallet (
    user_id VARCHAR NOT NULL, 
    balance DECIMAL(15, 2) NOT NULL, 
    last_daily_credit DATE, 
    total_credited DECIMAL(15, 2) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (user_id)
);

UPDATE alembic_version SET version_num='a93410803701' WHERE alembic_version.version_num = 'a3213cf998f7';

-- Running upgrade a93410803701 -> 2b13f785f11e

CREATE TABLE wallet_transaction (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    amount DECIMAL(15, 2) NOT NULL, 
    balance_after DECIMAL(15, 2) NOT NULL, 
    tx_type VARCHAR(50) NOT NULL, 
    description VARCHAR(200) NOT NULL, 
    reference_id VARCHAR(100), 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_wallet_transaction_user_id ON wallet_transaction (user_id);

UPDATE alembic_version SET version_num='2b13f785f11e' WHERE alembic_version.version_num = 'a93410803701';

-- Running upgrade 2b13f785f11e -> 658e8f561a0b

ALTER TABLE stock_account DROP COLUMN cash_balance;

ALTER TABLE stock_account DROP COLUMN last_daily_claim;

ALTER TABLE stock_account DROP COLUMN total_claimed;

UPDATE alembic_version SET version_num='658e8f561a0b' WHERE alembic_version.version_num = '2b13f785f11e';

-- Running upgrade 658e8f561a0b -> df7319985502

CREATE TABLE private_rooms (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    room_id VARCHAR NOT NULL, 
    pending_message_id VARCHAR, 
    pending_room_id VARCHAR, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX idx_private_rooms_room_id ON private_rooms (room_id);

CREATE UNIQUE INDEX ix_private_rooms_user_id ON private_rooms (user_id);

UPDATE alembic_version SET version_num='df7319985502' WHERE alembic_version.version_num = '658e8f561a0b';

-- Running upgrade df7319985502 -> 47d025f47f3a

ALTER TABLE private_rooms ADD COLUMN invite_link VARCHAR;

UPDATE alembic_version SET version_num='47d025f47f3a' WHERE alembic_version.version_num = 'df7319985502';

-- Running upgrade 47d025f47f3a -> ed5e378e9d17

ALTER TABLE private_rooms ADD COLUMN protected BOOLEAN DEFAULT 'true' NOT NULL;

ALTER TABLE private_rooms ALTER COLUMN protected DROP DEFAULT;

UPDATE alembic_version SET version_num='ed5e378e9d17' WHERE alembic_version.version_num = '47d025f47f3a';

-- Running upgrade ed5e378e9d17 -> aa58c597b3c0

ALTER TABLE bot_memory ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE;

UPDATE bot_memory SET updated_at = created_at WHERE updated_at IS NULL;

ALTER TABLE bot_memory ALTER COLUMN updated_at SET NOT NULL;

UPDATE alembic_version SET version_num='aa58c597b3c0' WHERE alembic_version.version_num = 'ed5e378e9d17';

-- Running upgrade aa58c597b3c0 -> 666b1acce57f

CREATE TABLE turnip_price (
    id SERIAL NOT NULL, 
    price DECIMAL(15, 2) NOT NULL, 
    trend VARCHAR(20) NOT NULL, 
    trend_hour INTEGER NOT NULL, 
    base_price DECIMAL(15, 2) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE TABLE turnip_inventory (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    quantity INTEGER NOT NULL, 
    buy_price DECIMAL(15, 2) NOT NULL, 
    purchased_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_turnip_inventory_user_id ON turnip_inventory (user_id);

UPDATE alembic_version SET version_num='666b1acce57f' WHERE alembic_version.version_num = 'aa58c597b3c0';

-- Running upgrade 666b1acce57f -> 10e4452c341e

ALTER TABLE turnip_price RENAME trend_hour TO trend_tick;

UPDATE alembic_version SET version_num='10e4452c341e' WHERE alembic_version.version_num = '666b1acce57f';

-- Running upgrade 10e4452c341e -> 5bdb3e59d1fe

ALTER TABLE wallet_transaction ADD COLUMN memo VARCHAR(200);

UPDATE alembic_version SET version_num='5bdb3e59d1fe' WHERE alembic_version.version_num = '10e4452c341e';

-- Running upgrade 5bdb3e59d1fe -> 5862a76be9ce

CREATE TABLE user_item (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    item_type VARCHAR(50) NOT NULL, 
    quantity INTEGER NOT NULL, 
    purchased_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE, 
    price_paid DECIMAL(15, 2) NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_user_item_item_type ON user_item (item_type);

CREATE INDEX ix_user_item_user_id ON user_item (user_id);

UPDATE alembic_version SET version_num='5862a76be9ce' WHERE alembic_version.version_num = '5bdb3e59d1fe';

-- Running upgrade 5862a76be9ce -> 0c844b478c6c

ALTER TABLE user_item ADD COLUMN activated_at TIMESTAMP WITH TIME ZONE;

UPDATE alembic_version SET version_num='0c844b478c6c' WHERE alembic_version.version_num = '5862a76be9ce';

-- Running upgrade 0c844b478c6c -> bda85d2c56e6

CREATE TABLE turnip_seed (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    quantity INTEGER NOT NULL, 
    seed_price DECIMAL(15, 2) NOT NULL, 
    planted_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    matures_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    fertilize_count INTEGER NOT NULL, 
    status VARCHAR(20) NOT NULL, 
    harvested_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_turnip_seed_status_matures ON turnip_seed (status, matures_at);

CREATE INDEX ix_turnip_seed_user_status ON turnip_seed (user_id, status);

UPDATE alembic_version SET version_num='bda85d2c56e6' WHERE alembic_version.version_num = '0c844b478c6c';

-- Running upgrade bda85d2c56e6 -> 356d46983f5f

CREATE TABLE market_maker_state (
    id SERIAL NOT NULL, 
    mid_price DECIMAL(15, 2) NOT NULL, 
    inventory INTEGER NOT NULL, 
    fair_value DECIMAL(15, 2) NOT NULL, 
    base_spread DECIMAL(10, 6) NOT NULL, 
    impact_factor DECIMAL(10, 6) NOT NULL, 
    skew_rate DECIMAL(10, 6) NOT NULL, 
    reversion_speed DECIMAL(10, 6) NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE TABLE trade_log (
    id SERIAL NOT NULL, 
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL, 
    price DECIMAL(15, 2) NOT NULL, 
    quantity INTEGER NOT NULL, 
    direction VARCHAR(4) NOT NULL, 
    trade_type VARCHAR(7) NOT NULL, 
    user_id VARCHAR(50), 
    mid_price_before DECIMAL(15, 2) NOT NULL, 
    mid_price_after DECIMAL(15, 2) NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_trade_log_timestamp ON trade_log (timestamp);

ALTER TABLE turnip_price ADD COLUMN open DECIMAL(15, 2);

ALTER TABLE turnip_price ADD COLUMN high DECIMAL(15, 2);

ALTER TABLE turnip_price ADD COLUMN low DECIMAL(15, 2);

ALTER TABLE turnip_price ADD COLUMN volume INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE turnip_price ADD COLUMN trade_count INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE turnip_price ADD COLUMN real_volume INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE turnip_price ADD COLUMN real_trade_count INTEGER DEFAULT '0' NOT NULL;

UPDATE turnip_price SET open = price, high = price, low = price WHERE open IS NULL;

UPDATE alembic_version SET version_num='356d46983f5f' WHERE alembic_version.version_num = 'bda85d2c56e6';

-- Running upgrade 356d46983f5f -> c7e3d25940cc

ALTER TABLE market_maker_state ADD COLUMN impact_strength DECIMAL(10, 6) DEFAULT '0.20' NOT NULL;

ALTER TABLE market_maker_state ADD COLUMN market_depth DECIMAL(15, 2) DEFAULT '500000' NOT NULL;

ALTER TABLE market_maker_state DROP COLUMN impact_factor;

UPDATE alembic_version SET version_num='c7e3d25940cc' WHERE alembic_version.version_num = '356d46983f5f';

-- Running upgrade c7e3d25940cc -> d8912bea78f1

ALTER TABLE market_maker_state ADD COLUMN depth_cash_ratio DECIMAL(10, 6) DEFAULT '0.02' NOT NULL;

UPDATE alembic_version SET version_num='d8912bea78f1' WHERE alembic_version.version_num = 'c7e3d25940cc';

-- Running upgrade d8912bea78f1 -> b5b7090b8c61

ALTER TABLE turnip_inventory ADD COLUMN settles_at TIMESTAMP WITH TIME ZONE;

UPDATE turnip_inventory
        SET settles_at = purchased_at
        WHERE settles_at IS NULL;

ALTER TABLE turnip_inventory ALTER COLUMN settles_at SET NOT NULL;

UPDATE alembic_version SET version_num='b5b7090b8c61' WHERE alembic_version.version_num = 'd8912bea78f1';

-- Running upgrade b5b7090b8c61 -> bf99afa289b6

ALTER TABLE stock_account ALTER COLUMN total_realized_pnl TYPE DECIMAL(24, 2);

ALTER TABLE stock_account ALTER COLUMN best_trade_pnl TYPE DECIMAL(24, 2);

ALTER TABLE stock_portfolio ALTER COLUMN shares TYPE DECIMAL(24, 6);

ALTER TABLE stock_portfolio ALTER COLUMN buy_price TYPE DECIMAL(24, 2);

ALTER TABLE stock_trade_history ALTER COLUMN shares TYPE DECIMAL(24, 6);

ALTER TABLE stock_trade_history ALTER COLUMN price TYPE DECIMAL(24, 2);

ALTER TABLE stock_trade_history ALTER COLUMN pnl TYPE DECIMAL(24, 2);

ALTER TABLE stock_trigger ALTER COLUMN shares TYPE DECIMAL(24, 6);

ALTER TABLE wallet ALTER COLUMN balance TYPE DECIMAL(24, 2);

ALTER TABLE wallet ALTER COLUMN total_credited TYPE DECIMAL(24, 2);

ALTER TABLE wallet_transaction ALTER COLUMN amount TYPE DECIMAL(24, 2);

ALTER TABLE wallet_transaction ALTER COLUMN balance_after TYPE DECIMAL(24, 2);

UPDATE alembic_version SET version_num='bf99afa289b6' WHERE alembic_version.version_num = 'b5b7090b8c61';

-- Running upgrade bf99afa289b6 -> c318e09d0852

CREATE TABLE user_achievement (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    achievement_key VARCHAR(100) NOT NULL, 
    unlocked_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    context JSONB, 
    PRIMARY KEY (id), 
    CONSTRAINT uq_user_achievement UNIQUE (user_id, achievement_key)
);

CREATE INDEX ix_user_achievement_achievement_key ON user_achievement (achievement_key);

CREATE INDEX ix_user_achievement_user_id ON user_achievement (user_id);

CREATE TABLE user_achievement_progress (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    stat_key VARCHAR(100) NOT NULL, 
    value INTEGER NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    CONSTRAINT uq_user_stat UNIQUE (user_id, stat_key)
);

CREATE INDEX ix_user_achievement_progress_stat_key ON user_achievement_progress (stat_key);

CREATE INDEX ix_user_achievement_progress_user_id ON user_achievement_progress (user_id);

UPDATE alembic_version SET version_num='c318e09d0852' WHERE alembic_version.version_num = 'bf99afa289b6';

-- Running upgrade c318e09d0852 -> c4ae0a1ffaad

ALTER TABLE user_achievement_progress ALTER COLUMN value TYPE BIGINT;

UPDATE alembic_version SET version_num='c4ae0a1ffaad' WHERE alembic_version.version_num = 'c318e09d0852';

-- Running upgrade c4ae0a1ffaad -> fb02b3fb4b0a

ALTER TABLE turnip_seed ADD COLUMN fertilize_cost DECIMAL(15, 2) DEFAULT '0' NOT NULL;

UPDATE alembic_version SET version_num='fb02b3fb4b0a' WHERE alembic_version.version_num = 'c4ae0a1ffaad';

-- Running upgrade fb02b3fb4b0a -> 082617c3cae4

ALTER TABLE market_maker_state DROP COLUMN skew_rate;

ALTER TABLE market_maker_state DROP COLUMN depth_cash_ratio;

ALTER TABLE market_maker_state DROP COLUMN base_spread;

ALTER TABLE market_maker_state DROP COLUMN market_depth;

ALTER TABLE market_maker_state DROP COLUMN impact_strength;

ALTER TABLE market_maker_state DROP COLUMN reversion_speed;

UPDATE alembic_version SET version_num='082617c3cae4' WHERE alembic_version.version_num = 'fb02b3fb4b0a';

-- Running upgrade 082617c3cae4 -> f2d3925f30ce

ALTER TABLE market_maker_state ALTER COLUMN inventory TYPE BIGINT;

UPDATE alembic_version SET version_num='f2d3925f30ce' WHERE alembic_version.version_num = '082617c3cae4';

-- Running upgrade f2d3925f30ce -> 8137b93968f7

ALTER TABLE trade_log ALTER COLUMN quantity TYPE BIGINT;

UPDATE alembic_version SET version_num='8137b93968f7' WHERE alembic_version.version_num = 'f2d3925f30ce';

-- Running upgrade 8137b93968f7 -> 55a48c564bb6

ALTER TABLE turnip_inventory ALTER COLUMN quantity TYPE BIGINT;

ALTER TABLE turnip_price ALTER COLUMN volume TYPE BIGINT;

ALTER TABLE turnip_price ALTER COLUMN trade_count TYPE BIGINT;

ALTER TABLE turnip_price ALTER COLUMN real_volume TYPE BIGINT;

ALTER TABLE turnip_price ALTER COLUMN real_trade_count TYPE BIGINT;

UPDATE alembic_version SET version_num='55a48c564bb6' WHERE alembic_version.version_num = '8137b93968f7';

-- Running upgrade 55a48c564bb6 -> 39294f6ee32c

CREATE TABLE land (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    land_type VARCHAR(20) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_land_user_id ON land (user_id);

CREATE INDEX ix_land_user_type ON land (user_id, land_type);

ALTER TABLE turnip_inventory ADD COLUMN is_stored BOOLEAN DEFAULT false NOT NULL;

ALTER TABLE turnip_inventory ADD COLUMN stored_shelf_life_seconds DECIMAL(15, 2);

UPDATE alembic_version SET version_num='39294f6ee32c' WHERE alembic_version.version_num = '55a48c564bb6';

-- Running upgrade 39294f6ee32c -> 0da9b3168115

CREATE TABLE uno_game (
    id VARCHAR NOT NULL, 
    room_id VARCHAR NOT NULL, 
    creator_id VARCHAR NOT NULL, 
    players JSON NOT NULL, 
    status VARCHAR NOT NULL, 
    result JSON, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    finished_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_uno_game_room_id ON uno_game (room_id);

UPDATE alembic_version SET version_num='0da9b3168115' WHERE alembic_version.version_num = '39294f6ee32c';

-- Running upgrade 0da9b3168115 -> 0c9b4b97aa7d

ALTER TABLE uno_game ADD COLUMN rematch_from_id VARCHAR;

CREATE INDEX ix_uno_game_rematch_from_id ON uno_game (rematch_from_id);

UPDATE alembic_version SET version_num='0c9b4b97aa7d' WHERE alembic_version.version_num = '0da9b3168115';

-- Running upgrade 0c9b4b97aa7d -> bea9ea4c8eda

ALTER TABLE turnip_price ADD COLUMN cycle_ticks INTEGER;

UPDATE alembic_version SET version_num='bea9ea4c8eda' WHERE alembic_version.version_num = '0c9b4b97aa7d';

-- Running upgrade bea9ea4c8eda -> d831ca25a703

CREATE TABLE pal (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    species_code VARCHAR(100) NOT NULL, 
    custom_name VARCHAR(100), 
    rarity INTEGER NOT NULL, 
    dormitory_id INTEGER, 
    hatched_from_egg_id BIGINT, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(dormitory_id) REFERENCES land (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_pal_user_id ON pal (user_id);

CREATE INDEX ix_pal_user_species ON pal (user_id, species_code);

CREATE TABLE pal_egg (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    egg_tier INTEGER NOT NULL, 
    element VARCHAR(20), 
    status VARCHAR(20) NOT NULL, 
    price_paid DECIMAL(20, 2) NOT NULL, 
    dormitory_id INTEGER, 
    hatching_started_at TIMESTAMP WITH TIME ZONE, 
    hatches_at TIMESTAMP WITH TIME ZONE, 
    hatched_pal_id BIGINT, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(dormitory_id) REFERENCES land (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_pal_egg_user_id ON pal_egg (user_id);

CREATE INDEX ix_pal_egg_user_status ON pal_egg (user_id, status);

UPDATE alembic_version SET version_num='d831ca25a703' WHERE alembic_version.version_num = 'bea9ea4c8eda';

-- Running upgrade d831ca25a703 -> f6fe5e55b610

ALTER TABLE pal ADD COLUMN gender VARCHAR(10);

UPDATE pal
        SET gender = CASE WHEN random() < 0.5 THEN 'male' ELSE 'female' END
        WHERE gender IS NULL;

ALTER TABLE pal ALTER COLUMN gender SET NOT NULL;

UPDATE alembic_version SET version_num='f6fe5e55b610' WHERE alembic_version.version_num = 'd831ca25a703';

-- Running upgrade f6fe5e55b610 -> 6f0a2d9c1b7a

UPDATE land
        SET land_type = lower(split_part(land_type, '.', 2))
        WHERE land_type ILIKE 'landtype.%';

UPDATE alembic_version SET version_num='6f0a2d9c1b7a' WHERE alembic_version.version_num = 'f6fe5e55b610';

-- Running upgrade 6f0a2d9c1b7a -> 7b6a2f4c1a6e

ALTER TABLE pal DROP CONSTRAINT IF EXISTS pal_dormitory_id_fkey;

ALTER TABLE pal_egg DROP CONSTRAINT IF EXISTS pal_egg_dormitory_id_fkey;

ALTER TABLE pal DROP COLUMN dormitory_id;

ALTER TABLE pal_egg DROP COLUMN dormitory_id;

UPDATE alembic_version SET version_num='7b6a2f4c1a6e' WHERE alembic_version.version_num = '6f0a2d9c1b7a';

-- Running upgrade 7b6a2f4c1a6e -> e423908bad35

ALTER TABLE pal ADD COLUMN breeding_cooldown_until TIMESTAMP WITH TIME ZONE;

ALTER TABLE pal_egg ADD COLUMN breeding_started_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE pal_egg ADD COLUMN breeding_ready_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE pal_egg ADD COLUMN parent1_id BIGINT;

ALTER TABLE pal_egg ADD COLUMN parent2_id BIGINT;

ALTER TABLE pal_egg ADD COLUMN offspring_species VARCHAR(100);

ALTER TABLE pal_egg ADD COLUMN is_special_combo BOOLEAN DEFAULT false NOT NULL;

ALTER TABLE pal_egg ALTER COLUMN is_special_combo DROP DEFAULT;

UPDATE alembic_version SET version_num='e423908bad35' WHERE alembic_version.version_num = '7b6a2f4c1a6e';

-- Running upgrade e423908bad35 -> c938cd639b35

CREATE TABLE land_assignment (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    land_id INTEGER NOT NULL, 
    pal_id BIGINT NOT NULL, 
    role VARCHAR(30) NOT NULL, 
    assigned_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    released_at TIMESTAMP WITH TIME ZONE, 
    last_tick_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id), 
    FOREIGN KEY(land_id) REFERENCES land (id), 
    FOREIGN KEY(pal_id) REFERENCES pal (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_land_assignment_land_id ON land_assignment (land_id);

CREATE UNIQUE INDEX ix_land_assignment_pal_active ON land_assignment (pal_id) WHERE released_at IS NULL;

CREATE INDEX ix_land_assignment_pal_id ON land_assignment (pal_id);

CREATE INDEX ix_land_assignment_user_active ON land_assignment (user_id, released_at);

CREATE INDEX ix_land_assignment_user_id ON land_assignment (user_id);

ALTER TABLE land ADD COLUMN upgrade_level INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE land ADD COLUMN upgrade_work_total BIGINT DEFAULT '0' NOT NULL;

ALTER TABLE land ADD COLUMN upgrade_work_done BIGINT DEFAULT '0' NOT NULL;

ALTER TABLE land ADD COLUMN upgrade_status VARCHAR(20) DEFAULT 'idle' NOT NULL;

ALTER TABLE land ADD COLUMN upgrade_started_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE land ADD COLUMN upgrade_completed_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE pal ADD COLUMN level INTEGER DEFAULT '1' NOT NULL;

ALTER TABLE pal ADD COLUMN exp INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE land ALTER COLUMN upgrade_level DROP DEFAULT;

ALTER TABLE land ALTER COLUMN upgrade_work_total DROP DEFAULT;

ALTER TABLE land ALTER COLUMN upgrade_work_done DROP DEFAULT;

ALTER TABLE land ALTER COLUMN upgrade_status DROP DEFAULT;

ALTER TABLE pal ALTER COLUMN level DROP DEFAULT;

ALTER TABLE pal ALTER COLUMN exp DROP DEFAULT;

UPDATE alembic_version SET version_num='c938cd639b35' WHERE alembic_version.version_num = 'e423908bad35';

-- Running upgrade c938cd639b35 -> 82de9d39e527

ALTER TABLE land_assignment ADD COLUMN consumption_remainder FLOAT DEFAULT '0.0' NOT NULL;

UPDATE alembic_version SET version_num='82de9d39e527' WHERE alembic_version.version_num = 'c938cd639b35';

-- Running upgrade 82de9d39e527 -> f7b9c8f7c7ae

CREATE TABLE turnip_transaction (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    quantity BIGINT NOT NULL, 
    balance_after BIGINT NOT NULL, 
    tx_type VARCHAR(20) NOT NULL, 
    unit_price DECIMAL(15, 2) NOT NULL, 
    description VARCHAR(200) NOT NULL, 
    inventory_id INTEGER, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_turnip_transaction_user_id ON turnip_transaction (user_id);

UPDATE alembic_version SET version_num='f7b9c8f7c7ae' WHERE alembic_version.version_num = '82de9d39e527';

-- Running upgrade f7b9c8f7c7ae -> 50efe9f00e82

ALTER TABLE land_assignment ADD COLUMN assignment_type VARCHAR(20);

UPDATE land_assignment la
        SET assignment_type = CASE
            WHEN la.role = 'farm_bonus' THEN 'farm'
            WHEN la.role = 'warehouse_boost' THEN 'warehouse'
            WHEN la.role = 'land_upgrade' THEN 'upgrade'
            WHEN la.role = 'mine_work' THEN 'mine'
            WHEN la.role = 'lumber_work' THEN 'lumber_mill'
            WHEN la.role = 'workshop_work' THEN 'workshop'
            ELSE 'farm'
        END;

ALTER TABLE land_assignment ALTER COLUMN assignment_type SET NOT NULL;

DROP INDEX ix_land_assignment_land_id;

ALTER TABLE land_assignment DROP CONSTRAINT land_assignment_land_id_fkey;

ALTER TABLE land_assignment DROP COLUMN land_id;

ALTER TABLE land_assignment DROP COLUMN role;

UPDATE alembic_version SET version_num='50efe9f00e82' WHERE alembic_version.version_num = 'f7b9c8f7c7ae';

-- Running upgrade 50efe9f00e82 -> 2de0dec8d2c1

CREATE INDEX ix_land_assignment_type ON land_assignment (user_id, assignment_type);

ALTER TABLE turnip_transaction ADD COLUMN inventory_ids INTEGER[];

UPDATE turnip_transaction
        SET inventory_ids = ARRAY[inventory_id]
        WHERE inventory_id IS NOT NULL;

ALTER TABLE turnip_transaction DROP COLUMN inventory_id;

UPDATE alembic_version SET version_num='2de0dec8d2c1' WHERE alembic_version.version_num = '50efe9f00e82';

-- Running upgrade 2de0dec8d2c1 -> ba4503a662d5

CREATE TABLE llm_usage_log (
    id VARCHAR NOT NULL, 
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
);

CREATE INDEX ix_llm_usage_log_created_at ON llm_usage_log (created_at);

CREATE INDEX ix_llm_usage_log_user_id ON llm_usage_log (user_id);

UPDATE alembic_version SET version_num='ba4503a662d5' WHERE alembic_version.version_num = '2de0dec8d2c1';

-- Running upgrade ba4503a662d5 -> fa291ef99b44

CREATE TABLE resource_production (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    land_id BIGINT NOT NULL, 
    accumulated_credits DECIMAL(20, 2) NOT NULL, 
    last_tick_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    is_paused BOOLEAN NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(land_id) REFERENCES land (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_resource_production_paused ON resource_production (is_paused);

CREATE UNIQUE INDEX ix_resource_production_user_land ON resource_production (user_id, land_id);

UPDATE alembic_version SET version_num='fa291ef99b44' WHERE alembic_version.version_num = 'ba4503a662d5';

-- Running upgrade ba4503a662d5 -> c8d9e5a1b2c3

CREATE TABLE game_escrow (
    escrow_token VARCHAR NOT NULL, 
    game_id VARCHAR NOT NULL, 
    game_type VARCHAR NOT NULL, 
    status VARCHAR NOT NULL, 
    locked_funds JSONB NOT NULL, 
    total_locked FLOAT NOT NULL, 
    settled_payouts JSONB, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    settled_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (escrow_token)
);

CREATE INDEX ix_game_escrow_game_id ON game_escrow (game_id);

INSERT INTO alembic_version (version_num) VALUES ('c8d9e5a1b2c3') RETURNING alembic_version.version_num;

-- Running upgrade c8d9e5a1b2c3, fa291ef99b44 -> 17bf1191104d

DELETE FROM alembic_version WHERE alembic_version.version_num = 'c8d9e5a1b2c3';

UPDATE alembic_version SET version_num='17bf1191104d' WHERE alembic_version.version_num = 'fa291ef99b44';

-- Running upgrade 17bf1191104d -> 44328ece83dc

CREATE TABLE games (
    id VARCHAR NOT NULL, 
    game_type VARCHAR NOT NULL, 
    room_id VARCHAR NOT NULL, 
    creator_id VARCHAR NOT NULL, 
    status VARCHAR NOT NULL, 
    config JSONB NOT NULL, 
    players JSONB NOT NULL, 
    result JSONB, 
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL, 
    started_at TIMESTAMP WITHOUT TIME ZONE, 
    finished_at TIMESTAMP WITHOUT TIME ZONE, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_games_created_at ON games (created_at);

CREATE INDEX ix_games_creator_id ON games (creator_id);

CREATE INDEX ix_games_game_type ON games (game_type);

CREATE INDEX ix_games_room_id ON games (room_id);

CREATE INDEX ix_games_status ON games (status);

ALTER TABLE game_escrow ALTER COLUMN locked_funds TYPE JSON;

ALTER TABLE game_escrow ALTER COLUMN settled_payouts TYPE JSON;

UPDATE alembic_version SET version_num='44328ece83dc' WHERE alembic_version.version_num = '17bf1191104d';

-- Running upgrade 44328ece83dc -> 895addd1b821

INSERT INTO games (id, game_type, room_id, creator_id, status,
                          config, players, result, created_at, started_at, finished_at)
        SELECT id, 'uno', room_id, creator_id, status,
               '{}'::jsonb, COALESCE(players::jsonb, '[]'::jsonb), result::jsonb,
               created_at, NULL, finished_at
        FROM uno_game
        WHERE NOT EXISTS (SELECT 1 FROM games WHERE games.id = uno_game.id);

UPDATE alembic_version SET version_num='895addd1b821' WHERE alembic_version.version_num = '44328ece83dc';

-- Running upgrade 895addd1b821 -> ee5f36c139fd

ALTER TABLE market_maker_state ADD COLUMN user_pressure BIGINT;

UPDATE market_maker_state SET user_pressure = 0 WHERE user_pressure IS NULL;

ALTER TABLE market_maker_state ALTER COLUMN user_pressure SET NOT NULL;

UPDATE alembic_version SET version_num='ee5f36c139fd' WHERE alembic_version.version_num = '895addd1b821';

-- Running upgrade ee5f36c139fd -> bb22615af5e3

ALTER TABLE turnip_price ADD COLUMN cycle_context JSON;

UPDATE alembic_version SET version_num='bb22615af5e3' WHERE alembic_version.version_num = 'ee5f36c139fd';

-- Running upgrade bb22615af5e3 -> b822dc671776

ALTER TABLE games ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';

ALTER TABLE games ALTER COLUMN started_at TYPE TIMESTAMPTZ USING started_at AT TIME ZONE 'UTC';

ALTER TABLE games ALTER COLUMN finished_at TYPE TIMESTAMPTZ USING finished_at AT TIME ZONE 'UTC';

UPDATE alembic_version SET version_num='b822dc671776' WHERE alembic_version.version_num = 'bb22615af5e3';

-- Running upgrade b822dc671776 -> bad2a7525199

DROP INDEX ix_trade_log_timestamp;

DROP TABLE trade_log;

ALTER TABLE turnip_price DROP COLUMN real_trade_count;

ALTER TABLE turnip_price DROP COLUMN real_volume;

UPDATE alembic_version SET version_num='bad2a7525199' WHERE alembic_version.version_num = 'b822dc671776';

-- Running upgrade bad2a7525199 -> 69911c7bf89a

ALTER TABLE turnip_inventory ADD COLUMN is_harvested BOOLEAN DEFAULT false NOT NULL;

UPDATE alembic_version SET version_num='69911c7bf89a' WHERE alembic_version.version_num = 'bad2a7525199';

-- Running upgrade 69911c7bf89a -> e05e6649b757

CREATE TABLE turnip_order (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    side VARCHAR(10) NOT NULL, 
    quantity BIGINT NOT NULL, 
    filled_quantity BIGINT NOT NULL, 
    limit_price DECIMAL(15, 2) NOT NULL, 
    escrow_amount DECIMAL(15, 2) NOT NULL, 
    status VARCHAR(20) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    filled_at TIMESTAMP WITH TIME ZONE, 
    cancelled_at TIMESTAMP WITH TIME ZONE, 
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_turnip_order_user_id ON turnip_order (user_id);

ALTER TABLE turnip_inventory ADD COLUMN locked_for_order_id INTEGER;

CREATE INDEX ix_turnip_inventory_locked_for_order_id ON turnip_inventory (locked_for_order_id);

ALTER TABLE turnip_inventory ADD FOREIGN KEY(locked_for_order_id) REFERENCES turnip_order (id);

UPDATE alembic_version SET version_num='e05e6649b757' WHERE alembic_version.version_num = '69911c7bf89a';

-- Running upgrade e05e6649b757 -> f0a67b05fb83

CREATE TABLE turnip_order_fill (
    id SERIAL NOT NULL, 
    order_id INTEGER NOT NULL, 
    fill_type VARCHAR(10) NOT NULL, 
    counterparty_order_id INTEGER, 
    quantity BIGINT NOT NULL, 
    price DECIMAL(15, 2) NOT NULL, 
    total DECIMAL(24, 2) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(order_id) REFERENCES turnip_order (id)
);

CREATE INDEX ix_turnip_order_fill_order_id ON turnip_order_fill (order_id);

ALTER TABLE turnip_order ADD COLUMN order_type VARCHAR(10) DEFAULT 'limit' NOT NULL;

ALTER TABLE turnip_order ADD COLUMN quote_price DECIMAL(15, 2);

ALTER TABLE turnip_order ADD COLUMN execution_price DECIMAL(15, 2);

ALTER TABLE turnip_order ADD COLUMN slippage_pct FLOAT;

ALTER TABLE turnip_order ALTER COLUMN escrow_amount TYPE DECIMAL(24, 2);

UPDATE alembic_version SET version_num='f0a67b05fb83' WHERE alembic_version.version_num = 'e05e6649b757';

-- Running upgrade f0a67b05fb83 -> 1f6cade33b82

ALTER TABLE land ALTER COLUMN upgrade_work_done TYPE DOUBLE PRECISION;

UPDATE alembic_version SET version_num='1f6cade33b82' WHERE alembic_version.version_num = 'f0a67b05fb83';

-- Running upgrade 1f6cade33b82 -> b29456bcbadf

ALTER TABLE turnip_order ALTER COLUMN escrow_amount TYPE DECIMAL(24, 2);

ALTER TABLE turnip_order_fill ALTER COLUMN total TYPE DECIMAL(24, 2);

UPDATE alembic_version SET version_num='b29456bcbadf' WHERE alembic_version.version_num = '1f6cade33b82';

-- Running upgrade b29456bcbadf -> 77e45ca29f69

UPDATE land_assignment
        SET released_at = NOW()
        WHERE released_at IS NULL
          AND pal_id IN (
              SELECT la.pal_id
              FROM land_assignment la
              JOIN pal p ON la.pal_id = p.id
              WHERE la.released_at IS NULL
                AND p.user_id != la.user_id
          );

UPDATE alembic_version SET version_num='77e45ca29f69' WHERE alembic_version.version_num = 'b29456bcbadf';

-- Running upgrade 77e45ca29f69 -> b016ad3197a1

ALTER TABLE market_maker_state ADD COLUMN amm_book JSON;

UPDATE alembic_version SET version_num='b016ad3197a1' WHERE alembic_version.version_num = '77e45ca29f69';

-- Running upgrade b016ad3197a1 -> 7dc9eb08ab4a

ALTER TABLE turnip_seed ALTER COLUMN quantity TYPE BIGINT;

UPDATE alembic_version SET version_num='7dc9eb08ab4a' WHERE alembic_version.version_num = 'b016ad3197a1';

-- Running upgrade 7dc9eb08ab4a -> 4db539e6e26b

DROP INDEX IF EXISTS ix_uno_game_room_id;

DROP INDEX IF EXISTS ix_uno_game_rematch_from_id;

DROP TABLE IF EXISTS uno_game;

UPDATE alembic_version SET version_num='4db539e6e26b' WHERE alembic_version.version_num = '7dc9eb08ab4a';

-- Running upgrade 4db539e6e26b -> 1ba97b3f7b77

CREATE TABLE trpg_checkpoints (
    id SERIAL NOT NULL, 
    room_id VARCHAR NOT NULL, 
    name VARCHAR NOT NULL, 
    state JSONB NOT NULL, 
    saved_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    UNIQUE (room_id, name)
);

CREATE INDEX ix_trpg_checkpoints_room_id ON trpg_checkpoints (room_id);

CREATE TABLE trpg_rooms (
    room_id VARCHAR NOT NULL, 
    background VARCHAR NOT NULL, 
    mission VARCHAR NOT NULL, 
    rule VARCHAR NOT NULL, 
    scene VARCHAR NOT NULL, 
    summary VARCHAR NOT NULL, 
    turns JSONB DEFAULT '[]' NOT NULL, 
    pcs JSONB DEFAULT '{}' NOT NULL, 
    npcs JSONB DEFAULT '{}' NOT NULL, 
    bags JSONB DEFAULT '{}' NOT NULL, 
    bag_logs JSONB DEFAULT '[]' NOT NULL, 
    undo_stack JSONB DEFAULT '[]' NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (room_id)
);

SELECT room_id, value FROM bot_memory WHERE namespace = 'trpg_state' AND key = 'state';

UPDATE alembic_version SET version_num='1ba97b3f7b77' WHERE alembic_version.version_num = '4db539e6e26b';

-- Running upgrade 1ba97b3f7b77 -> a7f3c2d1e8b9

CREATE TABLE trpg_games (
    id SERIAL NOT NULL, 
    room_id VARCHAR NOT NULL, 
    started_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
    ended_at TIMESTAMP WITH TIME ZONE, 
    background VARCHAR DEFAULT '' NOT NULL, 
    mission VARCHAR DEFAULT '' NOT NULL, 
    rule VARCHAR DEFAULT '' NOT NULL, 
    scene VARCHAR DEFAULT '' NOT NULL, 
    summary VARCHAR DEFAULT '' NOT NULL, 
    turns JSONB DEFAULT '[]' NOT NULL, 
    pcs JSONB DEFAULT '{}' NOT NULL, 
    npcs JSONB DEFAULT '{}' NOT NULL, 
    bags JSONB DEFAULT '{}' NOT NULL, 
    bag_logs JSONB DEFAULT '[]' NOT NULL, 
    undo_stack JSONB DEFAULT '[]' NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_trpg_games_room_id ON trpg_games (room_id);

INSERT INTO trpg_games (room_id, started_at, ended_at,
            background, mission, rule, scene, summary,
            turns, pcs, npcs, bags, bag_logs, undo_stack,
            created_at, updated_at)
        SELECT room_id, created_at, NULL,
            background, mission, rule, scene, summary,
            turns, pcs, npcs, bags, bag_logs, undo_stack,
            created_at, updated_at
        FROM trpg_rooms;

ALTER TABLE trpg_checkpoints ADD COLUMN game_id INTEGER;

UPDATE trpg_checkpoints
        SET game_id = (
            SELECT id FROM trpg_games
            WHERE trpg_games.room_id = trpg_checkpoints.room_id
            LIMIT 1
        );

ALTER TABLE trpg_checkpoints DROP CONSTRAINT trpg_checkpoints_room_id_name_key;

DROP INDEX ix_trpg_checkpoints_room_id;

ALTER TABLE trpg_checkpoints DROP COLUMN room_id;

ALTER TABLE trpg_checkpoints ALTER COLUMN game_id SET NOT NULL;

CREATE INDEX ix_trpg_checkpoints_game_id ON trpg_checkpoints (game_id);

ALTER TABLE trpg_checkpoints ADD CONSTRAINT trpg_checkpoints_game_id_name_key UNIQUE (game_id, name);

DROP TABLE trpg_rooms;

UPDATE alembic_version SET version_num='a7f3c2d1e8b9' WHERE alembic_version.version_num = '1ba97b3f7b77';

-- Running upgrade a7f3c2d1e8b9 -> b222fa06d87a

ALTER TABLE trpg_games ADD COLUMN summarized_turn_count INTEGER DEFAULT '0' NOT NULL;

UPDATE alembic_version SET version_num='b222fa06d87a' WHERE alembic_version.version_num = 'a7f3c2d1e8b9';

-- Running upgrade b222fa06d87a -> 19efcedadde3

UPDATE land
        SET upgrade_work_total = 8000 * POWER(2, upgrade_level)::bigint
        WHERE upgrade_status = 'upgrading'
          AND upgrade_work_total != 8000 * POWER(2, upgrade_level)::bigint;

UPDATE land
        SET upgrade_level = upgrade_level + 1,
            upgrade_status = 'idle',
            upgrade_work_done = 0,
            upgrade_work_total = 0,
            upgrade_completed_at = NOW()
        WHERE upgrade_status = 'upgrading'
          AND upgrade_work_done >= upgrade_work_total
          AND upgrade_work_total > 0;

UPDATE alembic_version SET version_num='19efcedadde3' WHERE alembic_version.version_num = 'b222fa06d87a';

-- Running upgrade 19efcedadde3 -> d77182b10cfa

ALTER TABLE market_maker_state ADD COLUMN recent_trades JSON;

UPDATE alembic_version SET version_num='d77182b10cfa' WHERE alembic_version.version_num = '19efcedadde3';

-- Running upgrade d77182b10cfa -> 8020789bbd59

CREATE TABLE battle_records (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    mode VARCHAR(10) NOT NULL, 
    rounds_cleared INTEGER NOT NULL, 
    pal_ids TEXT NOT NULL, 
    pal_levels TEXT NOT NULL, 
    exp_earned INTEGER DEFAULT '0' NOT NULL, 
    credits_earned INTEGER DEFAULT '0' NOT NULL, 
    battle_log TEXT DEFAULT '{}' NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_battle_records_user_id ON battle_records (user_id);

UPDATE alembic_version SET version_num='8020789bbd59' WHERE alembic_version.version_num = 'd77182b10cfa';

-- Running upgrade 8020789bbd59 -> a775e2139915

ALTER TABLE pal ADD COLUMN revival_until TIMESTAMP WITH TIME ZONE;

UPDATE alembic_version SET version_num='a775e2139915' WHERE alembic_version.version_num = '8020789bbd59';

-- Running upgrade a775e2139915 -> be732125ba59

ALTER TABLE battle_records ADD COLUMN active_state JSONB;

UPDATE alembic_version SET version_num='be732125ba59' WHERE alembic_version.version_num = 'a775e2139915';

-- Running upgrade be732125ba59 -> cfe3b0a3cd43

CREATE TABLE user_notification (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    category VARCHAR NOT NULL, 
    content VARCHAR NOT NULL, 
    is_read BOOLEAN NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_user_notification_unread ON user_notification (user_id, is_read);

CREATE INDEX ix_user_notification_user_id ON user_notification (user_id);

UPDATE alembic_version SET version_num='cfe3b0a3cd43' WHERE alembic_version.version_num = 'be732125ba59';

-- Running upgrade cfe3b0a3cd43 -> f7e537b63dc2

CREATE TYPE positionside AS ENUM ('LONG', 'SHORT');

CREATE TYPE positionstatus AS ENUM ('OPEN', 'LIQUIDATED', 'CLOSED');

CREATE TABLE futuresposition (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    side positionside NOT NULL, 
    quantity INTEGER NOT NULL, 
    entry_price DECIMAL(24, 2) NOT NULL, 
    margin DECIMAL(24, 2) NOT NULL, 
    liquidation_price DECIMAL(24, 2) NOT NULL, 
    status positionstatus NOT NULL, 
    realized_pnl DECIMAL(24, 2) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    closed_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_futuresposition_status ON futuresposition (status);

CREATE INDEX ix_futuresposition_user_id ON futuresposition (user_id);

CREATE TYPE futurestransactiontype AS ENUM ('OPEN', 'CLOSE', 'LIQUIDATION', 'FUNDING', 'MARGIN_ADD');

CREATE TABLE futurestransaction (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    position_id INTEGER, 
    tx_type futurestransactiontype NOT NULL, 
    quantity INTEGER NOT NULL, 
    price DECIMAL(24, 2) NOT NULL, 
    pnl DECIMAL(24, 2) NOT NULL, 
    fee DECIMAL(24, 2) NOT NULL, 
    margin_change DECIMAL(24, 2) NOT NULL, 
    description VARCHAR NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_futurestransaction_created_at ON futurestransaction (created_at);

CREATE INDEX ix_futurestransaction_position_id ON futurestransaction (position_id);

CREATE INDEX ix_futurestransaction_user_id ON futurestransaction (user_id);

ALTER TABLE market_maker_state ADD COLUMN futures_vamm_x DECIMAL(24, 2);

ALTER TABLE market_maker_state ADD COLUMN futures_vamm_y DECIMAL(24, 2);

ALTER TABLE market_maker_state ADD COLUMN futures_vamm_k DECIMAL(24, 2);

ALTER TABLE market_maker_state ADD COLUMN futures_insurance_fund DECIMAL(24, 2);

ALTER TABLE market_maker_state ADD COLUMN futures_open_interest INTEGER;

ALTER TABLE market_maker_state ADD COLUMN last_funding_at TIMESTAMP WITH TIME ZONE;

UPDATE market_maker_state SET futures_vamm_x = 0.0 WHERE futures_vamm_x IS NULL;

UPDATE market_maker_state SET futures_vamm_y = 0.0 WHERE futures_vamm_y IS NULL;

UPDATE market_maker_state SET futures_vamm_k = 0.0 WHERE futures_vamm_k IS NULL;

UPDATE market_maker_state SET futures_insurance_fund = 0.0 WHERE futures_insurance_fund IS NULL;

UPDATE market_maker_state SET futures_open_interest = 0 WHERE futures_open_interest IS NULL;

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_x SET NOT NULL;

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_y SET NOT NULL;

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_k SET NOT NULL;

ALTER TABLE market_maker_state ALTER COLUMN futures_insurance_fund SET NOT NULL;

ALTER TABLE market_maker_state ALTER COLUMN futures_open_interest SET NOT NULL;

UPDATE alembic_version SET version_num='f7e537b63dc2' WHERE alembic_version.version_num = 'cfe3b0a3cd43';

-- Running upgrade f7e537b63dc2 -> f904d1022c36

ALTER TABLE futuresposition ALTER COLUMN entry_price TYPE DECIMAL(24, 2) USING entry_price::numeric(24,2);

ALTER TABLE futuresposition ALTER COLUMN margin TYPE DECIMAL(24, 2) USING margin::numeric(24,2);

ALTER TABLE futuresposition ALTER COLUMN liquidation_price TYPE DECIMAL(24, 2) USING liquidation_price::numeric(24,2);

ALTER TABLE futuresposition ALTER COLUMN realized_pnl TYPE DECIMAL(24, 2) USING realized_pnl::numeric(24,2);

ALTER TABLE futurestransaction ALTER COLUMN price TYPE DECIMAL(24, 2) USING price::numeric(24,2);

ALTER TABLE futurestransaction ALTER COLUMN pnl TYPE DECIMAL(24, 2) USING pnl::numeric(24,2);

ALTER TABLE futurestransaction ALTER COLUMN fee TYPE DECIMAL(24, 2) USING fee::numeric(24,2);

ALTER TABLE futurestransaction ALTER COLUMN margin_change TYPE DECIMAL(24, 2) USING margin_change::numeric(24,2);

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_x TYPE DECIMAL(24, 2) USING futures_vamm_x::numeric(24,2);

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_y TYPE DECIMAL(24, 2) USING futures_vamm_y::numeric(24,2);

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_k TYPE DECIMAL(24, 2) USING futures_vamm_k::numeric(24,2);

ALTER TABLE market_maker_state ALTER COLUMN futures_insurance_fund TYPE DECIMAL(24, 2) USING futures_insurance_fund::numeric(24,2);

UPDATE alembic_version SET version_num='f904d1022c36' WHERE alembic_version.version_num = 'f7e537b63dc2';

-- Running upgrade f904d1022c36 -> fcd56ecd7004

CREATE TABLE futures_price_snapshot (
    id SERIAL NOT NULL, 
    mid_price DECIMAL(24, 2) NOT NULL, 
    spot_price DECIMAL(24, 2) NOT NULL, 
    open_interest INTEGER NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_futures_price_snapshot_created_at ON futures_price_snapshot (created_at);

UPDATE alembic_version SET version_num='fcd56ecd7004' WHERE alembic_version.version_num = 'f904d1022c36';

-- Running upgrade fcd56ecd7004 -> af08666ea622

ALTER TABLE futures_price_snapshot ADD COLUMN funding_rate DECIMAL(12, 8) DEFAULT '0' NOT NULL;

UPDATE alembic_version SET version_num='af08666ea622' WHERE alembic_version.version_num = 'fcd56ecd7004';

-- Running upgrade af08666ea622 -> 2d0574bd3e4b

ALTER TABLE market_maker_state ADD COLUMN futures_anchor_spot DECIMAL(15, 2) DEFAULT '0' NOT NULL;

UPDATE alembic_version SET version_num='2d0574bd3e4b' WHERE alembic_version.version_num = 'af08666ea622';

-- Running upgrade 2d0574bd3e4b -> b71c36fa3a91

ALTER TABLE futures_price_snapshot ALTER COLUMN open_interest TYPE BIGINT;

ALTER TABLE futuresposition ALTER COLUMN quantity TYPE BIGINT;

ALTER TABLE futurestransaction ALTER COLUMN quantity TYPE BIGINT;

ALTER TABLE market_maker_state ALTER COLUMN futures_open_interest TYPE BIGINT;

UPDATE alembic_version SET version_num='b71c36fa3a91' WHERE alembic_version.version_num = '2d0574bd3e4b';

-- Running upgrade b71c36fa3a91 -> b45d6882c43d

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_x TYPE DECIMAL(38, 2);

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_y TYPE DECIMAL(38, 2);

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_k TYPE DECIMAL(38, 2);

UPDATE alembic_version SET version_num='b45d6882c43d' WHERE alembic_version.version_num = 'b71c36fa3a91';

-- Running upgrade b45d6882c43d -> e67f636fec4f

CREATE TABLE red_envelope (
    id SERIAL NOT NULL, 
    sender_id VARCHAR NOT NULL, 
    room_id VARCHAR NOT NULL, 
    message_id VARCHAR NOT NULL, 
    envelope_type VARCHAR NOT NULL, 
    total_amount DECIMAL(24, 2) NOT NULL, 
    remaining_amount DECIMAL(24, 2) NOT NULL, 
    total_count INTEGER NOT NULL, 
    remaining_count INTEGER NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    is_expired BOOLEAN NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_red_envelope_message_id ON red_envelope (message_id);

CREATE INDEX ix_red_envelope_sender_id ON red_envelope (sender_id);

CREATE TABLE red_envelope_claim (
    id SERIAL NOT NULL, 
    envelope_id INTEGER NOT NULL, 
    user_id VARCHAR NOT NULL, 
    amount DECIMAL(24, 2) NOT NULL, 
    claimed_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(envelope_id) REFERENCES red_envelope (id)
);

CREATE INDEX ix_red_envelope_claim_envelope_id ON red_envelope_claim (envelope_id);

CREATE INDEX ix_red_envelope_claim_user_id ON red_envelope_claim (user_id);

UPDATE alembic_version SET version_num='e67f636fec4f' WHERE alembic_version.version_num = 'b45d6882c43d';

-- Running upgrade e67f636fec4f -> bdba68cb6d00

ALTER TABLE futures_price_snapshot ALTER COLUMN funding_rate TYPE DECIMAL(24, 8);

ALTER TABLE market_maker_state ALTER COLUMN mid_price TYPE DECIMAL(24, 2);

ALTER TABLE market_maker_state ALTER COLUMN fair_value TYPE DECIMAL(24, 2);

ALTER TABLE market_maker_state ALTER COLUMN futures_anchor_spot TYPE DECIMAL(24, 2);

ALTER TABLE pal_egg ALTER COLUMN price_paid TYPE DECIMAL(24, 2);

ALTER TABLE stock_trigger ALTER COLUMN trigger_price TYPE DECIMAL(24, 2);

ALTER TABLE turnip_inventory ALTER COLUMN buy_price TYPE DECIMAL(24, 2);

ALTER TABLE turnip_inventory ALTER COLUMN stored_shelf_life_seconds TYPE DECIMAL(24, 2);

ALTER TABLE turnip_order ALTER COLUMN limit_price TYPE DECIMAL(24, 2);

ALTER TABLE turnip_order ALTER COLUMN quote_price TYPE DECIMAL(24, 2);

ALTER TABLE turnip_order ALTER COLUMN execution_price TYPE DECIMAL(24, 2);

ALTER TABLE turnip_order_fill ALTER COLUMN price TYPE DECIMAL(24, 2);

ALTER TABLE turnip_price ALTER COLUMN price TYPE DECIMAL(24, 2);

ALTER TABLE turnip_price ALTER COLUMN open TYPE DECIMAL(24, 2);

ALTER TABLE turnip_price ALTER COLUMN high TYPE DECIMAL(24, 2);

ALTER TABLE turnip_price ALTER COLUMN low TYPE DECIMAL(24, 2);

ALTER TABLE turnip_price ALTER COLUMN base_price TYPE DECIMAL(24, 2);

ALTER TABLE turnip_seed ALTER COLUMN seed_price TYPE DECIMAL(24, 2);

ALTER TABLE turnip_seed ALTER COLUMN fertilize_cost TYPE DECIMAL(24, 2);

ALTER TABLE turnip_transaction ALTER COLUMN unit_price TYPE DECIMAL(24, 2);

ALTER TABLE user_item ALTER COLUMN price_paid TYPE DECIMAL(24, 2);

UPDATE alembic_version SET version_num='bdba68cb6d00' WHERE alembic_version.version_num = 'e67f636fec4f';

-- Running upgrade bdba68cb6d00 -> e38ac3baf542

CREATE TABLE map_floor (
    floor SERIAL NOT NULL, 
    tiles JSON NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (floor)
);

CREATE TABLE raid_profile (
    user_id VARCHAR(64) NOT NULL, 
    name VARCHAR(32) NOT NULL, 
    str_stat INTEGER NOT NULL, 
    dex INTEGER NOT NULL, 
    wil INTEGER NOT NULL, 
    per INTEGER NOT NULL, 
    level INTEGER NOT NULL, 
    experience INTEGER NOT NULL, 
    max_willpower INTEGER NOT NULL, 
    activated_waystones JSON, 
    warehouse JSON, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (user_id)
);

CREATE TABLE raid_session (
    id VARCHAR(32) NOT NULL, 
    user_id VARCHAR(64) NOT NULL, 
    state_json JSON NOT NULL, 
    is_active BOOLEAN NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_raid_session_user_id ON raid_session (user_id);

CREATE TABLE strand_object (
    id BIGSERIAL NOT NULL, 
    type VARCHAR(16) NOT NULL, 
    floor INTEGER NOT NULL, 
    x INTEGER NOT NULL, 
    y INTEGER NOT NULL, 
    owner_user_id VARCHAR(64) NOT NULL, 
    data JSON NOT NULL, 
    likes INTEGER NOT NULL, 
    picked_up_by VARCHAR(64), 
    expires_at TIMESTAMP WITH TIME ZONE, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_strand_object_floor_x_y ON strand_object (floor, x, y);

CREATE INDEX ix_strand_object_owner ON strand_object (owner_user_id);

UPDATE alembic_version SET version_num='e38ac3baf542' WHERE alembic_version.version_num = 'bdba68cb6d00';

-- Running upgrade fa291ef99b44 -> 8188dc7a649b

CREATE INDEX ix_turnip_transaction_created_at ON turnip_transaction (created_at);

INSERT INTO alembic_version (version_num) VALUES ('8188dc7a649b') RETURNING alembic_version.version_num;

-- Running upgrade 8188dc7a649b, e38ac3baf542 -> 5bc30f0941ef

DELETE FROM alembic_version WHERE alembic_version.version_num = '8188dc7a649b';

UPDATE alembic_version SET version_num='5bc30f0941ef' WHERE alembic_version.version_num = 'e38ac3baf542';

-- Running upgrade 5bc30f0941ef -> 7ef304d5e27f

CREATE TABLE raid_map (
    id BIGSERIAL NOT NULL, 
    seed INTEGER NOT NULL, 
    config VARCHAR(32) NOT NULL, 
    version INTEGER NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

DROP TABLE map_floor;

CREATE TABLE raid_map_floor (
    map_id BIGINT NOT NULL, 
    floor INTEGER NOT NULL, 
    tiles JSON NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (map_id, floor)
);

ALTER TABLE raid_session ADD COLUMN map_id BIGINT;

UPDATE alembic_version SET version_num='7ef304d5e27f' WHERE alembic_version.version_num = '5bc30f0941ef';

-- Running upgrade 7ef304d5e27f -> b089f1972d43

ALTER TABLE raid_map ADD COLUMN config_new JSONB;

SELECT id, config FROM raid_map;

UPDATE raid_map
        SET config_new = CASE config
            WHEN 'default' THEN '{"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL,"extraction_ring_radius"NULL,"spawn_extraction"NULL,"floors":[{"floor"NULL,"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL},{"floor":-1,"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL},{"floor":-2,"radius"NULL,"radius_noise"NULL.5,"extraction_count"NULL}]}'::jsonb
WHEN 'small' THEN '{"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL,"extraction_ring_radius"NULL,"spawn_extraction"NULL,"floors":[{"floor"NULL,"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL},{"floor":-1,"radius"NULL,"radius_noise"NULL.5,"extraction_count"NULL}]}'::jsonb
WHEN 'large' THEN '{"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL,"extraction_ring_radius"NULL,"spawn_extraction"NULL,"floors":[{"floor"NULL,"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL},{"floor":-1,"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL},{"floor":-2,"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL},{"floor":-3,"radius"NULL,"radius_noise"NULL.5,"extraction_count"NULL}]}'::jsonb
            ELSE '{"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL,"extraction_ring_radius"NULL,"spawn_extraction"NULL,"floors":[{"floor"NULL,"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL},{"floor":-1,"radius"NULL,"radius_noise"NULL.0,"extraction_count"NULL},{"floor":-2,"radius"NULL,"radius_noise"NULL.5,"extraction_count"NULL}]}'::jsonb
        END;

ALTER TABLE raid_map DROP COLUMN config;

ALTER TABLE raid_map ALTER COLUMN config_new SET NOT NULL;

ALTER TABLE raid_map RENAME config_new TO config;

UPDATE alembic_version SET version_num='b089f1972d43' WHERE alembic_version.version_num = '7ef304d5e27f';

-- Running upgrade 7ef304d5e27f -> b81a8bafd021

ALTER TABLE trpg_checkpoints ADD COLUMN room_id VARCHAR;

UPDATE trpg_checkpoints cp
        SET room_id = g.room_id
        FROM trpg_games g
        WHERE cp.game_id = g.id;

DELETE FROM trpg_checkpoints WHERE room_id IS NULL;

ALTER TABLE trpg_checkpoints ALTER COLUMN room_id SET NOT NULL;

ALTER TABLE trpg_checkpoints ALTER COLUMN game_id DROP NOT NULL;

DELETE FROM trpg_checkpoints
        WHERE id NOT IN (
            SELECT DISTINCT ON (room_id, name) id
            FROM trpg_checkpoints
            ORDER BY room_id, name, saved_at DESC
        );

DROP INDEX ix_trpg_checkpoints_game_id;

ALTER TABLE trpg_checkpoints DROP CONSTRAINT trpg_checkpoints_game_id_name_key;

CREATE INDEX ix_trpg_checkpoints_room_id ON trpg_checkpoints (room_id);

ALTER TABLE trpg_checkpoints ADD CONSTRAINT trpg_checkpoints_room_id_name_key UNIQUE (room_id, name);

INSERT INTO alembic_version (version_num) VALUES ('b81a8bafd021') RETURNING alembic_version.version_num;

-- Running upgrade b81a8bafd021, b089f1972d43 -> 64d429319d2d

DELETE FROM alembic_version WHERE alembic_version.version_num = 'b81a8bafd021';

UPDATE alembic_version SET version_num='64d429319d2d' WHERE alembic_version.version_num = 'b089f1972d43';

-- Running upgrade 64d429319d2d -> c1ce80aa06e0

ALTER TABLE trpg_games ADD COLUMN title VARCHAR DEFAULT '' NOT NULL;

UPDATE alembic_version SET version_num='c1ce80aa06e0' WHERE alembic_version.version_num = '64d429319d2d';

-- Running upgrade c1ce80aa06e0 -> 041f0c4f9cb6

ALTER TABLE raid_profile ADD COLUMN loadout JSON;

UPDATE alembic_version SET version_num='041f0c4f9cb6' WHERE alembic_version.version_num = 'c1ce80aa06e0';

-- Running upgrade 041f0c4f9cb6 -> 764bbe373dde

ALTER TABLE raid_profile ADD COLUMN pending_attribute_points INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN skill_search INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN skill_combat INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN skill_stealth INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN skill_resist INTEGER DEFAULT '0' NOT NULL;

UPDATE raid_profile SET pending_attribute_points = level - 1, max_willpower = 100 + (level - 1) * 5 WHERE level > 1;

UPDATE raid_profile SET max_willpower = 100 WHERE level = 1;

ALTER TABLE raid_profile ALTER COLUMN pending_attribute_points DROP DEFAULT;

ALTER TABLE raid_profile ALTER COLUMN skill_search DROP DEFAULT;

ALTER TABLE raid_profile ALTER COLUMN skill_combat DROP DEFAULT;

ALTER TABLE raid_profile ALTER COLUMN skill_stealth DROP DEFAULT;

ALTER TABLE raid_profile ALTER COLUMN skill_resist DROP DEFAULT;

UPDATE alembic_version SET version_num='764bbe373dde' WHERE alembic_version.version_num = '041f0c4f9cb6';

-- Running upgrade 764bbe373dde -> e721d24d6218

ALTER TABLE raid_profile ALTER COLUMN activated_waystones TYPE JSONB;

ALTER TABLE raid_profile ALTER COLUMN warehouse TYPE JSONB;

ALTER TABLE raid_profile ALTER COLUMN loadout TYPE JSONB;

ALTER TABLE raid_session ALTER COLUMN state_json TYPE JSONB;

ALTER TABLE strand_object ALTER COLUMN data TYPE JSONB;

UPDATE alembic_version SET version_num='e721d24d6218' WHERE alembic_version.version_num = '764bbe373dde';

-- Running upgrade e721d24d6218 -> d0f11bfe030b

CREATE TABLE raid_action_log (
    id BIGSERIAL NOT NULL, 
    session_id VARCHAR(32) NOT NULL, 
    seq INTEGER NOT NULL, 
    action JSONB NOT NULL, 
    effects JSONB NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_raid_action_log_session_id ON raid_action_log (session_id);

UPDATE alembic_version SET version_num='d0f11bfe030b' WHERE alembic_version.version_num = 'e721d24d6218';

-- Running upgrade d0f11bfe030b -> 66b7b040df4a

ALTER TABLE turnip_inventory ALTER COLUMN quantity TYPE DECIMAL(38, 0);

ALTER TABLE turnip_order ALTER COLUMN quantity TYPE DECIMAL(38, 0);

ALTER TABLE turnip_order ALTER COLUMN filled_quantity TYPE DECIMAL(38, 0);

ALTER TABLE turnip_price ALTER COLUMN volume TYPE DECIMAL(38, 0);

ALTER TABLE turnip_price ALTER COLUMN trade_count TYPE DECIMAL(38, 0);

ALTER TABLE turnip_transaction ALTER COLUMN quantity TYPE DECIMAL(38, 0);

ALTER TABLE turnip_transaction ALTER COLUMN balance_after TYPE DECIMAL(38, 0);

ALTER TABLE user_achievement_progress ALTER COLUMN value TYPE DECIMAL(38, 0);

UPDATE alembic_version SET version_num='66b7b040df4a' WHERE alembic_version.version_num = 'd0f11bfe030b';

-- Running upgrade 66b7b040df4a -> 1ffb492a3caa

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_x TYPE DECIMAL(48, 2);

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_y TYPE DECIMAL(48, 2);

ALTER TABLE market_maker_state ALTER COLUMN futures_vamm_k TYPE DECIMAL(78, 2);

UPDATE alembic_version SET version_num='1ffb492a3caa' WHERE alembic_version.version_num = '66b7b040df4a';

-- Running upgrade 1ffb492a3caa -> bd7ae672e932

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
    starting_balance DECIMAL(24, 2) NOT NULL, 
    pals_kept INTEGER NOT NULL, 
    pals_settled INTEGER NOT NULL, 
    lands_settled INTEGER NOT NULL, 
    details JSONB, 
    settled_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_season_settlement_season_rank ON season_settlement (season, rank);

CREATE INDEX ix_season_settlement_user_id ON season_settlement (user_id);

CREATE UNIQUE INDEX ix_season_settlement_user_season ON season_settlement (user_id, season);

UPDATE alembic_version SET version_num='bd7ae672e932' WHERE alembic_version.version_num = '1ffb492a3caa';

-- Running upgrade bd7ae672e932 -> b0ebba997bf0

ALTER TABLE raid_profile ADD COLUMN total_raids INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN raids_survived INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN total_loot_value INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN max_rooms_explored INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN total_kills INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN total_bosses_killed INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN survival_streak INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN best_survival_streak INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN gifts_placed INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_profile ADD COLUMN gifts_received INTEGER DEFAULT '0' NOT NULL;

UPDATE alembic_version SET version_num='b0ebba997bf0' WHERE alembic_version.version_num = 'bd7ae672e932';

-- Running upgrade b0ebba997bf0 -> e3e7229dec13

ALTER TABLE strand_object ADD COLUMN map_id BIGINT;

UPDATE strand_object SET map_id = 0 WHERE map_id IS NULL;

ALTER TABLE strand_object ALTER COLUMN map_id SET NOT NULL;

DROP INDEX ix_strand_object_floor_x_y;

CREATE INDEX ix_strand_object_map_floor_x_y ON strand_object (map_id, floor, x, y);

UPDATE alembic_version SET version_num='e3e7229dec13' WHERE alembic_version.version_num = 'b0ebba997bf0';

-- Running upgrade e3e7229dec13 -> a1b2c3d4e5f6

UPDATE raid_profile
        SET warehouse = jsonb_set(warehouse, '{items}', 
(
    SELECT coalesce(jsonb_agg(
        CASE
            WHEN item->>'id' NOT LIKE '%%@%%'
            THEN jsonb_set(
                item, '{id}',
                to_jsonb((item->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
            )
            ELSE item
        END
        ORDER BY ordinality
    ), '[]'::jsonb)
    FROM jsonb_array_elements(warehouse->'items')
        WITH ORDINALITY AS t(item, ordinality)
)
)
        WHERE warehouse IS NOT NULL
          AND warehouse->'items' IS NOT NULL
          AND jsonb_array_length(warehouse->'items') > 0
          AND EXISTS (
              SELECT 1 FROM jsonb_array_elements(warehouse->'items') AS item
              WHERE item->>'id' NOT LIKE '%%@%%'
          );

UPDATE raid_profile
        SET loadout = jsonb_set(
            loadout,
            '{item_ids}',
            (
                SELECT coalesce(jsonb_agg(
                    coalesce(
                        (
                            SELECT w_item->>'id'
                            FROM jsonb_array_elements(warehouse->'items') AS w_item
                            WHERE split_part(w_item->>'id', '@', 1) = lid.val
                            LIMIT 1
                        ),
                        lid.val
                    )
                    ORDER BY lid.ordinality
                ), '[]'::jsonb)
                FROM jsonb_array_elements_text(loadout->'item_ids')
                    WITH ORDINALITY AS lid(val, ordinality)
            )
        )
        WHERE loadout IS NOT NULL
          AND loadout->'item_ids' IS NOT NULL
          AND jsonb_array_length(loadout->'item_ids') > 0
          AND warehouse IS NOT NULL;

UPDATE raid_session
        SET state_json = jsonb_set(state_json, '{inventory}', 
(
    SELECT coalesce(jsonb_agg(
        CASE
            WHEN item->>'id' NOT LIKE '%%@%%'
            THEN jsonb_set(
                item, '{id}',
                to_jsonb((item->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
            )
            ELSE item
        END
        ORDER BY ordinality
    ), '[]'::jsonb)
    FROM jsonb_array_elements(state_json->'inventory')
        WITH ORDINALITY AS t(item, ordinality)
)
)
        WHERE is_active = true
          AND state_json->'inventory' IS NOT NULL
          AND jsonb_array_length(state_json->'inventory') > 0
          AND EXISTS (
              SELECT 1 FROM jsonb_array_elements(state_json->'inventory') AS item
              WHERE item->>'id' NOT LIKE '%%@%%'
          );

UPDATE raid_session
            SET state_json = jsonb_set(
                state_json,
                '{equipment,weapon}',
            
    CASE
        WHEN state_json->'equipment'->'weapon' IS NOT NULL
         AND state_json->'equipment'->'weapon'->>'id' IS NOT NULL
         AND state_json->'equipment'->'weapon'->>'id' NOT LIKE '%%@%%'
        THEN jsonb_set(
            state_json->'equipment'->'weapon', '{id}',
            to_jsonb((state_json->'equipment'->'weapon'->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
        )
        ELSE state_json->'equipment'->'weapon'
    END

            )
            WHERE is_active = true
              AND state_json->'equipment' IS NOT NULL
              AND state_json->'equipment'->'weapon' IS NOT NULL
              AND state_json->'equipment'->'weapon'->>'id' IS NOT NULL
              AND state_json->'equipment'->'weapon'->>'id' NOT LIKE '%%@%%';

UPDATE raid_session
            SET state_json = jsonb_set(
                state_json,
                '{equipment,armor}',
            
    CASE
        WHEN state_json->'equipment'->'armor' IS NOT NULL
         AND state_json->'equipment'->'armor'->>'id' IS NOT NULL
         AND state_json->'equipment'->'armor'->>'id' NOT LIKE '%%@%%'
        THEN jsonb_set(
            state_json->'equipment'->'armor', '{id}',
            to_jsonb((state_json->'equipment'->'armor'->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
        )
        ELSE state_json->'equipment'->'armor'
    END

            )
            WHERE is_active = true
              AND state_json->'equipment' IS NOT NULL
              AND state_json->'equipment'->'armor' IS NOT NULL
              AND state_json->'equipment'->'armor'->>'id' IS NOT NULL
              AND state_json->'equipment'->'armor'->>'id' NOT LIKE '%%@%%';

UPDATE raid_session
            SET state_json = jsonb_set(
                state_json,
                '{equipment,accessory}',
            
    CASE
        WHEN state_json->'equipment'->'accessory' IS NOT NULL
         AND state_json->'equipment'->'accessory'->>'id' IS NOT NULL
         AND state_json->'equipment'->'accessory'->>'id' NOT LIKE '%%@%%'
        THEN jsonb_set(
            state_json->'equipment'->'accessory', '{id}',
            to_jsonb((state_json->'equipment'->'accessory'->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
        )
        ELSE state_json->'equipment'->'accessory'
    END

            )
            WHERE is_active = true
              AND state_json->'equipment' IS NOT NULL
              AND state_json->'equipment'->'accessory' IS NOT NULL
              AND state_json->'equipment'->'accessory'->>'id' IS NOT NULL
              AND state_json->'equipment'->'accessory'->>'id' NOT LIKE '%%@%%';

UPDATE raid_session
            SET state_json = jsonb_set(
                state_json,
                '{equipment,backpack}',
            
    CASE
        WHEN state_json->'equipment'->'backpack' IS NOT NULL
         AND state_json->'equipment'->'backpack'->>'id' IS NOT NULL
         AND state_json->'equipment'->'backpack'->>'id' NOT LIKE '%%@%%'
        THEN jsonb_set(
            state_json->'equipment'->'backpack', '{id}',
            to_jsonb((state_json->'equipment'->'backpack'->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
        )
        ELSE state_json->'equipment'->'backpack'
    END

            )
            WHERE is_active = true
              AND state_json->'equipment' IS NOT NULL
              AND state_json->'equipment'->'backpack' IS NOT NULL
              AND state_json->'equipment'->'backpack'->>'id' IS NOT NULL
              AND state_json->'equipment'->'backpack'->>'id' NOT LIKE '%%@%%';

UPDATE alembic_version SET version_num='a1b2c3d4e5f6' WHERE alembic_version.version_num = 'e3e7229dec13';

-- Running upgrade fcd56ecd7004 -> c5d7be7b1f3c

ALTER TABLE wallet ADD COLUMN escrow_balance DECIMAL(24, 2) DEFAULT '0' NOT NULL;

ALTER TABLE wallet_transaction ADD COLUMN counterparty_id VARCHAR(100);

ALTER TABLE wallet_transaction ADD COLUMN tx_group_id VARCHAR(50);

ALTER TABLE wallet_transaction ADD COLUMN escrow_after DECIMAL(24, 2);

CREATE INDEX ix_wallet_transaction_counterparty_id ON wallet_transaction (counterparty_id);

CREATE INDEX ix_wallet_transaction_tx_group_id ON wallet_transaction (tx_group_id);

INSERT INTO alembic_version (version_num) VALUES ('c5d7be7b1f3c') RETURNING alembic_version.version_num;

-- Running upgrade a1b2c3d4e5f6, c5d7be7b1f3c -> e65630fb8d83

DELETE FROM alembic_version WHERE alembic_version.version_num = 'a1b2c3d4e5f6';

UPDATE alembic_version SET version_num='e65630fb8d83' WHERE alembic_version.version_num = 'c5d7be7b1f3c';

-- Running upgrade e65630fb8d83 -> 336104b44fe2

UPDATE wallet_transaction SET counterparty_id = NULL WHERE counterparty_id = NULL;

SELECT balance, escrow_balance FROM wallet WHERE user_id = NULL;

UPDATE alembic_version SET version_num='336104b44fe2' WHERE alembic_version.version_num = 'e65630fb8d83';

-- Running upgrade 336104b44fe2 -> 8ca96bbab5ef

INSERT INTO wallet_transaction
                (user_id, amount, balance_after, tx_type, description,
                 counterparty_id, tx_group_id, reference_id, memo, created_at)
            SELECT
                NULL,
                -amount,
                0,
                tx_type,
                description,
                user_id,
                'backfill-' || id::text,
                reference_id,
                memo,
                created_at
            FROM wallet_transaction
            WHERE tx_group_id IS NULL
              AND user_id != NULL
              AND tx_type NOT IN (NULL, NULL, NULL, NULL);

UPDATE wallet
            SET balance = (
                SELECT COALESCE(SUM(amount), 0)
                FROM wallet_transaction
                WHERE user_id = NULL
            )
            WHERE user_id = NULL;

UPDATE wallet_transaction
            SET tx_group_id = 'backfill-' || id::text
            WHERE tx_group_id IS NULL
              AND user_id != NULL
              AND tx_type NOT IN (NULL, NULL, NULL, NULL);

UPDATE alembic_version SET version_num='8ca96bbab5ef' WHERE alembic_version.version_num = '336104b44fe2';

-- Running upgrade 8ca96bbab5ef -> beb6047dc81d

SELECT
                (SELECT COALESCE(SUM(balance), 0) FROM wallet
                 WHERE user_id <> NULL)
              + (SELECT COALESCE(balance, 0) FROM wallet
                 WHERE user_id = NULL);

UPDATE alembic_version SET version_num='beb6047dc81d' WHERE alembic_version.version_num = '8ca96bbab5ef';

-- Running upgrade a1b2c3d4e5f6 -> 51162bbe7918

ALTER TABLE market_maker_state ADD COLUMN futures_total_short BIGINT DEFAULT '0' NOT NULL;

UPDATE market_maker_state SET
            futures_total_short = COALESCE(
                (SELECT SUM(quantity) FROM futuresposition WHERE status = 'OPEN' AND side = 'SHORT'), 0
            ),
            futures_open_interest = COALESCE(
                (SELECT SUM(quantity) FROM futuresposition WHERE status = 'OPEN' AND side = 'LONG'), 0
            );

INSERT INTO alembic_version (version_num) VALUES ('51162bbe7918') RETURNING alembic_version.version_num;

-- Running upgrade 51162bbe7918, beb6047dc81d -> 1fde55128e0a

DELETE FROM alembic_version WHERE alembic_version.version_num = '51162bbe7918';

UPDATE alembic_version SET version_num='1fde55128e0a' WHERE alembic_version.version_num = 'beb6047dc81d';

-- Running upgrade 1fde55128e0a -> d1e2f3a4b5c6

UPDATE wallet w
            SET
                balance = balance - o.total_escrow,
                escrow_balance = escrow_balance + o.total_escrow
            FROM (
                SELECT user_id, SUM(escrow_amount) AS total_escrow
                FROM turnip_order
                WHERE side = 'buy'
                  AND status IN ('pending', 'partial')
                  AND escrow_amount > 0
                GROUP BY user_id
            ) o
            WHERE w.user_id = o.user_id;

UPDATE alembic_version SET version_num='d1e2f3a4b5c6' WHERE alembic_version.version_num = '1fde55128e0a';

-- Running upgrade 1fde55128e0a -> 53e257f3f7b6

ALTER TABLE market_maker_state ADD COLUMN is_paused BOOLEAN DEFAULT false NOT NULL;

INSERT INTO alembic_version (version_num) VALUES ('53e257f3f7b6') RETURNING alembic_version.version_num;

-- Running upgrade 53e257f3f7b6, d1e2f3a4b5c6 -> d24dff813320

DELETE FROM alembic_version WHERE alembic_version.version_num = '53e257f3f7b6';

UPDATE alembic_version SET version_num='d24dff813320' WHERE alembic_version.version_num = 'd1e2f3a4b5c6';

-- Running upgrade d24dff813320 -> 7f534fefbc6f

ALTER TABLE futurestransaction ADD COLUMN spot_price DECIMAL(24, 2);

ALTER TABLE turnip_transaction ADD COLUMN mid_price DECIMAL(24, 2);

UPDATE alembic_version SET version_num='7f534fefbc6f' WHERE alembic_version.version_num = 'd24dff813320';

-- Running upgrade 7f534fefbc6f -> 8add917e8eaa

UPDATE games AS g
        SET players = normalized.players
        FROM (
            SELECT
                game_rows.id,
                COALESCE(
                    jsonb_agg(game_rows.normalized_elem)
                        FILTER (WHERE game_rows.normalized_elem IS NOT NULL),
                    '[]'::jsonb
                ) AS players
            FROM (
                SELECT
                    src.id,
                    CASE
                        WHEN jsonb_typeof(elem) = 'string' THEN elem
                        WHEN jsonb_typeof(elem) = 'object'
                             AND COALESCE(elem->>'user_id', elem->>'userId', '') <> ''
                        THEN to_jsonb(COALESCE(elem->>'user_id', elem->>'userId'))
                        ELSE NULL
                    END AS normalized_elem
                FROM games AS src
                CROSS JOIN LATERAL jsonb_array_elements(src.players) AS elem
                WHERE jsonb_typeof(src.players) = 'array'
            ) AS game_rows
            GROUP BY game_rows.id
        ) AS normalized
        WHERE g.id = normalized.id
          AND g.players IS DISTINCT FROM normalized.players;

UPDATE games
        SET players = '[]'::jsonb
        WHERE COALESCE(jsonb_typeof(players), 'null') <> 'array';

UPDATE alembic_version SET version_num='8add917e8eaa' WHERE alembic_version.version_num = '7f534fefbc6f';

-- Running upgrade 8add917e8eaa -> bd0ddb8085de

DELETE FROM wallet_transaction
            WHERE user_id = NULL
              AND tx_type IN (NULL, NULL, NULL);

UPDATE wallet_transaction
            SET tx_group_id = NULL
            WHERE tx_group_id LIKE 'backfill-%%'
              AND user_id != NULL
              AND tx_type IN (NULL, NULL, NULL);

UPDATE wallet
            SET balance = (
                SELECT COALESCE(SUM(amount), 0)
                FROM wallet_transaction
                WHERE user_id = NULL
            )
            WHERE user_id = NULL;

UPDATE alembic_version SET version_num='bd0ddb8085de' WHERE alembic_version.version_num = '8add917e8eaa';

-- Running upgrade 7f534fefbc6f -> 73a054867fd1

ALTER TABLE market_maker_state ADD COLUMN sub_tick INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE market_maker_state ADD COLUMN cached_effective_depth DECIMAL(24, 2);

ALTER TABLE market_maker_state ADD COLUMN cached_market_center DECIMAL(24, 2);

INSERT INTO alembic_version (version_num) VALUES ('73a054867fd1') RETURNING alembic_version.version_num;

-- Running upgrade 73a054867fd1, bd0ddb8085de -> eeed56e9fa28

DELETE FROM alembic_version WHERE alembic_version.version_num = '73a054867fd1';

UPDATE alembic_version SET version_num='eeed56e9fa28' WHERE alembic_version.version_num = 'bd0ddb8085de';

-- Running upgrade eeed56e9fa28 -> 309b90aaedf7

SELECT
                (SELECT COALESCE(SUM(balance + escrow_balance), 0)
                 FROM wallet WHERE user_id <> NULL)
              + (SELECT COALESCE(balance, 0)
                 FROM wallet WHERE user_id = NULL);

UPDATE alembic_version SET version_num='309b90aaedf7' WHERE alembic_version.version_num = 'eeed56e9fa28';

-- Running upgrade 309b90aaedf7 -> b07522d2a4a1

CREATE OR REPLACE FUNCTION notify_turnip_tick()
        RETURNS trigger AS $$
        BEGIN
            PERFORM pg_notify('turnip_tick', '');
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;;

CREATE TRIGGER market_maker_state_tick_trigger
        AFTER UPDATE ON market_maker_state
        FOR EACH ROW
        WHEN (NEW.id = 1)
        EXECUTE FUNCTION notify_turnip_tick();;

CREATE OR REPLACE FUNCTION notify_futures_tick()
        RETURNS trigger AS $$
        BEGIN
            PERFORM pg_notify('futures_tick', json_build_object(
                'mid_price', NEW.mid_price::text,
                'spot_price', NEW.spot_price::text,
                'funding_rate', NEW.funding_rate::text,
                'open_interest', NEW.open_interest,
                'created_at', NEW.created_at::text
            )::text);

            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;;

CREATE TRIGGER futures_snapshot_tick_trigger
        AFTER INSERT ON futures_price_snapshot
        FOR EACH ROW
        EXECUTE FUNCTION notify_futures_tick();;

UPDATE alembic_version SET version_num='b07522d2a4a1' WHERE alembic_version.version_num = '309b90aaedf7';

-- Running upgrade b07522d2a4a1 -> a8be76065240

ALTER TABLE futuresposition ADD COLUMN take_profit_price DECIMAL(24, 2);

ALTER TABLE futuresposition ADD COLUMN stop_loss_price DECIMAL(24, 2);

UPDATE alembic_version SET version_num='a8be76065240' WHERE alembic_version.version_num = 'b07522d2a4a1';

-- Running upgrade a8be76065240 -> f064dd05400b

ALTER TABLE futures_price_snapshot ADD COLUMN futures_price DECIMAL(24, 2);

UPDATE futures_price_snapshot SET futures_price = mid_price WHERE futures_price IS NULL;

ALTER TABLE futures_price_snapshot ALTER COLUMN futures_price SET NOT NULL;

UPDATE alembic_version SET version_num='f064dd05400b' WHERE alembic_version.version_num = 'a8be76065240';

-- Running upgrade f064dd05400b -> c3a1f8e92b10

CREATE OR REPLACE FUNCTION notify_futures_tick()
        RETURNS trigger AS $$
        BEGIN
            PERFORM pg_notify('futures_tick', json_build_object(
                'mid_price', NEW.mid_price::text,
                'spot_price', NEW.spot_price::text,
                'futures_price', NEW.futures_price::text,
                'funding_rate', NEW.funding_rate::text,
                'open_interest', NEW.open_interest,
                'created_at', NEW.created_at::text
            )::text);

            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;;

UPDATE alembic_version SET version_num='c3a1f8e92b10' WHERE alembic_version.version_num = 'f064dd05400b';

-- Running upgrade c3a1f8e92b10 -> 9f8a6c4e2b10

ALTER TABLE futures_price_snapshot ADD COLUMN total_long BIGINT;

ALTER TABLE futures_price_snapshot ADD COLUMN total_short BIGINT;

UPDATE futures_price_snapshot SET total_long = 0, total_short = 0 WHERE total_long IS NULL OR total_short IS NULL;

ALTER TABLE futures_price_snapshot ALTER COLUMN total_long SET NOT NULL;

ALTER TABLE futures_price_snapshot ALTER COLUMN total_short SET NOT NULL;

UPDATE alembic_version SET version_num='9f8a6c4e2b10' WHERE alembic_version.version_num = 'c3a1f8e92b10';

-- Running upgrade c3a1f8e92b10 -> 947f84fffc98

ALTER TABLE market_maker_state ADD COLUMN futures_paused BOOLEAN DEFAULT false NOT NULL;

INSERT INTO alembic_version (version_num) VALUES ('947f84fffc98') RETURNING alembic_version.version_num;

-- Running upgrade 947f84fffc98 -> 2a7f1f2fd8af

CREATE TABLE game_currency_account (
    id SERIAL NOT NULL, 
    user_id VARCHAR(64) NOT NULL, 
    currency VARCHAR(32) NOT NULL, 
    balance DECIMAL(24, 6) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    CONSTRAINT uq_game_currency_account_user_currency UNIQUE (user_id, currency)
);

CREATE INDEX ix_game_currency_account_currency ON game_currency_account (currency);

CREATE INDEX ix_game_currency_account_user_id ON game_currency_account (user_id);

CREATE TABLE game_currency_transaction (
    id SERIAL NOT NULL, 
    user_id VARCHAR(64) NOT NULL, 
    currency VARCHAR(32) NOT NULL, 
    amount DECIMAL(24, 6) NOT NULL, 
    balance_after DECIMAL(24, 6) NOT NULL, 
    tx_type VARCHAR(32) NOT NULL, 
    description VARCHAR(255) NOT NULL, 
    reference_id VARCHAR(128), 
    order_id VARCHAR(64), 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_game_currency_transaction_created_at ON game_currency_transaction (created_at);

CREATE INDEX ix_game_currency_transaction_currency ON game_currency_transaction (currency);

CREATE INDEX ix_game_currency_transaction_order_id ON game_currency_transaction (order_id);

CREATE INDEX ix_game_currency_transaction_reference_id ON game_currency_transaction (reference_id);

CREATE INDEX ix_game_currency_transaction_user_id ON game_currency_transaction (user_id);

CREATE TABLE currency_exchange_order (
    id VARCHAR(32) NOT NULL, 
    user_id VARCHAR(64) NOT NULL, 
    currency VARCHAR(32) NOT NULL, 
    direction VARCHAR(24) NOT NULL, 
    input_amount DECIMAL(24, 8) NOT NULL, 
    output_amount DECIMAL(24, 8) NOT NULL, 
    rate DECIMAL(24, 8) NOT NULL, 
    fee_amount DECIMAL(24, 8) NOT NULL, 
    wallet_delta DECIMAL(24, 2) NOT NULL, 
    game_delta DECIMAL(24, 6) NOT NULL, 
    provider VARCHAR(32) NOT NULL, 
    provider_revision INTEGER NOT NULL, 
    wallet_inflation DECIMAL(24, 8) NOT NULL, 
    game_inflation DECIMAL(24, 8) NOT NULL, 
    status VARCHAR(16) NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    idempotency_key VARCHAR(64), 
    executed_at TIMESTAMP WITH TIME ZONE, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_currency_exchange_order_created_at ON currency_exchange_order (created_at);

CREATE INDEX ix_currency_exchange_order_currency ON currency_exchange_order (currency);

CREATE INDEX ix_currency_exchange_order_expires_at ON currency_exchange_order (expires_at);

CREATE INDEX ix_currency_exchange_order_idempotency_key ON currency_exchange_order (idempotency_key);

CREATE INDEX ix_currency_exchange_order_status ON currency_exchange_order (status);

CREATE INDEX ix_currency_exchange_order_user_id ON currency_exchange_order (user_id);

CREATE TABLE currency_rate_state (
    currency VARCHAR(32) NOT NULL, 
    current_rate DECIMAL(24, 8) NOT NULL, 
    revision INTEGER NOT NULL, 
    last_wallet_supply DECIMAL(24, 2) NOT NULL, 
    last_game_supply DECIMAL(24, 6) NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (currency)
);

UPDATE alembic_version SET version_num='2a7f1f2fd8af' WHERE alembic_version.version_num = '947f84fffc98';

-- Running upgrade 2a7f1f2fd8af -> c3d1e6a9f0b2

CREATE TABLE economy_control_state (
    id SERIAL NOT NULL, 
    money_growth_soft_pct DECIMAL(8, 6) NOT NULL, 
    money_growth_hard_pct DECIMAL(8, 6) NOT NULL, 
    turnip_growth_soft_pct DECIMAL(8, 6) NOT NULL, 
    turnip_growth_hard_pct DECIMAL(8, 6) NOT NULL, 
    seed_weight DECIMAL(8, 6) NOT NULL, 
    risk_mode VARCHAR(16) NOT NULL, 
    inventory_conservation_enabled BOOLEAN NOT NULL, 
    amm_trade_fee_bps INTEGER NOT NULL, 
    stored_decay_bps_per_day INTEGER NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE TABLE economy_snapshot (
    id SERIAL NOT NULL, 
    window_hours INTEGER NOT NULL, 
    money_source DECIMAL(24, 2) NOT NULL, 
    money_sink DECIMAL(24, 2) NOT NULL, 
    turnip_source BIGINT NOT NULL, 
    turnip_sink BIGINT NOT NULL, 
    liquidity DECIMAL(24, 2) NOT NULL, 
    effective_supply DECIMAL(24, 2) NOT NULL, 
    coverage_ratio DECIMAL(24, 8) NOT NULL, 
    bank_liability DECIMAL(24, 2) NOT NULL, 
    money_growth DECIMAL(24, 8) NOT NULL, 
    turnip_growth DECIMAL(24, 8) NOT NULL, 
    risk_mode VARCHAR(16) NOT NULL, 
    captured_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_economy_snapshot_captured_at ON economy_snapshot (captured_at);

CREATE UNIQUE INDEX uq_economy_snapshot_window_time ON economy_snapshot (window_hours, captured_at);

UPDATE alembic_version SET version_num='c3d1e6a9f0b2' WHERE alembic_version.version_num = '2a7f1f2fd8af';

-- Running upgrade 2a7f1f2fd8af, 9f8a6c4e2b10 -> ac6aeb3cd52b

UPDATE alembic_version SET version_num='ac6aeb3cd52b' WHERE alembic_version.version_num = '9f8a6c4e2b10';

-- Running upgrade ac6aeb3cd52b -> 2cf21e19d97f

ALTER TABLE private_rooms ADD COLUMN bot_user_id VARCHAR;

UPDATE private_rooms SET bot_user_id = '8e4f7199-4cdc-47ec-a579-6801073fad79' WHERE bot_user_id IS NULL;

UPDATE alembic_version SET version_num='2cf21e19d97f' WHERE alembic_version.version_num = 'ac6aeb3cd52b';

-- Running upgrade 2cf21e19d97f -> 7081419e4c91

DROP INDEX ix_private_rooms_user_id;

CREATE INDEX ix_private_rooms_user_id ON private_rooms (user_id);

CREATE UNIQUE INDEX uq_private_rooms_user_bot ON private_rooms (user_id, bot_user_id);

UPDATE alembic_version SET version_num='7081419e4c91' WHERE alembic_version.version_num = '2cf21e19d97f';

-- Running upgrade 7081419e4c91, c3d1e6a9f0b2 -> 23c955e18d7d

DELETE FROM alembic_version WHERE alembic_version.version_num = '7081419e4c91';

UPDATE alembic_version SET version_num='23c955e18d7d' WHERE alembic_version.version_num = 'c3d1e6a9f0b2';

-- Running upgrade 23c955e18d7d -> d5572af5b700

CREATE TABLE user_credential (
    id SERIAL NOT NULL, 
    username VARCHAR(64) NOT NULL, 
    password_hash VARCHAR(256) NOT NULL, 
    user_id VARCHAR NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    last_login TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX ix_user_credential_user_id ON user_credential (user_id);

CREATE UNIQUE INDEX ix_user_credential_username ON user_credential (username);

UPDATE alembic_version SET version_num='d5572af5b700' WHERE alembic_version.version_num = '23c955e18d7d';

-- Running upgrade d5572af5b700 -> 39db093faf94

ALTER TABLE turnip_price ALTER COLUMN cycle_context TYPE JSONB USING cycle_context::jsonb;

ALTER TABLE market_maker_state ALTER COLUMN amm_book TYPE JSONB USING amm_book::jsonb;

ALTER TABLE market_maker_state ALTER COLUMN recent_trades TYPE JSONB USING recent_trades::jsonb;

ALTER TABLE raid_map_floor ALTER COLUMN tiles TYPE JSONB USING tiles::jsonb;

UPDATE alembic_version SET version_num='39db093faf94' WHERE alembic_version.version_num = 'd5572af5b700';

-- Running upgrade 39db093faf94 -> 66cb674fe3a5

ALTER TABLE room_members
        ALTER COLUMN raw_data TYPE jsonb
        USING raw_data::jsonb;

ALTER TABLE battle_records
        ALTER COLUMN pal_ids TYPE jsonb
        USING pal_ids::jsonb;

ALTER TABLE battle_records
        ALTER COLUMN pal_levels TYPE jsonb
        USING pal_levels::jsonb;

ALTER TABLE battle_records
        ALTER COLUMN battle_log DROP DEFAULT;

ALTER TABLE battle_records
        ALTER COLUMN battle_log TYPE jsonb
        USING battle_log::jsonb;

ALTER TABLE battle_records
        ALTER COLUMN battle_log SET DEFAULT '{}'::jsonb;

UPDATE alembic_version SET version_num='66cb674fe3a5' WHERE alembic_version.version_num = '39db093faf94';

-- Running upgrade 66cb674fe3a5 -> 3b7d8740835b

ALTER TABLE market_maker_state ADD COLUMN fv_noise_offset FLOAT DEFAULT '0' NOT NULL;

UPDATE alembic_version SET version_num='3b7d8740835b' WHERE alembic_version.version_num = '66cb674fe3a5';

-- Running upgrade 3b7d8740835b -> aea564da28d6

ALTER TABLE economy_control_state ADD COLUMN money_bucket_level DECIMAL(24, 2) DEFAULT '1000000' NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN money_bucket_max DECIMAL(24, 2) DEFAULT '1000000' NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN money_bucket_refill_rate DECIMAL(24, 2) DEFAULT '1000' NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN turnip_bucket_level DECIMAL(24, 2) DEFAULT '1000000' NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN turnip_bucket_max DECIMAL(24, 2) DEFAULT '1000000' NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN turnip_bucket_refill_rate DECIMAL(24, 2) DEFAULT '1000' NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN last_refill_at TIMESTAMP WITH TIME ZONE;

UPDATE alembic_version SET version_num='aea564da28d6' WHERE alembic_version.version_num = '3b7d8740835b';

-- Running upgrade aea564da28d6 -> c50b282fc969

ALTER TYPE futurestransactiontype ADD VALUE IF NOT EXISTS 'ADL';

UPDATE alembic_version SET version_num='c50b282fc969' WHERE alembic_version.version_num = 'aea564da28d6';

-- Running upgrade c50b282fc969 -> f3d2b1c4d5e6

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
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX ix_user_passkey_credential_id ON user_passkey (credential_id);

CREATE INDEX ix_user_passkey_user_id ON user_passkey (user_id);

UPDATE alembic_version SET version_num='f3d2b1c4d5e6' WHERE alembic_version.version_num = 'c50b282fc969';

-- Running upgrade f3d2b1c4d5e6 -> 0c1c8ef9f7d1

ALTER TABLE economy_control_state ADD COLUMN futures_payout_bucket_level DECIMAL(24, 2) DEFAULT '1000000' NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN futures_payout_bucket_max DECIMAL(24, 2) DEFAULT '1000000' NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN futures_payout_bucket_refill_rate DECIMAL(24, 2) DEFAULT '1000' NOT NULL;

UPDATE alembic_version SET version_num='0c1c8ef9f7d1' WHERE alembic_version.version_num = 'f3d2b1c4d5e6';

-- Running upgrade 0c1c8ef9f7d1 -> 6e52e6db5ec2

ALTER TABLE futures_transaction ADD COLUMN raw_pnl DECIMAL(24, 2);

ALTER TABLE futures_transaction ADD COLUMN paid_pnl DECIMAL(24, 2);

ALTER TABLE futures_transaction ADD COLUMN unpaid_pnl DECIMAL(24, 2);

UPDATE futures_transaction
            SET raw_pnl = pnl,
                paid_pnl = pnl,
                unpaid_pnl = 0;

ALTER TABLE futures_transaction ALTER COLUMN raw_pnl SET NOT NULL;

ALTER TABLE futures_transaction ALTER COLUMN paid_pnl SET NOT NULL;

ALTER TABLE futures_transaction ALTER COLUMN unpaid_pnl SET NOT NULL;

UPDATE alembic_version SET version_num='6e52e6db5ec2' WHERE alembic_version.version_num = '0c1c8ef9f7d1';

-- Running upgrade 6e52e6db5ec2 -> 658a84c40c71

CREATE TABLE auth_session (
    id SERIAL NOT NULL, 
    session_id VARCHAR(64) NOT NULL, 
    code VARCHAR(16) NOT NULL, 
    user_id VARCHAR, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    authenticated_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX ix_auth_session_code ON auth_session (code);

CREATE INDEX ix_auth_session_expires ON auth_session (expires_at);

CREATE UNIQUE INDEX ix_auth_session_session_id ON auth_session (session_id);

CREATE INDEX ix_auth_session_user_id ON auth_session (user_id);

UPDATE alembic_version SET version_num='658a84c40c71' WHERE alembic_version.version_num = '6e52e6db5ec2';

-- Running upgrade 658a84c40c71 -> 989c3eecb8c5

DROP INDEX ix_auth_session_code;

DROP INDEX ix_auth_session_expires;

DROP INDEX ix_auth_session_session_id;

DROP INDEX ix_auth_session_user_id;

CREATE INDEX ix_auth_session_expires_at ON auth_session (expires_at);

ALTER TABLE auth_session ADD UNIQUE (code);

ALTER TABLE auth_session ADD UNIQUE (session_id);

UPDATE alembic_version SET version_num='989c3eecb8c5' WHERE alembic_version.version_num = '658a84c40c71';

-- Running upgrade 989c3eecb8c5 -> c2f9b0e4d1a7

CREATE TABLE raid_map_progress (
    user_id VARCHAR(64) NOT NULL, 
    map_id BIGINT NOT NULL, 
    explored_rooms JSONB, 
    activated_waystones JSONB, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (user_id, map_id)
);

ALTER TABLE raid_profile DROP COLUMN activated_waystones;

UPDATE alembic_version SET version_num='c2f9b0e4d1a7' WHERE alembic_version.version_num = '989c3eecb8c5';

-- Running upgrade c2f9b0e4d1a7 -> 4f1c6d8b9a2e

UPDATE raid_profile
        SET warehouse = jsonb_set(warehouse, '{items}', 
(
    SELECT coalesce(jsonb_agg(
        CASE
            WHEN item->>'id' NOT LIKE '%%@%%'
            THEN jsonb_set(
                item, '{id}',
                to_jsonb((item->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
            )
            ELSE item
        END
        ORDER BY ordinality
    ), '[]'::jsonb)
    FROM jsonb_array_elements(warehouse->'items')
        WITH ORDINALITY AS t(item, ordinality)
)
)
        WHERE warehouse IS NOT NULL
          AND warehouse->'items' IS NOT NULL
          AND jsonb_array_length(warehouse->'items') > 0
          AND EXISTS (
              SELECT 1 FROM jsonb_array_elements(warehouse->'items') AS item
              WHERE item->>'id' NOT LIKE '%%@%%'
          );

UPDATE raid_profile
        SET loadout = jsonb_set(
            loadout,
            '{item_ids}',
            (
                SELECT coalesce(jsonb_agg(
                    coalesce(
                        (
                            SELECT w_item->>'id'
                            FROM jsonb_array_elements(warehouse->'items') AS w_item
                            WHERE split_part(w_item->>'id', '@', 1) = lid.val
                            LIMIT 1
                        ),
                        lid.val
                    )
                    ORDER BY lid.ordinality
                ), '[]'::jsonb)
                FROM jsonb_array_elements_text(loadout->'item_ids')
                    WITH ORDINALITY AS lid(val, ordinality)
            )
        )
        WHERE loadout IS NOT NULL
          AND loadout->'item_ids' IS NOT NULL
          AND jsonb_array_length(loadout->'item_ids') > 0
          AND warehouse IS NOT NULL;

UPDATE raid_session
        SET state_json = jsonb_set(state_json, '{inventory}', 
(
    SELECT coalesce(jsonb_agg(
        CASE
            WHEN item->>'id' NOT LIKE '%%@%%'
            THEN jsonb_set(
                item, '{id}',
                to_jsonb((item->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
            )
            ELSE item
        END
        ORDER BY ordinality
    ), '[]'::jsonb)
    FROM jsonb_array_elements(state_json->'inventory')
        WITH ORDINALITY AS t(item, ordinality)
)
)
        WHERE is_active = true
          AND state_json->'inventory' IS NOT NULL
          AND jsonb_array_length(state_json->'inventory') > 0
          AND EXISTS (
              SELECT 1 FROM jsonb_array_elements(state_json->'inventory') AS item
              WHERE item->>'id' NOT LIKE '%%@%%'
          );

UPDATE raid_session
            SET state_json = jsonb_set(
                state_json,
                '{equipment,weapon}',
            
    CASE
        WHEN state_json->'equipment'->'weapon' IS NOT NULL
         AND state_json->'equipment'->'weapon'->>'id' IS NOT NULL
         AND state_json->'equipment'->'weapon'->>'id' NOT LIKE '%%@%%'
        THEN jsonb_set(
            state_json->'equipment'->'weapon', '{id}',
            to_jsonb((state_json->'equipment'->'weapon'->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
        )
        ELSE state_json->'equipment'->'weapon'
    END

            )
            WHERE is_active = true
              AND state_json->'equipment' IS NOT NULL
              AND state_json->'equipment'->'weapon' IS NOT NULL
              AND state_json->'equipment'->'weapon'->>'id' IS NOT NULL
              AND state_json->'equipment'->'weapon'->>'id' NOT LIKE '%%@%%';

UPDATE raid_session
            SET state_json = jsonb_set(
                state_json,
                '{equipment,armor}',
            
    CASE
        WHEN state_json->'equipment'->'armor' IS NOT NULL
         AND state_json->'equipment'->'armor'->>'id' IS NOT NULL
         AND state_json->'equipment'->'armor'->>'id' NOT LIKE '%%@%%'
        THEN jsonb_set(
            state_json->'equipment'->'armor', '{id}',
            to_jsonb((state_json->'equipment'->'armor'->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
        )
        ELSE state_json->'equipment'->'armor'
    END

            )
            WHERE is_active = true
              AND state_json->'equipment' IS NOT NULL
              AND state_json->'equipment'->'armor' IS NOT NULL
              AND state_json->'equipment'->'armor'->>'id' IS NOT NULL
              AND state_json->'equipment'->'armor'->>'id' NOT LIKE '%%@%%';

UPDATE raid_session
            SET state_json = jsonb_set(
                state_json,
                '{equipment,accessory}',
            
    CASE
        WHEN state_json->'equipment'->'accessory' IS NOT NULL
         AND state_json->'equipment'->'accessory'->>'id' IS NOT NULL
         AND state_json->'equipment'->'accessory'->>'id' NOT LIKE '%%@%%'
        THEN jsonb_set(
            state_json->'equipment'->'accessory', '{id}',
            to_jsonb((state_json->'equipment'->'accessory'->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
        )
        ELSE state_json->'equipment'->'accessory'
    END

            )
            WHERE is_active = true
              AND state_json->'equipment' IS NOT NULL
              AND state_json->'equipment'->'accessory' IS NOT NULL
              AND state_json->'equipment'->'accessory'->>'id' IS NOT NULL
              AND state_json->'equipment'->'accessory'->>'id' NOT LIKE '%%@%%';

UPDATE raid_session
            SET state_json = jsonb_set(
                state_json,
                '{equipment,backpack}',
            
    CASE
        WHEN state_json->'equipment'->'backpack' IS NOT NULL
         AND state_json->'equipment'->'backpack'->>'id' IS NOT NULL
         AND state_json->'equipment'->'backpack'->>'id' NOT LIKE '%%@%%'
        THEN jsonb_set(
            state_json->'equipment'->'backpack', '{id}',
            to_jsonb((state_json->'equipment'->'backpack'->>'id') || '@' || replace(gen_random_uuid()::text, '-', ''))
        )
        ELSE state_json->'equipment'->'backpack'
    END

            )
            WHERE is_active = true
              AND state_json->'equipment' IS NOT NULL
              AND state_json->'equipment'->'backpack' IS NOT NULL
              AND state_json->'equipment'->'backpack'->>'id' IS NOT NULL
              AND state_json->'equipment'->'backpack'->>'id' NOT LIKE '%%@%%';

UPDATE alembic_version SET version_num='4f1c6d8b9a2e' WHERE alembic_version.version_num = 'c2f9b0e4d1a7';

-- Running upgrade 4f1c6d8b9a2e -> 6220b47a2409

CREATE TABLE market_order (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    side VARCHAR(10) NOT NULL, 
    item_category VARCHAR(20) NOT NULL, 
    item_key VARCHAR(100) NOT NULL, 
    item_quality VARCHAR(30) NOT NULL, 
    price DECIMAL(24, 2) NOT NULL, 
    quantity INTEGER NOT NULL, 
    filled_quantity INTEGER NOT NULL, 
    escrow_amount DECIMAL(24, 2) NOT NULL, 
    item_snapshot JSONB, 
    pal_min_level INTEGER, 
    status VARCHAR(20) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_market_order_expires ON market_order (expires_at);

CREATE INDEX ix_market_order_match ON market_order (item_category, item_key, item_quality, side, status);

CREATE INDEX ix_market_order_user_id ON market_order (user_id);

CREATE INDEX ix_market_order_user_status ON market_order (user_id, status);

CREATE TABLE market_order_fill (
    id SERIAL NOT NULL, 
    buy_order_id INTEGER NOT NULL, 
    sell_order_id INTEGER NOT NULL, 
    quantity INTEGER NOT NULL, 
    price DECIMAL(24, 2) NOT NULL, 
    total DECIMAL(24, 2) NOT NULL, 
    fee DECIMAL(24, 2) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(buy_order_id) REFERENCES market_order (id), 
    FOREIGN KEY(sell_order_id) REFERENCES market_order (id)
);

CREATE INDEX ix_market_order_fill_buy ON market_order_fill (buy_order_id);

CREATE INDEX ix_market_order_fill_sell ON market_order_fill (sell_order_id);

ALTER TABLE pal ADD COLUMN locked_for_order_id INTEGER;

ALTER TABLE pal ADD CONSTRAINT fk_pal_locked_for_order_id FOREIGN KEY(locked_for_order_id) REFERENCES market_order (id);

ALTER TABLE pal_egg ADD COLUMN locked_for_order_id INTEGER;

ALTER TABLE pal_egg ADD CONSTRAINT fk_pal_egg_locked_for_order_id FOREIGN KEY(locked_for_order_id) REFERENCES market_order (id);

UPDATE alembic_version SET version_num='6220b47a2409' WHERE alembic_version.version_num = '4f1c6d8b9a2e';

-- Running upgrade 6220b47a2409 -> babe30652ada

ALTER TABLE economy_control_state ADD COLUMN coverage_ratio DECIMAL(24, 8) DEFAULT 1 NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN coverage_ema DECIMAL(24, 8) DEFAULT 1 NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN bank_liability DECIMAL(24, 2) DEFAULT 0 NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN runtime_metrics_updated_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE economy_control_state ALTER COLUMN coverage_ratio DROP DEFAULT;

ALTER TABLE economy_control_state ALTER COLUMN coverage_ema DROP DEFAULT;

ALTER TABLE economy_control_state ALTER COLUMN bank_liability DROP DEFAULT;

UPDATE alembic_version SET version_num='babe30652ada' WHERE alembic_version.version_num = '6220b47a2409';

-- Running upgrade babe30652ada -> 7aa54c2ecbc9

ALTER TABLE currency_rate_state ADD COLUMN baseline_wallet_supply DECIMAL(24, 2) DEFAULT 0 NOT NULL;

ALTER TABLE currency_rate_state ADD COLUMN baseline_game_supply DECIMAL(24, 6) DEFAULT 0 NOT NULL;

UPDATE currency_rate_state
        SET baseline_wallet_supply = last_wallet_supply,
            baseline_game_supply = last_game_supply;

ALTER TABLE currency_rate_state ALTER COLUMN baseline_wallet_supply DROP DEFAULT;

ALTER TABLE currency_rate_state ALTER COLUMN baseline_game_supply DROP DEFAULT;

UPDATE alembic_version SET version_num='7aa54c2ecbc9' WHERE alembic_version.version_num = 'babe30652ada';

-- Running upgrade babe30652ada -> b0b0f8a6c4d1

CREATE TABLE partition_migration_state (
    table_name TEXT NOT NULL, 
    shadow_table_name TEXT NOT NULL, 
    snapshot_time TIMESTAMP WITH TIME ZONE, 
    snapshot_bigint BIGINT, 
    cursor_time TIMESTAMP WITH TIME ZONE, 
    cursor_text TEXT, 
    cursor_bigint BIGINT, 
    bulk_copy_completed_at TIMESTAMP WITH TIME ZONE, 
    cutover_completed_at TIMESTAMP WITH TIME ZONE, 
    created_at TIMESTAMP WITH TIME ZONE DEFAULT timezone('UTC', clock_timestamp()) NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT timezone('UTC', clock_timestamp()) NOT NULL, 
    PRIMARY KEY (table_name)
);

CREATE TABLE raid_action_log_partitioned (
    id BIGINT GENERATED BY DEFAULT AS IDENTITY, 
    session_id VARCHAR(32) NOT NULL, 
    seq INTEGER NOT NULL, 
    action JSONB NOT NULL, 
    effects JSONB NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id, created_at)
)
 PARTITION BY RANGE (created_at);

CREATE INDEX ix_raid_action_log_partitioned_session_id ON raid_action_log_partitioned (session_id);

CREATE INDEX ix_raid_action_log_partitioned_session_seq ON raid_action_log_partitioned (session_id, seq);

SELECT MIN(created_at), MAX(created_at) FROM raid_action_log_partitioned;

CREATE TABLE IF NOT EXISTS raid_action_log_p20260101
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260102
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260103
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260104
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260105
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260106
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260107
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260108
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260109
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260110
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260111
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260112
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260113
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260114
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260115
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260116
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260117
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260118
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260119
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260120
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260121
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260122
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260123
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260124
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260125
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260126
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260127
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260128
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260129
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260130
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260131
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260201
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260202
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260203
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260204
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260205
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260206
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260207
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260208
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260209
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260210
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260211
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260212
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260213
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260214
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260215
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260216
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260217
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260218
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260219
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260220
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260221
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260222
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260223
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260224
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260225
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260226
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260227
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260228
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260301
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260302
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260303
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260304
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260305
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260306
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260307
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260308
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260309
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260310
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260311
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260312
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260313
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260314
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260315
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260316
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260317
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260318
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260319
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260320
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260321
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260322
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260323
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260324
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260325
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260326
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260327
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260328
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260329
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260330
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260331
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260401
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260402
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260403
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260404
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260405
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260406
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260407
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260408
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260409
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260410
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260411
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260412
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260413
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260414
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260415
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260416
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260417
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260418
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260419
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260420
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260421
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260422
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260423
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260424
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260425
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260426
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260427
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260428
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260429
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260430
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260501
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260502
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260503
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260504
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260505
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260506
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260507
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260508
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260509
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260510
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260511
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260512
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260513
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260514
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260515
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260516
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260517
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260518
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260519
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260520
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260521
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260522
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260523
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260524
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260525
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260526
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260527
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260528
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260529
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260530
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260531
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260601
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260602
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260603
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260604
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260605
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260606
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260607
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260608
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260609
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260610
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260611
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260612
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260613
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260614
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260615
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260616
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260617
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260618
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260619
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260620
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260621
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260622
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260623
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260624
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260625
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260626
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260627
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260628
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260629
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260630
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260701
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260702
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260703
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260704
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260705
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260706
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260707
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260708
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260709
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260710
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260711
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260712
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260713
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260714
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260715
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260716
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260717
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260718
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260719
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260720
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260721
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260722
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260723
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260724
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260725
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260726
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260727
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260728
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260729
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260730
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260731
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260801
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260802
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260803
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260804
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260805
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260806
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260807
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260808
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260809
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260810
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260811
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260812
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260813
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260814
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260815
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260816
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260817
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260818
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260819
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260820
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260821
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260822
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260823
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260824
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260825
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260826
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260827
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260828
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260829
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260830
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260831
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260901
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260902
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260903
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260904
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260905
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260906
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260907
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260908
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260909
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260910
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260911
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260912
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260913
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260914
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260915
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260916
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260917
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260918
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260919
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260920
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260921
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260922
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260923
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260924
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260925
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260926
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260927
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260928
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260929
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20260930
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261001
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261002
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261003
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261004
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261005
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261006
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261007
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261008
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261009
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261010
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261011
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261012
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261013
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261014
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261015
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261016
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261017
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261018
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261019
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261020
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261021
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261022
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261023
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261024
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261025
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261026
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261027
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261028
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261029
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261030
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261031
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261101
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261102
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261103
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261104
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261105
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261106
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261107
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261108
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261109
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261110
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261111
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261112
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261113
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261114
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261115
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261116
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261117
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261118
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261119
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261120
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261121
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261122
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261123
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261124
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261125
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261126
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261127
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261128
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261129
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261130
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261201
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261202
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261203
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261204
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261205
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261206
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261207
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261208
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261209
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261210
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261211
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261212
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261213
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261214
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261215
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261216
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261217
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261218
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261219
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261220
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261221
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261222
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261223
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261224
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261225
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261226
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261227
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261228
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261229
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261230
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20261231
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270101
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270102
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270103
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270104
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270105
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270106
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270107
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270108
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270109
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270110
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270111
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270112
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270113
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270114
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270115
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270116
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270117
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270118
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270119
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270120
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270121
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270122
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270123
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270124
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270125
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270126
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270127
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270128
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270129
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270130
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270131
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270201
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270202
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270203
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270204
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270205
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270206
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270207
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270208
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270209
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270210
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270211
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270212
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270213
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270214
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270215
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270216
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270217
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270218
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270219
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270220
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270221
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270222
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270223
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270224
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270225
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270226
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270227
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270228
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270301
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270302
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270303
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270304
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270305
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270306
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270307
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270308
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270309
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270310
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270311
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270312
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270313
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270314
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270315
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270316
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270317
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270318
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270319
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270320
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270321
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270322
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270323
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270324
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270325
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270326
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270327
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270328
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270329
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270330
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270331
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270401
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270402
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270403
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270404
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270405
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270406
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270407
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270408
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270409
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270410
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270411
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270412
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270413
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270414
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270415
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270416
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270417
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270418
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270419
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270420
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270421
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270422
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270423
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270424
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270425
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270426
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270427
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270428
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270429
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270430
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270501
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270502
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270503
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270504
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270505
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270506
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270507
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270508
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270509
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270510
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270511
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270512
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270513
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270514
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270515
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270516
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270517
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270518
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270519
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270520
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270521
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270522
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270523
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270524
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270525
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270526
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270527
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270528
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270529
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270530
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270531
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270601
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270602
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270603
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270604
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270605
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270606
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270607
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270608
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270609
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270610
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270611
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270612
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270613
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270614
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270615
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270616
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270617
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270618
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270619
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270620
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270621
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270622
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270623
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270624
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270625
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270626
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270627
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270628
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270629
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270630
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270701
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270702
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270703
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270704
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270705
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270706
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270707
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270708
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270709
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270710
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270711
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270712
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270713
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270714
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270715
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270716
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270717
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270718
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270719
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270720
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270721
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270722
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270723
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270724
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270725
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270726
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270727
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270728
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270729
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270730
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270731
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270801
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270802
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270803
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270804
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270805
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270806
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270807
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270808
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270809
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270810
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270811
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270812
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270813
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270814
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270815
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270816
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270817
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270818
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270819
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270820
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270821
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270822
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270823
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270824
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270825
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270826
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270827
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270828
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270829
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270830
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270831
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270901
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270902
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270903
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270904
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270905
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270906
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270907
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270908
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270909
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270910
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270911
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270912
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270913
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270914
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270915
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270916
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270917
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270918
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270919
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270920
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270921
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270922
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270923
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270924
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270925
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270926
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270927
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270928
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270929
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20270930
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271001
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271002
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271003
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271004
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271005
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271006
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271007
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271008
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271009
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271010
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271011
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271012
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271013
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271014
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271015
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271016
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271017
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271018
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271019
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271020
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271021
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271022
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271023
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271024
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271025
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271026
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271027
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271028
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271029
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271030
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271031
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271101
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271102
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271103
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271104
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271105
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271106
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271107
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271108
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271109
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271110
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271111
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271112
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271113
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271114
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271115
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271116
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271117
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271118
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271119
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271120
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271121
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271122
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271123
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271124
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271125
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271126
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271127
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271128
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271129
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271130
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271201
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271202
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271203
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271204
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271205
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271206
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271207
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271208
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271209
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271210
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271211
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271212
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271213
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271214
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271215
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271216
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271217
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271218
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271219
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271220
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271221
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271222
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271223
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271224
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271225
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271226
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271227
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271228
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271229
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271230
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20271231
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280101
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280102
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280103
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280104
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280105
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280106
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280107
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280108
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280109
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280110
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280111
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280112
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280113
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280114
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280115
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280116
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280117
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280118
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280119
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280120
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280121
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280122
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280123
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280124
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280125
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280126
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280127
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280128
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280129
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280130
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280131
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280201
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280202
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280203
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280204
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280205
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280206
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280207
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280208
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280209
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280210
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280211
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280212
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280213
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280214
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280215
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280216
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280217
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280218
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280219
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280220
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280221
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280222
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280223
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280224
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280225
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280226
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280227
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280228
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280229
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280301
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280302
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280303
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280304
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280305
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280306
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280307
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280308
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280309
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280310
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280311
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280312
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280313
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280314
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280315
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280316
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280317
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280318
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280319
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280320
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280321
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280322
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280323
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280324
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280325
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280326
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280327
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280328
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280329
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280330
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280331
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280401
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280402
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280403
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280404
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280405
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280406
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280407
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280408
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280409
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280410
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280411
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280412
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280413
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280414
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280415
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280416
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280417
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280418
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280419
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280420
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280421
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280422
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280423
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280424
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280425
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280426
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280427
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280428
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280429
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280430
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280501
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280502
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280503
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280504
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280505
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280506
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280507
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280508
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280509
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280510
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280511
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280512
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280513
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280514
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280515
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280516
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280517
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280518
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280519
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280520
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280521
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280522
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280523
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280524
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280525
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280526
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280527
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280528
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280529
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280530
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280531
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280601
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280602
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280603
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280604
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280605
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280606
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280607
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280608
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280609
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280610
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280611
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280612
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280613
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280614
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280615
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280616
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280617
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280618
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280619
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280620
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280621
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280622
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280623
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280624
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280625
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280626
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280627
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280628
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280629
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280630
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280701
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280702
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280703
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280704
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280705
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280706
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280707
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280708
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280709
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280710
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280711
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280712
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280713
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280714
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280715
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280716
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280717
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280718
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280719
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280720
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280721
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280722
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280723
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280724
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280725
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280726
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280727
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280728
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280729
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280730
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280731
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280801
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280802
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280803
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280804
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280805
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280806
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280807
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280808
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280809
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280810
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280811
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280812
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280813
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280814
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280815
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280816
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280817
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280818
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280819
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280820
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280821
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280822
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280823
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280824
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280825
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280826
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280827
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280828
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280829
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280830
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280831
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280901
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280902
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280903
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280904
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280905
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280906
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280907
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280908
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280909
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280910
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280911
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280912
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280913
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280914
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280915
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280916
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280917
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280918
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280919
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280920
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280921
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280922
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280923
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280924
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280925
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280926
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280927
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280928
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280929
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20280930
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281001
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281002
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281003
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281004
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281005
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281006
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281007
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281008
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281009
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281010
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281011
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281012
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281013
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281014
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281015
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281016
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281017
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281018
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281019
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281020
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281021
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281022
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281023
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281024
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281025
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281026
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281027
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281028
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281029
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281030
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281031
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281101
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281102
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281103
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281104
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281105
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281106
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281107
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281108
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281109
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281110
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281111
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281112
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281113
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281114
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281115
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281116
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281117
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281118
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281119
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281120
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281121
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281122
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281123
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281124
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281125
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281126
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281127
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281128
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281129
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281130
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281201
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281202
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281203
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281204
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281205
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281206
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281207
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281208
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281209
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281210
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281211
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281212
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281213
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281214
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281215
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281216
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281217
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281218
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281219
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281220
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281221
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281222
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281223
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281224
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281225
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281226
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281227
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281228
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281229
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281230
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20281231
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290101
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290102
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290103
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290104
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290105
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290106
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290107
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290108
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290109
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290110
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290111
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290112
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290113
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290114
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290115
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290116
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290117
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290118
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290119
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290120
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290121
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290122
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290123
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290124
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290125
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290126
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290127
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290128
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290129
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290130
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290131
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290201
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290202
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290203
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290204
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290205
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290206
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290207
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290208
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290209
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290210
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290211
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290212
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290213
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290214
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290215
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290216
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290217
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290218
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290219
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290220
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290221
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290222
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290223
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290224
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290225
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290226
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290227
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290228
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290301
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290302
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290303
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290304
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290305
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290306
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290307
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290308
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290309
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290310
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290311
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290312
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290313
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290314
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290315
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290316
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290317
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290318
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290319
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290320
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290321
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290322
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290323
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290324
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290325
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290326
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290327
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290328
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290329
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290330
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290331
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290401
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290402
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290403
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290404
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290405
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290406
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290407
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290408
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290409
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290410
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290411
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290412
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290413
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290414
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290415
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290416
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290417
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290418
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290419
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290420
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290421
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290422
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290423
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290424
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290425
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290426
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290427
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290428
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290429
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290430
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290501
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290502
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290503
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290504
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290505
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290506
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290507
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290508
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290509
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290510
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290511
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290512
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290513
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290514
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290515
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290516
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290517
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290518
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290519
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290520
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290521
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290522
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290523
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290524
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290525
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290526
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290527
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290528
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290529
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290530
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290531
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290601
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290602
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290603
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290604
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290605
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290606
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290607
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290608
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290609
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290610
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290611
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290612
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290613
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290614
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290615
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290616
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290617
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290618
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290619
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290620
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290621
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290622
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290623
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290624
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290625
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290626
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290627
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290628
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290629
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290630
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290701
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290702
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290703
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290704
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290705
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290706
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290707
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290708
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290709
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290710
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290711
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290712
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290713
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290714
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290715
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290716
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290717
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290718
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290719
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290720
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290721
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290722
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290723
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290724
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290725
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290726
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290727
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290728
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290729
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290730
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290731
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290801
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290802
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290803
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290804
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290805
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290806
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290807
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290808
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290809
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290810
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290811
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290812
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290813
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290814
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290815
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290816
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290817
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290818
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290819
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290820
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290821
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290822
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290823
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290824
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290825
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290826
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290827
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290828
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290829
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290830
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290831
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290901
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290902
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290903
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290904
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290905
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290906
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290907
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290908
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290909
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290910
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290911
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290912
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290913
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290914
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290915
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290916
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290917
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290918
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290919
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290920
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290921
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290922
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290923
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290924
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290925
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290926
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290927
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290928
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290929
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20290930
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291001
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291002
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291003
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291004
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291005
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291006
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291007
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291008
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291009
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291010
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291011
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291012
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291013
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291014
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291015
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291016
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291017
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291018
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291019
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291020
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291021
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291022
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291023
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291024
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291025
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291026
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291027
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291028
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291029
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291030
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291031
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291101
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291102
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291103
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291104
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291105
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291106
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291107
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291108
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291109
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291110
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291111
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291112
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291113
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291114
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291115
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291116
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291117
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291118
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291119
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291120
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291121
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291122
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291123
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291124
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291125
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291126
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291127
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291128
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291129
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291130
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291201
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291202
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291203
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291204
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291205
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291206
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291207
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291208
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291209
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291210
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291211
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291212
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291213
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291214
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291215
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291216
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291217
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291218
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291219
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291220
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291221
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291222
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291223
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291224
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291225
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291226
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291227
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291228
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291229
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291230
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20291231
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300101
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300102
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300103
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300104
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300105
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300106
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300107
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300108
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300109
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300110
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300111
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300112
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300113
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300114
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300115
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300116
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300117
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300118
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300119
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300120
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300121
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300122
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300123
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300124
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300125
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300126
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300127
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300128
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300129
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300130
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300131
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300201
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300202
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300203
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300204
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300205
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300206
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300207
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300208
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300209
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300210
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300211
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300212
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300213
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300214
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300215
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300216
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300217
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300218
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300219
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300220
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300221
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300222
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300223
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300224
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300225
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300226
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300227
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300228
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300301
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300302
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300303
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300304
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300305
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300306
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300307
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300308
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300309
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300310
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300311
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300312
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300313
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300314
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300315
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300316
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300317
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300318
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300319
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300320
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300321
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300322
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300323
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300324
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300325
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300326
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300327
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300328
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300329
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300330
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300331
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300401
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300402
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300403
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300404
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300405
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300406
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300407
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300408
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300409
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300410
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300411
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300412
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300413
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300414
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300415
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300416
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300417
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300418
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300419
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300420
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300421
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300422
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300423
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300424
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300425
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300426
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300427
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300428
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300429
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300430
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300501
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300502
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300503
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300504
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300505
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300506
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300507
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300508
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300509
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300510
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300511
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300512
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300513
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300514
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300515
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300516
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300517
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300518
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300519
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300520
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300521
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300522
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300523
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300524
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300525
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300526
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300527
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300528
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300529
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300530
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300531
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300601
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300602
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300603
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300604
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300605
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300606
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300607
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300608
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300609
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300610
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300611
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300612
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300613
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300614
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300615
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300616
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300617
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300618
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300619
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300620
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300621
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300622
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300623
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300624
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300625
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300626
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300627
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300628
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300629
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300630
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300701
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300702
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300703
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300704
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300705
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300706
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300707
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300708
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300709
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300710
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300711
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300712
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300713
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300714
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300715
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300716
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300717
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300718
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300719
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300720
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300721
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300722
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300723
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300724
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300725
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300726
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300727
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300728
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300729
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300730
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300731
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300801
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300802
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300803
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300804
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300805
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300806
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300807
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300808
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300809
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300810
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300811
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300812
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300813
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300814
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300815
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300816
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300817
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300818
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300819
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300820
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300821
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300822
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300823
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300824
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300825
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300826
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300827
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300828
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300829
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300830
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300831
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300901
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300902
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300903
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300904
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300905
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300906
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300907
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300908
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300909
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300910
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300911
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300912
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300913
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300914
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300915
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300916
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300917
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300918
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300919
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300920
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300921
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300922
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300923
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300924
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300925
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300926
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300927
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300928
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300929
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20300930
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301001
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301002
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301003
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301004
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301005
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301006
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301007
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301008
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301009
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301010
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301011
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301012
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301013
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301014
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301015
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301016
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301017
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301018
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301019
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301020
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301021
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301022
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301023
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301024
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301025
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301026
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301027
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301028
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301029
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301030
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301031
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301101
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301102
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301103
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301104
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301105
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301106
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301107
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301108
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301109
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301110
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301111
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301112
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301113
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301114
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301115
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301116
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301117
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301118
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301119
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301120
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301121
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301122
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301123
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301124
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301125
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301126
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301127
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301128
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301129
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301130
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301201
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301202
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301203
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301204
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301205
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301206
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301207
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301208
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301209
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301210
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301211
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301212
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301213
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301214
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301215
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301216
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301217
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301218
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301219
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301220
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301221
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301222
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301223
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301224
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301225
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301226
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301227
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301228
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301229
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301230
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS raid_action_log_p20301231
            PARTITION OF raid_action_log_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

INSERT INTO alembic_version (version_num) VALUES ('b0b0f8a6c4d1') RETURNING alembic_version.version_num;

-- Running upgrade b0b0f8a6c4d1 -> e4c8f6172a3b

ALTER TABLE event_processor_offsets ADD COLUMN last_processed_timestamp TIMESTAMP WITH TIME ZONE;

UPDATE alembic_version SET version_num='e4c8f6172a3b' WHERE alembic_version.version_num = 'b0b0f8a6c4d1';

-- Running upgrade e4c8f6172a3b -> 4d5e6f7a8b9c

CREATE TABLE websocket_events_partitioned (
    id BIGINT GENERATED BY DEFAULT AS IDENTITY, 
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL, 
    user_id TEXT NOT NULL, 
    event TEXT NOT NULL, 
    data JSONB NOT NULL, 
    PRIMARY KEY (id, timestamp)
)
 PARTITION BY RANGE (timestamp);

CREATE INDEX ix_websocket_events_partitioned_timestamp_id ON websocket_events_partitioned (timestamp, id);

CREATE INDEX ix_websocket_events_partitioned_user_id_timestamp_id ON websocket_events_partitioned (user_id, timestamp, id);

CREATE INDEX ix_websocket_events_partitioned_event_timestamp_id ON websocket_events_partitioned (event, timestamp, id);

SELECT MIN(timestamp), MAX(timestamp) FROM websocket_events_partitioned;

CREATE TABLE IF NOT EXISTS websocket_events_p20241230
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250106
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250113
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250120
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250127
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250203
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250210
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250217
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250224
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250303
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250310
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250317
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250324
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250331
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250407
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250414
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250421
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250428
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250505
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250512
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250519
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250526
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250602
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250609
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250616
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250623
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250630
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250707
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250714
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250721
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250728
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250804
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250811
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250818
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250825
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250901
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250908
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250915
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250922
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20250929
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251006
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251013
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251020
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251027
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251103
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251110
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251117
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251124
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251201
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251208
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251215
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251222
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20251229
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260105
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260112
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260119
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260126
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260202
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260209
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260216
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260223
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260302
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260309
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260316
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260323
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260330
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260406
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260413
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260420
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260427
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260504
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260511
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260518
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260525
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260601
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260608
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260615
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260622
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260629
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260706
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260713
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260720
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260727
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260803
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260810
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260817
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260824
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260831
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260907
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260914
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260921
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20260928
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261005
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261012
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261019
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261026
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261102
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261109
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261116
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261123
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261130
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261207
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261214
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261221
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20261228
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270104
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270111
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270118
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270125
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270201
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270208
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270215
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270222
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270301
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270308
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270315
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270322
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270329
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270405
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270412
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270419
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270426
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270503
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270510
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270517
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270524
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270531
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270607
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270614
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270621
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270628
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270705
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270712
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270719
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270726
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270802
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270809
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270816
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270823
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270830
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270906
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270913
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270920
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20270927
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271004
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271011
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271018
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271025
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271101
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271108
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271115
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271122
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271129
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271206
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271213
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271220
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20271227
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280103
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280110
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280117
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280124
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280131
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280207
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280214
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280221
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280228
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280306
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280313
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280320
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280327
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280403
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280410
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280417
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280424
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280501
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280508
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280515
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280522
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280529
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280605
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280612
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280619
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280626
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280703
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280710
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280717
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280724
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280731
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280807
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280814
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280821
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280828
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280904
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280911
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280918
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20280925
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281002
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281009
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281016
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281023
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281030
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281106
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281113
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281120
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281127
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281204
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281211
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281218
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20281225
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290101
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290108
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290115
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290122
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290129
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290205
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290212
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290219
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290226
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290305
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290312
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290319
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290326
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290402
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290409
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290416
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290423
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290430
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290507
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290514
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290521
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290528
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290604
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290611
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290618
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290625
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290702
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290709
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290716
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290723
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290730
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290806
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290813
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290820
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290827
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290903
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290910
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290917
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20290924
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291001
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291008
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291015
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291022
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291029
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291105
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291112
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291119
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291126
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291203
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291210
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291217
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291224
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20291231
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300107
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300114
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300121
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300128
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300204
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300211
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300218
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300225
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300304
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300311
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300318
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300325
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300401
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300408
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300415
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300422
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300429
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300506
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300513
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300520
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300527
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300603
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300610
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300617
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300624
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300701
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300708
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300715
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300722
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300729
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300805
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300812
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300819
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300826
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300902
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300909
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300916
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300923
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20300930
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301007
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301014
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301021
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301028
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301104
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301111
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301118
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301125
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301202
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301209
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301216
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS websocket_events_p20301223
            PARTITION OF websocket_events_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

UPDATE alembic_version SET version_num='4d5e6f7a8b9c' WHERE alembic_version.version_num = 'e4c8f6172a3b';

-- Running upgrade 4d5e6f7a8b9c -> 9a0b1c2d3e4f

CREATE TABLE messages_partitioned (
    message_id TEXT NOT NULL, 
    room_id TEXT NOT NULL, 
    sent_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    sent_by TEXT NOT NULL, 
    content_type TEXT NOT NULL, 
    content_text TEXT, 
    content_tsv TSVECTOR, 
    attachment_url TEXT, 
    attachment_file TEXT, 
    sticker_id TEXT, 
    alt_text TEXT, 
    metadata JSONB, 
    raw_data JSONB NOT NULL, 
    source TEXT NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE, 
    is_deleted BOOLEAN DEFAULT false NOT NULL, 
    deleted_at TIMESTAMP WITH TIME ZONE, 
    deleted_by TEXT, 
    is_recalled BOOLEAN DEFAULT false NOT NULL, 
    is_edited BOOLEAN DEFAULT false NOT NULL, 
    history JSONB, 
    reference_message_id TEXT, 
    reference_data JSONB, 
    PRIMARY KEY (message_id, sent_at)
)
 PARTITION BY RANGE (sent_at);

CREATE INDEX ix_messages_partitioned_room_id ON messages_partitioned (room_id);

CREATE INDEX ix_messages_partitioned_sent_at ON messages_partitioned (sent_at);

CREATE INDEX ix_messages_partitioned_sent_by ON messages_partitioned (sent_by);

CREATE INDEX ix_messages_partitioned_content_type ON messages_partitioned (content_type);

CREATE INDEX ix_messages_partitioned_attachment_file ON messages_partitioned (attachment_file);

CREATE INDEX ix_messages_partitioned_sticker_id ON messages_partitioned (sticker_id);

CREATE INDEX ix_messages_partitioned_source ON messages_partitioned (source);

CREATE INDEX ix_messages_partitioned_is_deleted ON messages_partitioned (is_deleted);

CREATE INDEX ix_messages_partitioned_deleted_by ON messages_partitioned (deleted_by);

CREATE INDEX ix_messages_partitioned_is_recalled ON messages_partitioned (is_recalled);

CREATE INDEX ix_messages_partitioned_is_edited ON messages_partitioned (is_edited);

CREATE INDEX ix_messages_partitioned_reference_message_id ON messages_partitioned (reference_message_id);

CREATE INDEX idx_messages_partitioned_source_created_at_id ON messages_partitioned (source, created_at, message_id);

CREATE INDEX idx_messages_partitioned_room_id_sent_at ON messages_partitioned (room_id, sent_at);

CREATE INDEX idx_messages_partitioned_sent_by_sent_at_id ON messages_partitioned (sent_by, sent_at, message_id);

CREATE INDEX idx_messages_partitioned_content_tsv ON messages_partitioned USING gin (content_tsv);

SELECT MIN(sent_at), MAX(sent_at) FROM messages_partitioned;

CREATE TABLE IF NOT EXISTS messages_p202001
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202002
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202003
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202004
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202005
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202006
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202007
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202008
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202009
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202010
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202011
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202012
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202101
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202102
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202103
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202104
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202105
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202106
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202107
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202108
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202109
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202110
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202111
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202112
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202201
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202202
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202203
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202204
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202205
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202206
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202207
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202208
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202209
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202210
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202211
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202212
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202301
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202302
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202303
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202304
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202305
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202306
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202307
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202308
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202309
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202310
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202311
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202312
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202401
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202402
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202403
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202404
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202405
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202406
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202407
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202408
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202409
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202410
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202411
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202412
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202501
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202502
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202503
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202504
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202505
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202506
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202507
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202508
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202509
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202510
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202511
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202512
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202601
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202602
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202603
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202604
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202605
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202606
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202607
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202608
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202609
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202610
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202611
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202612
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202701
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202702
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202703
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202704
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202705
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202706
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202707
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202708
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202709
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202710
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202711
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202712
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202801
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202802
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202803
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202804
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202805
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202806
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202807
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202808
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202809
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202810
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202811
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202812
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202901
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202902
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202903
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202904
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202905
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202906
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202907
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202908
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202909
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202910
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202911
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p202912
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203001
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203002
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203003
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203004
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203005
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203006
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203007
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203008
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203009
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203010
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203011
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

CREATE TABLE IF NOT EXISTS messages_p203012
            PARTITION OF messages_partitioned
            FOR VALUES FROM (NULL) TO (NULL);

UPDATE alembic_version SET version_num='9a0b1c2d3e4f' WHERE alembic_version.version_num = '4d5e6f7a8b9c';

-- Running upgrade 9a0b1c2d3e4f -> 7c8d9e0f1a2b

COMMIT;

CREATE INDEX CONCURRENTLY IF NOT EXISTS
            idx_outgoing_commands_pending_account_id
            ON outgoing_commands (account_user_id, id)
            WHERE status = 'pending';

BEGIN;

UPDATE alembic_version SET version_num='7c8d9e0f1a2b' WHERE alembic_version.version_num = '9a0b1c2d3e4f';

-- Running upgrade 7c8d9e0f1a2b -> 8e1f2a3b4c5d

CREATE OR REPLACE FUNCTION ensure_time_partitions(
    p_table_name text DEFAULT NULL,
    p_anchor timestamptz DEFAULT timezone('UTC', clock_timestamp()),
    p_apply boolean DEFAULT true
)
RETURNS TABLE (
    table_name text,
    parent_name text,
    child_name text,
    partition_start timestamptz,
    partition_end timestamptz,
    applied boolean
)
LANGUAGE plpgsql
AS $$
DECLARE
    spec RECORD;
    live_relkind "char";
    shadow_relkind "char";
    current_start timestamptz;
    matched_spec boolean := false;
    step_index integer;
BEGIN
    FOR spec IN
        SELECT
            specs.table_name,
            specs.shadow_table_name,
            specs.cadence,
            specs.lookahead
        FROM (
            VALUES
                ('messages', 'messages_partitioned', 'month', 3),
                ('raid_action_log', 'raid_action_log_partitioned', 'day', 14),
                ('websocket_events', 'websocket_events_partitioned', 'week', 8)
        ) AS specs(table_name, shadow_table_name, cadence, lookahead)
        WHERE p_table_name IS NULL OR specs.table_name = p_table_name
        ORDER BY specs.table_name
    LOOP
        matched_spec := true;

        SELECT relkind INTO live_relkind
        FROM pg_class
        WHERE relname = spec.table_name;

        SELECT relkind INTO shadow_relkind
        FROM pg_class
        WHERE relname = spec.shadow_table_name;

        IF live_relkind = 'p' THEN
            parent_name := spec.table_name;
        ELSIF shadow_relkind = 'p' THEN
            parent_name := spec.shadow_table_name;
        ELSE
            CONTINUE;
        END IF;

        IF spec.cadence = 'day' THEN
            current_start := timezone(
                'UTC',
                date_trunc('day', timezone('UTC', p_anchor))
            );
        ELSIF spec.cadence = 'week' THEN
            current_start := timezone(
                'UTC',
                date_trunc('week', timezone('UTC', p_anchor))
            );
        ELSIF spec.cadence = 'month' THEN
            current_start := timezone(
                'UTC',
                date_trunc('month', timezone('UTC', p_anchor))
            );
        ELSE
            RAISE EXCEPTION 'Unsupported partition cadence: %', spec.cadence;
        END IF;

        FOR step_index IN 0..spec.lookahead LOOP
            IF spec.cadence = 'month' THEN
                partition_end := timezone(
                    'UTC',
                    date_trunc('month', timezone('UTC', current_start + INTERVAL '1 month'))
                );
                child_name := spec.table_name
                    || '_p'
                    || to_char(timezone('UTC', current_start), 'YYYYMM');
            ELSIF spec.cadence = 'week' THEN
                partition_end := current_start + INTERVAL '7 days';
                child_name := spec.table_name
                    || '_p'
                    || to_char(timezone('UTC', current_start), 'YYYYMMDD');
            ELSE
                partition_end := current_start + INTERVAL '1 day';
                child_name := spec.table_name
                    || '_p'
                    || to_char(timezone('UTC', current_start), 'YYYYMMDD');
            END IF;

            IF NOT EXISTS (
                SELECT 1
                FROM pg_inherits
                JOIN pg_class parent ON parent.oid = pg_inherits.inhparent
                JOIN pg_class child ON child.oid = pg_inherits.inhrelid
                WHERE parent.relname = parent_name
                  AND child.relname = child_name
            ) THEN
                table_name := spec.table_name;
                partition_start := current_start;
                applied := p_apply;

                IF p_apply THEN
                    EXECUTE format(
                        'CREATE TABLE IF NOT EXISTS %I '
                        'PARTITION OF %I '
                        'FOR VALUES FROM (%L) TO (%L)',
                        child_name,
                        parent_name,
                        partition_start,
                        partition_end
                    );
                END IF;

                RETURN NEXT;
            END IF;

            current_start := partition_end;
        END LOOP;
    END LOOP;

    IF p_table_name IS NOT NULL AND NOT matched_spec THEN
        RAISE EXCEPTION 'Unknown partitioned table: %', p_table_name;
    END IF;
END;
$$;;

DO $migration$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM pg_proc proc
        JOIN pg_namespace namespace ON namespace.oid = proc.pronamespace
        WHERE namespace.nspname = 'cron'
          AND proc.proname = 'schedule'
    ) THEN
        BEGIN
            PERFORM cron.schedule(
                'ensure-time-partitions',
                '5 0 * * *',
                $cmd$SELECT ensure_time_partitions(NULL, timezone('UTC', clock_timestamp()), true)$cmd$
            );
        EXCEPTION
            WHEN OTHERS THEN
                BEGIN
                    PERFORM cron.unschedule('ensure-time-partitions');
                EXCEPTION
                    WHEN OTHERS THEN
                        NULL;
                END;

                PERFORM cron.schedule(
                    'ensure-time-partitions',
                    '5 0 * * *',
                    $cmd$SELECT ensure_time_partitions(NULL, timezone('UTC', clock_timestamp()), true)$cmd$
                );
        END;
    ELSE
        RAISE NOTICE
            'pg_cron is not available in %, skipping %',
            current_database(),
            'ensure-time-partitions';
    END IF;
END
$migration$;;

UPDATE alembic_version SET version_num='8e1f2a3b4c5d' WHERE alembic_version.version_num = '7c8d9e0f1a2b';

-- Running upgrade 8e1f2a3b4c5d, 7aa54c2ecbc9 -> c6d7e8f9a0b1

SET LOCAL statement_timeout = 0;

CREATE OR REPLACE FUNCTION messages_fts_trigger()
        RETURNS trigger AS $$
        BEGIN
            NEW.content_tsv := to_tsvector(
                'zhparser'::regconfig,
                COALESCE(NEW.content_text, '')
            );
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS messages_fts_update ON messages;

CREATE TRIGGER messages_fts_update
        BEFORE INSERT OR UPDATE ON messages
        FOR EACH ROW
        EXECUTE FUNCTION messages_fts_trigger();

UPDATE messages
        SET content_tsv = to_tsvector(
            'zhparser'::regconfig,
            COALESCE(content_text, '')
        )
        WHERE content_tsv IS NULL;

DELETE FROM alembic_version WHERE alembic_version.version_num = '8e1f2a3b4c5d';

UPDATE alembic_version SET version_num='c6d7e8f9a0b1' WHERE alembic_version.version_num = '7aa54c2ecbc9';

-- Running upgrade c6d7e8f9a0b1 -> ab4d99e19241

CREATE INDEX ix_futures_transaction_tx_type ON futures_transaction (tx_type);

UPDATE alembic_version SET version_num='ab4d99e19241' WHERE alembic_version.version_num = 'c6d7e8f9a0b1';

-- Running upgrade ab4d99e19241 -> 18d09d52d77a

ALTER TABLE economy_control_state DROP COLUMN risk_mode;

ALTER TABLE economy_snapshot DROP COLUMN risk_mode;

UPDATE alembic_version SET version_num='18d09d52d77a' WHERE alembic_version.version_num = 'ab4d99e19241';

-- Running upgrade 18d09d52d77a -> f4e8f3f7f0f2

ALTER TABLE economy_snapshot DROP CONSTRAINT IF EXISTS uq_economy_snapshot_window_time;

DROP INDEX IF EXISTS uq_economy_snapshot_window_time;

ALTER TABLE economy_snapshot ADD CONSTRAINT uq_economy_snapshot_captured_at UNIQUE (captured_at);

ALTER TABLE economy_control_state DROP COLUMN money_growth_soft_pct;

ALTER TABLE economy_control_state DROP COLUMN money_growth_hard_pct;

ALTER TABLE economy_control_state DROP COLUMN turnip_growth_soft_pct;

ALTER TABLE economy_control_state DROP COLUMN turnip_growth_hard_pct;

ALTER TABLE economy_snapshot DROP COLUMN window_hours;

ALTER TABLE economy_snapshot DROP COLUMN money_source;

ALTER TABLE economy_snapshot DROP COLUMN money_sink;

ALTER TABLE economy_snapshot DROP COLUMN turnip_source;

ALTER TABLE economy_snapshot DROP COLUMN turnip_sink;

ALTER TABLE economy_snapshot DROP COLUMN money_growth;

ALTER TABLE economy_snapshot DROP COLUMN turnip_growth;

UPDATE alembic_version SET version_num='f4e8f3f7f0f2' WHERE alembic_version.version_num = '18d09d52d77a';

-- Running upgrade f4e8f3f7f0f2 -> 0d3c0e4a7b6f

ALTER TABLE user_item ADD COLUMN display_name VARCHAR(200);

UPDATE alembic_version SET version_num='0d3c0e4a7b6f' WHERE alembic_version.version_num = 'f4e8f3f7f0f2';

-- Running upgrade 0d3c0e4a7b6f -> d0f8c4e03007

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_currency_exchange_order_created_at ON currency_exchange_order (created_at);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_currency_exchange_order_currency ON currency_exchange_order (currency);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_currency_exchange_order_expires_at ON currency_exchange_order (expires_at);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_currency_exchange_order_idempotency_key ON currency_exchange_order (idempotency_key);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_currency_exchange_order_status ON currency_exchange_order (status);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_currency_exchange_order_user_id ON currency_exchange_order (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_economy_snapshot_captured_at ON economy_snapshot (captured_at);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_futures_position_status ON futures_position (status);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_futures_position_user_id ON futures_position (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_futures_price_snapshot_created_at ON futures_price_snapshot (created_at);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_futures_transaction_created_at ON futures_transaction (created_at);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_futures_transaction_position_id ON futures_transaction (position_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_futures_transaction_user_id ON futures_transaction (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_game_currency_account_currency ON game_currency_account (currency);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_game_currency_account_user_id ON game_currency_account (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_game_currency_transaction_created_at ON game_currency_transaction (created_at);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_game_currency_transaction_currency ON game_currency_transaction (currency);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_game_currency_transaction_order_id ON game_currency_transaction (order_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_game_currency_transaction_reference_id ON game_currency_transaction (reference_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_game_currency_transaction_user_id ON game_currency_transaction (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_game_escrow_game_id ON game_escrow (game_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_land_user_id ON land (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_land_user_type ON land (user_id, land_type);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE UNIQUE INDEX ix_land_assignment_pal_active ON land_assignment (pal_id) WHERE released_at IS NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_land_assignment_pal_id ON land_assignment (pal_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_land_assignment_type ON land_assignment (user_id, assignment_type);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_land_assignment_user_id ON land_assignment (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_land_assignment_user_active ON land_assignment (user_id, released_at);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_pal_user_id ON pal (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_pal_user_species ON pal (user_id, species_code);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_pal_egg_user_id ON pal_egg (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_pal_egg_user_status ON pal_egg (user_id, status);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_raid_session_user_id ON raid_session (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_red_envelope_message_id ON red_envelope (message_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_red_envelope_sender_id ON red_envelope (sender_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_red_envelope_claim_envelope_id ON red_envelope_claim (envelope_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_red_envelope_claim_user_id ON red_envelope_claim (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_resource_production_paused ON resource_production (is_paused);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE UNIQUE INDEX ix_resource_production_user_land ON resource_production (user_id, land_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_stock_portfolio_symbol ON stock_portfolio (symbol);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_stock_portfolio_user_id ON stock_portfolio (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_stock_trade_history_symbol ON stock_trade_history (symbol);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_stock_trade_history_user_id ON stock_trade_history (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_stock_trigger_symbol ON stock_trigger (symbol);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_stock_trigger_user_id ON stock_trigger (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_turnip_inventory_locked_for_order_id ON turnip_inventory (locked_for_order_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_turnip_inventory_user_id ON turnip_inventory (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_turnip_order_user_id ON turnip_order (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_turnip_order_fill_order_id ON turnip_order_fill (order_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_turnip_seed_status_matures ON turnip_seed (status, matures_at);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_turnip_seed_user_status ON turnip_seed (user_id, status);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_turnip_transaction_created_at ON turnip_transaction (created_at);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_turnip_transaction_user_id ON turnip_transaction (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_user_item_item_type ON user_item (item_type);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_user_item_user_id ON user_item (user_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_wallet_transaction_counterparty_id ON wallet_transaction (counterparty_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_wallet_transaction_tx_group_id ON wallet_transaction (tx_group_id);

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

SELECT 1
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'i'
              AND c.relname = NULL;

CREATE INDEX ix_wallet_transaction_user_id ON wallet_transaction (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            WHERE n.nspname = 'public'
              AND t.relname = NULL
              AND c.conname = NULL;

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            WHERE n.nspname = 'public'
              AND t.relname = NULL
              AND c.conname = NULL;

ALTER TABLE game_currency_account ADD CONSTRAINT uq_game_currency_account_user_currency UNIQUE (user_id, currency);

DELETE FROM land_assignment la
            WHERE NOT EXISTS (
                SELECT 1
                FROM pal p
                WHERE p.id = la.pal_id
            );

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE land ADD CONSTRAINT fk_land_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE land_assignment ADD CONSTRAINT fk_land_assignment_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE land_assignment ADD CONSTRAINT fk_land_assignment_pal_id_pal_id FOREIGN KEY(pal_id) REFERENCES pal (id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE pal ADD CONSTRAINT fk_pal_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE pal_egg ADD CONSTRAINT fk_pal_egg_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE red_envelope_claim ADD CONSTRAINT fk_red_envelope_claim_envelope_id_red_envelope_id FOREIGN KEY(envelope_id) REFERENCES red_envelope (id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE resource_production ADD CONSTRAINT fk_resource_production_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE resource_production ADD CONSTRAINT fk_resource_production_land_id_land_id FOREIGN KEY(land_id) REFERENCES land (id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE turnip_inventory ADD CONSTRAINT fk_turnip_inventory_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE turnip_inventory ADD CONSTRAINT fk_turnip_inventory_locked_for_order_id_turnip_order_id FOREIGN KEY(locked_for_order_id) REFERENCES turnip_order (id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE turnip_order ADD CONSTRAINT fk_turnip_order_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE turnip_order_fill ADD CONSTRAINT fk_turnip_order_fill_order_id_turnip_order_id FOREIGN KEY(order_id) REFERENCES turnip_order (id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE turnip_seed ADD CONSTRAINT fk_turnip_seed_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE turnip_transaction ADD CONSTRAINT fk_turnip_transaction_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE user_item ADD CONSTRAINT fk_user_item_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

SELECT 1
            FROM pg_constraint c
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_class rt ON rt.oid = c.confrelid
            WHERE n.nspname = 'public'
              AND c.contype = 'f'
              AND t.relname = NULL
              AND rt.relname = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.conkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = t.oid
                 AND att.attnum = cols.attnum
              ) = NULL
              AND (
                SELECT string_agg(att.attname, ',' ORDER BY cols.ord)
                FROM unnest(c.confkey) WITH ORDINALITY AS cols(attnum, ord)
                JOIN pg_attribute att
                  ON att.attrelid = rt.oid
                 AND att.attnum = cols.attnum
              ) = NULL;

ALTER TABLE wallet_transaction ADD CONSTRAINT fk_wallet_transaction_user_id_wallet_user_id FOREIGN KEY(user_id) REFERENCES wallet (user_id);

UPDATE alembic_version SET version_num='d0f8c4e03007' WHERE alembic_version.version_num = '0d3c0e4a7b6f';

-- Running upgrade d0f8c4e03007 -> 2ffecde0b9c1

ALTER TABLE game_escrow ALTER COLUMN locked_funds TYPE JSONB USING locked_funds::jsonb;

ALTER TABLE game_escrow ALTER COLUMN settled_payouts TYPE JSONB USING settled_payouts::jsonb;

UPDATE alembic_version SET version_num='2ffecde0b9c1' WHERE alembic_version.version_num = 'd0f8c4e03007';

-- Running upgrade 0d3c0e4a7b6f -> 22c09ead444f

CREATE TABLE turnip_market_snapshot (
    id SERIAL NOT NULL, 
    last_trade_price DECIMAL(24, 2) NOT NULL, 
    ema_fair_value DECIMAL(24, 2) NOT NULL, 
    best_bid DECIMAL(24, 2), 
    best_ask DECIMAL(24, 2), 
    bid_depth VARCHAR(4096) NOT NULL, 
    ask_depth VARCHAR(4096) NOT NULL, 
    spread DECIMAL(24, 2) NOT NULL, 
    treasury_cash_balance DECIMAL(24, 2) NOT NULL, 
    treasury_turnip_balance NUMERIC(38, 0) NOT NULL, 
    tick_number INTEGER NOT NULL, 
    sub_tick_number INTEGER NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX idx_turnip_market_snapshot_created_at ON turnip_market_snapshot (created_at);

ALTER TABLE turnip_inventory ALTER COLUMN expires_at DROP NOT NULL;

INSERT INTO alembic_version (version_num) VALUES ('22c09ead444f') RETURNING alembic_version.version_num;

-- Running upgrade 22c09ead444f, 2ffecde0b9c1 -> 7a3c1c9b5e2f

ALTER TABLE turnip_inventory ADD COLUMN market_locked_for_order_id INTEGER;

CREATE INDEX ix_turnip_inventory_market_locked_for_order_id ON turnip_inventory (market_locked_for_order_id);

ALTER TABLE turnip_inventory ADD CONSTRAINT fk_turnip_inventory_market_locked_for_order_id_market_order FOREIGN KEY(market_locked_for_order_id) REFERENCES market_order (id);

DELETE FROM alembic_version WHERE alembic_version.version_num = '22c09ead444f';

UPDATE alembic_version SET version_num='7a3c1c9b5e2f' WHERE alembic_version.version_num = '2ffecde0b9c1';

-- Running upgrade 7a3c1c9b5e2f -> 6cc4fff4a9d6

ALTER TABLE currency_exchange_order ALTER COLUMN fee_amount TYPE DECIMAL(38, 8);

ALTER TABLE currency_exchange_order ALTER COLUMN game_delta TYPE DECIMAL(38, 6);

ALTER TABLE currency_exchange_order ALTER COLUMN game_inflation TYPE DECIMAL(38, 8);

ALTER TABLE currency_exchange_order ALTER COLUMN input_amount TYPE DECIMAL(38, 8);

ALTER TABLE currency_exchange_order ALTER COLUMN output_amount TYPE DECIMAL(38, 8);

ALTER TABLE currency_exchange_order ALTER COLUMN rate TYPE DECIMAL(38, 8);

ALTER TABLE currency_exchange_order ALTER COLUMN wallet_delta TYPE DECIMAL(38, 2);

ALTER TABLE currency_exchange_order ALTER COLUMN wallet_inflation TYPE DECIMAL(38, 8);

ALTER TABLE currency_rate_state ALTER COLUMN baseline_game_supply TYPE DECIMAL(38, 6);

ALTER TABLE currency_rate_state ALTER COLUMN baseline_wallet_supply TYPE DECIMAL(38, 2);

ALTER TABLE currency_rate_state ALTER COLUMN current_rate TYPE DECIMAL(38, 8);

ALTER TABLE currency_rate_state ALTER COLUMN last_game_supply TYPE DECIMAL(38, 6);

ALTER TABLE currency_rate_state ALTER COLUMN last_wallet_supply TYPE DECIMAL(38, 2);

ALTER TABLE economy_control_state ALTER COLUMN bank_liability TYPE DECIMAL(38, 2);

ALTER TABLE economy_control_state ALTER COLUMN coverage_ema TYPE DECIMAL(38, 8);

ALTER TABLE economy_control_state ALTER COLUMN coverage_ratio TYPE DECIMAL(38, 8);

ALTER TABLE economy_control_state ALTER COLUMN futures_payout_bucket_level TYPE DECIMAL(38, 2);

ALTER TABLE economy_control_state ALTER COLUMN futures_payout_bucket_max TYPE DECIMAL(38, 2);

ALTER TABLE economy_control_state ALTER COLUMN futures_payout_bucket_refill_rate TYPE DECIMAL(38, 2);

ALTER TABLE economy_control_state ALTER COLUMN money_bucket_level TYPE DECIMAL(38, 2);

ALTER TABLE economy_control_state ALTER COLUMN money_bucket_max TYPE DECIMAL(38, 2);

ALTER TABLE economy_control_state ALTER COLUMN money_bucket_refill_rate TYPE DECIMAL(38, 2);

ALTER TABLE economy_control_state ALTER COLUMN seed_weight TYPE DECIMAL(38, 6);

ALTER TABLE economy_control_state ALTER COLUMN turnip_bucket_level TYPE DECIMAL(38, 2);

ALTER TABLE economy_control_state ALTER COLUMN turnip_bucket_max TYPE DECIMAL(38, 2);

ALTER TABLE economy_control_state ALTER COLUMN turnip_bucket_refill_rate TYPE DECIMAL(38, 2);

ALTER TABLE economy_snapshot ALTER COLUMN bank_liability TYPE DECIMAL(38, 2);

ALTER TABLE economy_snapshot ALTER COLUMN coverage_ratio TYPE DECIMAL(38, 8);

ALTER TABLE economy_snapshot ALTER COLUMN effective_supply TYPE DECIMAL(38, 2);

ALTER TABLE economy_snapshot ALTER COLUMN liquidity TYPE DECIMAL(38, 2);

ALTER TABLE futures_position ALTER COLUMN entry_price TYPE DECIMAL(38, 2);

ALTER TABLE futures_position ALTER COLUMN liquidation_price TYPE DECIMAL(38, 2);

ALTER TABLE futures_position ALTER COLUMN margin TYPE DECIMAL(38, 2);

ALTER TABLE futures_position ALTER COLUMN realized_pnl TYPE DECIMAL(38, 2);

ALTER TABLE futures_position ALTER COLUMN stop_loss_price TYPE DECIMAL(38, 2);

ALTER TABLE futures_position ALTER COLUMN take_profit_price TYPE DECIMAL(38, 2);

ALTER TABLE futures_price_snapshot ALTER COLUMN funding_rate TYPE DECIMAL(38, 8);

ALTER TABLE futures_price_snapshot ALTER COLUMN futures_price TYPE DECIMAL(38, 2);

ALTER TABLE futures_price_snapshot ALTER COLUMN mid_price TYPE DECIMAL(38, 2);

ALTER TABLE futures_price_snapshot ALTER COLUMN spot_price TYPE DECIMAL(38, 2);

ALTER TABLE futures_transaction ALTER COLUMN fee TYPE DECIMAL(38, 2);

ALTER TABLE futures_transaction ALTER COLUMN margin_change TYPE DECIMAL(38, 2);

ALTER TABLE futures_transaction ALTER COLUMN paid_pnl TYPE DECIMAL(38, 2);

ALTER TABLE futures_transaction ALTER COLUMN pnl TYPE DECIMAL(38, 2);

ALTER TABLE futures_transaction ALTER COLUMN price TYPE DECIMAL(38, 2);

ALTER TABLE futures_transaction ALTER COLUMN raw_pnl TYPE DECIMAL(38, 2);

ALTER TABLE futures_transaction ALTER COLUMN spot_price TYPE DECIMAL(38, 2);

ALTER TABLE futures_transaction ALTER COLUMN unpaid_pnl TYPE DECIMAL(38, 2);

ALTER TABLE game_currency_account ALTER COLUMN balance TYPE DECIMAL(38, 6);

ALTER TABLE game_currency_transaction ALTER COLUMN amount TYPE DECIMAL(38, 6);

ALTER TABLE game_currency_transaction ALTER COLUMN balance_after TYPE DECIMAL(38, 6);

ALTER TABLE market_maker_state ALTER COLUMN cached_effective_depth TYPE DECIMAL(38, 2);

ALTER TABLE market_maker_state ALTER COLUMN cached_market_center TYPE DECIMAL(38, 2);

ALTER TABLE market_maker_state ALTER COLUMN fair_value TYPE DECIMAL(38, 2);

ALTER TABLE market_maker_state ALTER COLUMN futures_anchor_spot TYPE DECIMAL(38, 2);

ALTER TABLE market_maker_state ALTER COLUMN futures_insurance_fund TYPE DECIMAL(38, 2);

ALTER TABLE market_maker_state ALTER COLUMN mid_price TYPE DECIMAL(38, 2);

ALTER TABLE market_order ALTER COLUMN escrow_amount TYPE DECIMAL(38, 2);

ALTER TABLE market_order ALTER COLUMN price TYPE DECIMAL(38, 2);

ALTER TABLE market_order_fill ALTER COLUMN fee TYPE DECIMAL(38, 2);

ALTER TABLE market_order_fill ALTER COLUMN price TYPE DECIMAL(38, 2);

ALTER TABLE market_order_fill ALTER COLUMN total TYPE DECIMAL(38, 2);

ALTER TABLE pal_egg ALTER COLUMN price_paid TYPE DECIMAL(38, 2);

ALTER TABLE red_envelope ALTER COLUMN remaining_amount TYPE DECIMAL(38, 2);

ALTER TABLE red_envelope ALTER COLUMN total_amount TYPE DECIMAL(38, 2);

ALTER TABLE red_envelope_claim ALTER COLUMN amount TYPE DECIMAL(38, 2);

ALTER TABLE resource_production ALTER COLUMN accumulated_credits TYPE DECIMAL(38, 2);

ALTER TABLE season_settlement ALTER COLUMN starting_balance TYPE DECIMAL(38, 2);

ALTER TABLE stock_account ALTER COLUMN best_trade_pnl TYPE DECIMAL(38, 2);

ALTER TABLE stock_account ALTER COLUMN total_realized_pnl TYPE DECIMAL(38, 2);

ALTER TABLE stock_portfolio ALTER COLUMN buy_price TYPE DECIMAL(38, 2);

ALTER TABLE stock_portfolio ALTER COLUMN shares TYPE DECIMAL(38, 6);

ALTER TABLE stock_trade_history ALTER COLUMN pnl TYPE DECIMAL(38, 2);

ALTER TABLE stock_trade_history ALTER COLUMN price TYPE DECIMAL(38, 2);

ALTER TABLE stock_trade_history ALTER COLUMN shares TYPE DECIMAL(38, 6);

ALTER TABLE stock_trigger ALTER COLUMN shares TYPE DECIMAL(38, 6);

ALTER TABLE stock_trigger ALTER COLUMN trigger_price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_inventory ALTER COLUMN buy_price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_inventory ALTER COLUMN stored_shelf_life_seconds TYPE DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ALTER COLUMN best_ask TYPE DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ALTER COLUMN best_bid TYPE DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ALTER COLUMN ema_fair_value TYPE DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ALTER COLUMN last_trade_price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ALTER COLUMN spread TYPE DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ALTER COLUMN treasury_cash_balance TYPE DECIMAL(38, 2);

ALTER TABLE turnip_order ALTER COLUMN escrow_amount TYPE DECIMAL(38, 2);

ALTER TABLE turnip_order ALTER COLUMN execution_price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_order ALTER COLUMN limit_price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_order ALTER COLUMN quote_price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_order_fill ALTER COLUMN price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_order_fill ALTER COLUMN total TYPE DECIMAL(38, 2);

ALTER TABLE turnip_price ALTER COLUMN base_price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_price ALTER COLUMN high TYPE DECIMAL(38, 2);

ALTER TABLE turnip_price ALTER COLUMN low TYPE DECIMAL(38, 2);

ALTER TABLE turnip_price ALTER COLUMN open TYPE DECIMAL(38, 2);

ALTER TABLE turnip_price ALTER COLUMN price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_seed ALTER COLUMN fertilize_cost TYPE DECIMAL(38, 2);

ALTER TABLE turnip_seed ALTER COLUMN seed_price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_transaction ALTER COLUMN mid_price TYPE DECIMAL(38, 2);

ALTER TABLE turnip_transaction ALTER COLUMN unit_price TYPE DECIMAL(38, 2);

ALTER TABLE user_item ALTER COLUMN price_paid TYPE DECIMAL(38, 2);

ALTER TABLE wallet ALTER COLUMN balance TYPE DECIMAL(38, 2);

ALTER TABLE wallet ALTER COLUMN escrow_balance TYPE DECIMAL(38, 2);

ALTER TABLE wallet ALTER COLUMN total_credited TYPE DECIMAL(38, 2);

ALTER TABLE wallet_transaction ALTER COLUMN amount TYPE DECIMAL(38, 2);

ALTER TABLE wallet_transaction ALTER COLUMN balance_after TYPE DECIMAL(38, 2);

ALTER TABLE wallet_transaction ALTER COLUMN escrow_after TYPE DECIMAL(38, 2);

UPDATE alembic_version SET version_num='6cc4fff4a9d6' WHERE alembic_version.version_num = '7a3c1c9b5e2f';

-- Running upgrade 22c09ead444f, 2ffecde0b9c1 -> 9c3e402acdbd

INSERT INTO alembic_version (version_num) VALUES ('9c3e402acdbd') RETURNING alembic_version.version_num;

-- Running upgrade 9c3e402acdbd -> 2df0744bd860

CREATE TRIGGER turnip_market_snapshot_tick_trigger
        AFTER INSERT ON turnip_market_snapshot
        FOR EACH ROW
        EXECUTE FUNCTION notify_turnip_tick();;

UPDATE alembic_version SET version_num='2df0744bd860' WHERE alembic_version.version_num = '9c3e402acdbd';

-- Running upgrade 2df0744bd860 -> 6a3d0b7ac2f1

ALTER TABLE market_maker_state ALTER COLUMN inventory TYPE NUMERIC(38, 0);

ALTER TABLE market_maker_state ALTER COLUMN user_pressure TYPE NUMERIC(38, 0);

UPDATE alembic_version SET version_num='6a3d0b7ac2f1' WHERE alembic_version.version_num = '2df0744bd860';

-- Running upgrade 6a3d0b7ac2f1, 6cc4fff4a9d6 -> 9e52b6d4c1af

ALTER TABLE futures_position ALTER COLUMN quantity TYPE NUMERIC(38, 0);

ALTER TABLE futures_price_snapshot ALTER COLUMN open_interest TYPE NUMERIC(38, 0);

ALTER TABLE futures_price_snapshot ALTER COLUMN total_long TYPE NUMERIC(38, 0);

ALTER TABLE futures_price_snapshot ALTER COLUMN total_short TYPE NUMERIC(38, 0);

ALTER TABLE futures_transaction ALTER COLUMN quantity TYPE NUMERIC(38, 0);

ALTER TABLE market_maker_state ALTER COLUMN futures_open_interest TYPE NUMERIC(38, 0);

ALTER TABLE market_maker_state ALTER COLUMN futures_total_short TYPE NUMERIC(38, 0);

ALTER TABLE turnip_order_fill ALTER COLUMN quantity TYPE NUMERIC(38, 0);

ALTER TABLE turnip_order ALTER COLUMN quantity TYPE NUMERIC(38, 0);

ALTER TABLE turnip_order ALTER COLUMN filled_quantity TYPE NUMERIC(38, 0);

ALTER TABLE turnip_seed ALTER COLUMN quantity TYPE NUMERIC(38, 0);

DELETE FROM alembic_version WHERE alembic_version.version_num = '6a3d0b7ac2f1';

UPDATE alembic_version SET version_num='9e52b6d4c1af' WHERE alembic_version.version_num = '6cc4fff4a9d6';

-- Running upgrade 9e52b6d4c1af -> 841c10d868e9

ALTER TABLE economy_control_state ADD COLUMN cash_injection_multiplier NUMERIC(38, 6) DEFAULT '1.000000' NOT NULL;

UPDATE alembic_version SET version_num='841c10d868e9' WHERE alembic_version.version_num = '9e52b6d4c1af';

-- Running upgrade 841c10d868e9 -> 072842120515

CREATE TABLE turnip_trade (
    id SERIAL NOT NULL, 
    side VARCHAR(10) NOT NULL, 
    quantity NUMERIC(38, 0) NOT NULL, 
    price NUMERIC(38, 2) NOT NULL, 
    total NUMERIC(38, 2) NOT NULL, 
    maker_actor VARCHAR(100) NOT NULL, 
    taker_actor VARCHAR(100) NOT NULL, 
    maker_order_id INTEGER, 
    taker_order_id INTEGER, 
    trade_type VARCHAR(16) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(maker_order_id) REFERENCES turnip_order (id), 
    FOREIGN KEY(taker_order_id) REFERENCES turnip_order (id)
);

CREATE INDEX ix_turnip_trade_created_at ON turnip_trade (created_at);

CREATE INDEX ix_turnip_trade_maker_order_id ON turnip_trade (maker_order_id);

CREATE INDEX ix_turnip_trade_taker_order_id ON turnip_trade (taker_order_id);

UPDATE alembic_version SET version_num='072842120515' WHERE alembic_version.version_num = '841c10d868e9';

-- Running upgrade 072842120515 -> ae29f903f26e

ALTER TABLE turnip_market_snapshot ADD COLUMN treasury_imbalance DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN policy_sell_rate DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN policy_buy_rate DECIMAL(38, 6);

UPDATE alembic_version SET version_num='ae29f903f26e' WHERE alembic_version.version_num = '072842120515';

-- Running upgrade ae29f903f26e -> a839210f2f26

ALTER TABLE economy_control_state ADD COLUMN treasury_policy_enabled BOOLEAN DEFAULT true NOT NULL;

ALTER TABLE economy_control_state ADD COLUMN treasury_policy_k DECIMAL(38, 6);

ALTER TABLE economy_control_state ADD COLUMN treasury_policy_beta DECIMAL(38, 6);

UPDATE alembic_version SET version_num='a839210f2f26' WHERE alembic_version.version_num = 'ae29f903f26e';

-- Running upgrade 072842120515 -> 5a1f9f3a4b2c

ALTER TABLE economy_control_state DROP COLUMN turnip_bucket_refill_rate;

ALTER TABLE economy_control_state DROP COLUMN turnip_bucket_max;

ALTER TABLE economy_control_state DROP COLUMN turnip_bucket_level;

ALTER TABLE economy_control_state DROP COLUMN money_bucket_refill_rate;

ALTER TABLE economy_control_state DROP COLUMN money_bucket_max;

ALTER TABLE economy_control_state DROP COLUMN money_bucket_level;

INSERT INTO alembic_version (version_num) VALUES ('5a1f9f3a4b2c') RETURNING alembic_version.version_num;

-- Running upgrade 5a1f9f3a4b2c, a839210f2f26 -> dcaf1247a088

DELETE FROM alembic_version WHERE alembic_version.version_num = '5a1f9f3a4b2c';

UPDATE alembic_version SET version_num='dcaf1247a088' WHERE alembic_version.version_num = 'a839210f2f26';

-- Running upgrade dcaf1247a088 -> 39f5b6090f2c

CREATE TABLE futures_order (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    side VARCHAR(10) NOT NULL, 
    order_type VARCHAR(10) NOT NULL, 
    intent VARCHAR(10) NOT NULL, 
    close_position_id INTEGER, 
    price NUMERIC(38, 2), 
    quantity NUMERIC(38, 0) NOT NULL, 
    filled_quantity NUMERIC(38, 0) NOT NULL, 
    margin_frozen NUMERIC(38, 2) NOT NULL, 
    status VARCHAR(20) NOT NULL, 
    cancel_requested BOOLEAN NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(close_position_id) REFERENCES futures_position (id)
);

CREATE INDEX ix_futures_order_status ON futures_order (status);

CREATE INDEX ix_futures_order_user_id ON futures_order (user_id);

CREATE TABLE futures_order_fill (
    id SERIAL NOT NULL, 
    buy_order_id INTEGER, 
    sell_order_id INTEGER, 
    price NUMERIC(38, 2) NOT NULL, 
    quantity NUMERIC(38, 0) NOT NULL, 
    buyer_id VARCHAR NOT NULL, 
    seller_id VARCHAR NOT NULL, 
    settled BOOLEAN NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(buy_order_id) REFERENCES futures_order (id), 
    FOREIGN KEY(sell_order_id) REFERENCES futures_order (id)
);

CREATE INDEX ix_futures_order_fill_buy_order_id ON futures_order_fill (buy_order_id);

CREATE INDEX ix_futures_order_fill_sell_order_id ON futures_order_fill (sell_order_id);

UPDATE alembic_version SET version_num='39f5b6090f2c' WHERE alembic_version.version_num = 'dcaf1247a088';

-- Running upgrade 39f5b6090f2c -> f5208ffc8f96

ALTER TABLE futures_position ADD COLUMN pending_close_quantity NUMERIC(38, 0) DEFAULT '0' NOT NULL;

CREATE UNIQUE INDEX uq_futures_position_user_side_open ON futures_position (user_id, side) WHERE status = 'OPEN';

UPDATE alembic_version SET version_num='f5208ffc8f96' WHERE alembic_version.version_num = '39f5b6090f2c';

-- Running upgrade 5a1f9f3a4b2c, a839210f2f26 -> b3c4d5e6f7a8

CREATE TABLE sudoku_puzzle (
    id SERIAL NOT NULL, 
    room_id VARCHAR(255) NOT NULL, 
    creator_id VARCHAR(255) NOT NULL, 
    request_message_id VARCHAR(255) NOT NULL, 
    announcement_message_id VARCHAR(255), 
    difficulty VARCHAR(32) NOT NULL, 
    puzzle VARCHAR(81) NOT NULL, 
    solution VARCHAR(81) NOT NULL, 
    status VARCHAR(32) NOT NULL, 
    solver_id VARCHAR(255), 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    finished_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_sudoku_puzzle_room_id ON sudoku_puzzle (room_id);

CREATE INDEX ix_sudoku_puzzle_creator_id ON sudoku_puzzle (creator_id);

CREATE INDEX ix_sudoku_puzzle_request_message_id ON sudoku_puzzle (request_message_id);

CREATE INDEX ix_sudoku_puzzle_announcement_message_id ON sudoku_puzzle (announcement_message_id);

CREATE INDEX ix_sudoku_puzzle_status ON sudoku_puzzle (status);

INSERT INTO alembic_version (version_num) VALUES ('b3c4d5e6f7a8') RETURNING alembic_version.version_num;

-- Running upgrade 5a1f9f3a4b2c, a839210f2f26 -> 4e1f6a9b2c3d

CREATE TABLE raid_warehouse_item (
    item_id VARCHAR(128) NOT NULL, 
    user_id VARCHAR(64) NOT NULL, 
    template_id VARCHAR(64) NOT NULL, 
    item_type VARCHAR(32) NOT NULL, 
    quality VARCHAR(32) NOT NULL, 
    quantity INTEGER NOT NULL, 
    item_data JSONB NOT NULL, 
    market_locked_for_order_id BIGINT, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (user_id, item_id)
);

CREATE INDEX ix_raid_warehouse_item_user_id ON raid_warehouse_item (user_id);

CREATE INDEX ix_raid_warehouse_item_user_locked ON raid_warehouse_item (user_id, market_locked_for_order_id);

CREATE INDEX ix_raid_warehouse_item_user_template_quality_locked ON raid_warehouse_item (user_id, template_id, quality, market_locked_for_order_id);

DO $$
        BEGIN
            IF EXISTS (
                SELECT user_id, item_id
                FROM (
                    SELECT profile.user_id, item->>'id' AS item_id
                    FROM raid_profile AS profile
                    CROSS JOIN LATERAL jsonb_array_elements(
                        coalesce(profile.warehouse->'items', '[]'::jsonb)
                    ) AS item
                    WHERE profile.warehouse IS NOT NULL
                      AND profile.warehouse->'items' IS NOT NULL
                ) AS extracted_items
                GROUP BY user_id, item_id
                HAVING count(*) > 1
            ) THEN
                RAISE EXCEPTION
                    'same-user duplicate raid warehouse item_id found in legacy raid_profile warehouse payload';
            END IF;
        END
        $$;;

INSERT INTO raid_warehouse_item (
            item_id,
            user_id,
            template_id,
            item_type,
            quality,
            quantity,
            item_data,
            market_locked_for_order_id,
            created_at,
            updated_at
        )
        SELECT
            item->>'id' AS item_id,
            profile.user_id,
            split_part(item->>'id', '@', 1) AS template_id,
            coalesce(item->>'item_type', 'loot') AS item_type,
            coalesce(item->>'quality', 'common') AS quality,
            coalesce(nullif(item->>'quantity', '')::integer, 1) AS quantity,
            item - 'id' - 'quantity' AS item_data,
            NULL AS market_locked_for_order_id,
            profile.created_at,
            profile.updated_at
        FROM raid_profile AS profile
        CROSS JOIN LATERAL jsonb_array_elements(
            coalesce(profile.warehouse->'items', '[]'::jsonb)
        ) AS item
        WHERE profile.warehouse IS NOT NULL
          AND profile.warehouse->'items' IS NOT NULL;

ALTER TABLE raid_profile DROP COLUMN warehouse;

INSERT INTO alembic_version (version_num) VALUES ('4e1f6a9b2c3d') RETURNING alembic_version.version_num;

-- Running upgrade 4e1f6a9b2c3d -> 5c9e2a41b7d0

ALTER TABLE raid_warehouse_item ADD COLUMN loadout_slot VARCHAR(16);

ALTER TABLE raid_warehouse_item ADD COLUMN loadout_order INTEGER;

CREATE INDEX ix_raid_warehouse_item_user_loadout ON raid_warehouse_item (user_id, loadout_slot, loadout_order);

CREATE UNIQUE INDEX ux_raid_warehouse_item_user_loadout_equipment_slot ON raid_warehouse_item (user_id, loadout_slot) WHERE loadout_slot IN ('weapon', 'armor', 'accessory', 'backpack');

ALTER TABLE raid_warehouse_item ADD CONSTRAINT ck_raid_warehouse_item_loadout_pairing CHECK ((loadout_slot IS NULL) = (loadout_order IS NULL));

WITH legacy_loadout AS (
            SELECT
                profile.user_id,
                loadout_item.item_id,
                loadout_item.ordinality - 1 AS legacy_order
            FROM raid_profile AS profile
            CROSS JOIN LATERAL jsonb_array_elements_text(profile.loadout->'item_ids')
                WITH ORDINALITY AS loadout_item(item_id, ordinality)
            WHERE profile.loadout IS NOT NULL
              AND profile.loadout->'item_ids' IS NOT NULL
              AND jsonb_array_length(profile.loadout->'item_ids') > 0
        ),
        loadout_candidates AS (
            SELECT
                legacy_loadout.user_id,
                legacy_loadout.item_id,
                legacy_loadout.legacy_order,
                CASE
                    WHEN warehouse.item_type IN (
                        'weapon', 'armor', 'accessory', 'backpack'
                    ) THEN warehouse.item_type
                    ELSE 'supply'
                END AS resolved_slot
            FROM legacy_loadout
            JOIN raid_warehouse_item AS warehouse
              ON warehouse.user_id = legacy_loadout.user_id
             AND warehouse.item_id = legacy_loadout.item_id
        ),
        ranked_candidates AS (
            SELECT
                user_id,
                item_id,
                legacy_order,
                resolved_slot,
                CASE
                    WHEN resolved_slot IN ('weapon', 'armor', 'accessory', 'backpack')
                    THEN row_number() OVER (
                        PARTITION BY user_id, resolved_slot
                        ORDER BY legacy_order, item_id
                    )
                    ELSE 1
                END AS candidate_rank
            FROM loadout_candidates
        ),
        normalized_loadout AS (
            SELECT
                user_id,
                item_id,
                resolved_slot,
                CASE
                    WHEN resolved_slot = 'supply'
                    THEN row_number() OVER (
                        PARTITION BY user_id, resolved_slot
                        ORDER BY legacy_order, item_id
                    ) - 1
                    ELSE 0
                END AS loadout_order
            FROM ranked_candidates
            WHERE resolved_slot = 'supply'
               OR candidate_rank = 1
        )
        UPDATE raid_warehouse_item AS warehouse
        SET loadout_slot = normalized_loadout.resolved_slot,
            loadout_order = normalized_loadout.loadout_order
        FROM normalized_loadout
        WHERE warehouse.user_id = normalized_loadout.user_id
          AND warehouse.item_id = normalized_loadout.item_id;

UPDATE alembic_version SET version_num='5c9e2a41b7d0' WHERE alembic_version.version_num = '4e1f6a9b2c3d';

-- Running upgrade 5c9e2a41b7d0 -> 9a7f0c1d2e3b

ALTER TABLE raid_profile DROP COLUMN loadout;

UPDATE alembic_version SET version_num='9a7f0c1d2e3b' WHERE alembic_version.version_num = '5c9e2a41b7d0';

-- Running upgrade 5a1f9f3a4b2c, a839210f2f26 -> d92e7fa1d76f

INSERT INTO alembic_version (version_num) VALUES ('d92e7fa1d76f') RETURNING alembic_version.version_num;

-- Running upgrade d92e7fa1d76f -> 1ff08cc8b6af

ALTER TABLE currency_exchange_order ALTER COLUMN expires_at DROP NOT NULL;

UPDATE alembic_version SET version_num='1ff08cc8b6af' WHERE alembic_version.version_num = 'd92e7fa1d76f';

-- Running upgrade 1ff08cc8b6af, 9a7f0c1d2e3b, b3c4d5e6f7a8, f5208ffc8f96 -> bb211bf7a8c6

DELETE FROM alembic_version WHERE alembic_version.version_num = '1ff08cc8b6af';

DELETE FROM alembic_version WHERE alembic_version.version_num = '9a7f0c1d2e3b';

DELETE FROM alembic_version WHERE alembic_version.version_num = 'b3c4d5e6f7a8';

UPDATE alembic_version SET version_num='bb211bf7a8c6' WHERE alembic_version.version_num = 'f5208ffc8f96';

-- Running upgrade bb211bf7a8c6 -> 6f5e2d1b2472

CREATE TABLE futures_engine_state (
    id SERIAL NOT NULL, 
    npc_net_position NUMERIC(38, 0) NOT NULL, 
    hedge_pending_delta NUMERIC(38, 0) NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

UPDATE alembic_version SET version_num='6f5e2d1b2472' WHERE alembic_version.version_num = 'bb211bf7a8c6';

-- Running upgrade 6f5e2d1b2472 -> 9a1d6f4b2c3e

CREATE OR REPLACE FUNCTION wallet_guard_validate_tx_group(p_tx_group_id text)
    RETURNS void AS $$
    DECLARE
        tx_count integer;
        group_amount numeric;
        group_escrow_delta numeric;
        self_user_id text;
        self_counterparty_id text;
        self_amount numeric;
        self_escrow_delta numeric;
    BEGIN
        IF p_tx_group_id IS NULL THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id may not be null'
                USING ERRCODE = '23514';
        END IF;

        SELECT
            COUNT(*),
            COALESCE(SUM(amount), 0),
            COALESCE(SUM(escrow_delta), 0)
        INTO tx_count, group_amount, group_escrow_delta
        FROM wallet_transaction
        WHERE tx_group_id = p_tx_group_id;

        IF tx_count = 1 THEN
            SELECT user_id, counterparty_id, amount, escrow_delta
            INTO self_user_id, self_counterparty_id, self_amount, self_escrow_delta
            FROM wallet_transaction
            WHERE tx_group_id = p_tx_group_id
            LIMIT 1;

            IF self_user_id IS DISTINCT FROM self_counterparty_id
               OR self_amount + self_escrow_delta <> 0 THEN
                RAISE EXCEPTION
                    'wallet ledger guard: tx_group_id % is not a valid self-group',
                    p_tx_group_id
                    USING ERRCODE = '23514';
            END IF;
        ELSIF tx_count > 2 THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id % must not have more than 2 rows, got %',
                p_tx_group_id,
                tx_count
                USING ERRCODE = '23514';
        ELSIF EXISTS (
            SELECT 1
            FROM wallet_transaction wt
            WHERE wt.tx_group_id = p_tx_group_id
              AND (
                  SELECT COUNT(*)
                  FROM wallet_transaction peer
                  WHERE peer.tx_group_id = wt.tx_group_id
                    AND peer.id <> wt.id
                    AND peer.user_id = wt.counterparty_id
                    AND peer.counterparty_id = wt.user_id
              ) <> 1
        ) THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id % has broken counterparty links',
                p_tx_group_id
                USING ERRCODE = '23514';
        END IF;

        IF group_amount + group_escrow_delta <> 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id % must balance amount+escrow_delta to zero, got %',
                p_tx_group_id,
                group_amount + group_escrow_delta
                USING ERRCODE = '23514';
        END IF;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_validate_wallet(p_user_id text)
    RETURNS void AS $$
    DECLARE
        ledger_balance numeric;
        ledger_escrow numeric;
        wallet_allow_negative boolean;
    BEGIN
        IF p_user_id IS NULL THEN
            RETURN;
        END IF;

        SELECT
            "allow_negative_balance"
        INTO
            wallet_allow_negative
        FROM "wallet"
        WHERE "user_id" = p_user_id;

        IF NOT FOUND THEN
            IF EXISTS (
                SELECT 1
                FROM "wallet_transaction"
                WHERE "user_id" = p_user_id
            ) THEN
                RAISE EXCEPTION
                    'wallet ledger guard: wallet % missing while transactions exist',
                    p_user_id
                    USING ERRCODE = '23514';
            END IF;
            RETURN;
        END IF;

        SELECT COALESCE(SUM("amount"), 0)
        INTO ledger_balance
        FROM "wallet_transaction"
        WHERE "user_id" = p_user_id;

        SELECT COALESCE(SUM("escrow_delta"), 0)
        INTO ledger_escrow
        FROM "wallet_transaction"
        WHERE "user_id" = p_user_id;

        -- Best-effort guard for append-only wallets: this rechecks committed
        -- rows visible to this transaction, but intentionally does not take a
        -- per-wallet serialization lock. Extreme concurrent spends may still
        -- commit into a negative ledger balance and are handled by follow-up
        -- balance reads / recovery flows.
        IF NOT wallet_allow_negative AND ledger_balance < 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: wallet % may not go negative (ledger=%)',
                p_user_id,
                ledger_balance
                USING ERRCODE = '23514';
        END IF;

        IF ledger_escrow < 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: wallet % may not have negative escrow (ledger=%)',
                p_user_id,
                ledger_escrow
                USING ERRCODE = '23514';
        END IF;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_enforce_tx_group()
    RETURNS trigger AS $$
    BEGIN
        IF TG_OP = 'DELETE' THEN
            PERFORM wallet_guard_validate_tx_group(OLD.tx_group_id);
            RETURN NULL;
        END IF;

        PERFORM wallet_guard_validate_tx_group(NEW.tx_group_id);
        IF TG_OP = 'UPDATE' AND OLD.tx_group_id IS DISTINCT FROM NEW.tx_group_id THEN
            PERFORM wallet_guard_validate_tx_group(OLD.tx_group_id);
        END IF;
        RETURN NULL;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_enforce_tx_wallet()
    RETURNS trigger AS $$
    BEGIN
        IF TG_OP = 'DELETE' THEN
            PERFORM wallet_guard_validate_wallet(OLD.user_id);
            RETURN NULL;
        END IF;

        PERFORM wallet_guard_validate_wallet(NEW.user_id);
        IF TG_OP = 'UPDATE' AND OLD.user_id IS DISTINCT FROM NEW.user_id THEN
            PERFORM wallet_guard_validate_wallet(OLD.user_id);
        END IF;
        RETURN NULL;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_enforce_wallet()
    RETURNS trigger AS $$
    BEGIN
        IF TG_OP = 'DELETE' THEN
            PERFORM wallet_guard_validate_wallet(OLD.user_id);
            RETURN NULL;
        END IF;

        PERFORM wallet_guard_validate_wallet(NEW.user_id);
        RETURN NULL;
    END;
    $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS wallet_tx_group_guard ON wallet_transaction;

CREATE CONSTRAINT TRIGGER wallet_tx_group_guard
    AFTER INSERT OR UPDATE OF user_id, amount, escrow_delta, counterparty_id, tx_group_id OR DELETE ON wallet_transaction
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION wallet_guard_enforce_tx_group();

DROP TRIGGER IF EXISTS wallet_tx_wallet_guard ON wallet_transaction;

CREATE CONSTRAINT TRIGGER wallet_tx_wallet_guard
    AFTER INSERT OR UPDATE OF user_id, amount, escrow_delta, counterparty_id, tx_group_id OR DELETE ON wallet_transaction
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION wallet_guard_enforce_tx_wallet();

DROP TRIGGER IF EXISTS wallet_row_wallet_guard ON wallet;

CREATE CONSTRAINT TRIGGER wallet_row_wallet_guard
    AFTER INSERT OR UPDATE OF user_id, allow_negative_balance OR DELETE ON wallet
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION wallet_guard_enforce_wallet();

UPDATE alembic_version SET version_num='9a1d6f4b2c3e' WHERE alembic_version.version_num = '6f5e2d1b2472';

-- Running upgrade 9a1d6f4b2c3e -> 9faefcf0d150

ALTER TABLE futures_price_snapshot ADD COLUMN open DECIMAL(38, 2);

ALTER TABLE futures_price_snapshot ADD COLUMN high DECIMAL(38, 2);

ALTER TABLE futures_price_snapshot ADD COLUMN low DECIMAL(38, 2);

ALTER TABLE futures_price_snapshot ADD COLUMN volume DECIMAL(38, 0);

ALTER TABLE futures_price_snapshot ADD COLUMN trade_count INTEGER;

UPDATE alembic_version SET version_num='9faefcf0d150' WHERE alembic_version.version_num = '9a1d6f4b2c3e';

-- Running upgrade 9faefcf0d150 -> a3c1d4e5f678

ALTER TABLE economy_control_state ADD COLUMN turnip_runtime_overrides JSONB;

UPDATE alembic_version SET version_num='a3c1d4e5f678' WHERE alembic_version.version_num = '9faefcf0d150';

-- Running upgrade a3c1d4e5f678 -> b7c2d9e4f1a0

ALTER TABLE raid_map_progress ADD COLUMN pressure INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_map_progress ADD COLUMN run_count INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE raid_map_progress ADD COLUMN last_started_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE raid_map_progress ADD COLUMN last_settled_at TIMESTAMP WITH TIME ZONE;

UPDATE alembic_version SET version_num='b7c2d9e4f1a0' WHERE alembic_version.version_num = 'a3c1d4e5f678';

-- Running upgrade b7c2d9e4f1a0 -> c4f6a1d2b9e7

ALTER TABLE land ADD COLUMN permit_item_id VARCHAR(128);

UPDATE alembic_version SET version_num='c4f6a1d2b9e7' WHERE alembic_version.version_num = 'b7c2d9e4f1a0';

-- Running upgrade c4f6a1d2b9e7 -> ecf4ab7f2781

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
);

CREATE INDEX ix_shared_kv_namespace ON shared_kv (namespace);

CREATE INDEX ix_shared_kv_expires ON shared_kv (expires_at);

INSERT INTO shared_kv (namespace, key, value, created_at, updated_at, expires_at)
            SELECT
                namespace,
                normalized_key,
                normalized_value,
                created_at,
                updated_at,
                expires_at
            FROM (
                SELECT DISTINCT ON (namespace, normalized_key)
                    namespace,
                    normalized_key,
                    normalized_value,
                    created_at,
                    updated_at,
                    expires_at
                FROM (
                    SELECT
                        namespace,
                        CASE
                            WHEN namespace IN ('default_talk_settings', 'greeting_settings')
                                THEN 'room:' || room_id
                            ELSE key
                        END AS normalized_key,
                        CASE
                            WHEN namespace IN ('default_talk_settings', 'greeting_settings')
                                 AND value::text = '"true"'
                                THEN 'true'::jsonb
                            WHEN namespace IN ('default_talk_settings', 'greeting_settings')
                                 AND value::text = '"false"'
                                THEN 'false'::jsonb
                            ELSE value
                        END AS normalized_value,
                        created_at,
                        updated_at,
                        expires_at,
                        id
                    FROM bot_memory
                    WHERE namespace IN (
                        'room_blacklist',
                        'default_talk_settings',
                        'greeting_settings'
                    )
                ) source
                ORDER BY namespace, normalized_key, updated_at DESC, created_at DESC, id DESC
            ) deduped;

DELETE FROM bot_memory
            WHERE namespace IN (
                'room_blacklist',
                'default_talk_settings',
                'greeting_settings'
            );

UPDATE alembic_version SET version_num='ecf4ab7f2781' WHERE alembic_version.version_num = 'c4f6a1d2b9e7';

-- Running upgrade ecf4ab7f2781 -> 9a389e8a6adf

DELETE FROM bot_memory
            WHERE namespace IN (
                'passkey_register',
                'passkey_login',
                'stock_prices',
                'pal_icon_url'
            );

UPDATE alembic_version SET version_num='9a389e8a6adf' WHERE alembic_version.version_num = 'ecf4ab7f2781';

-- Running upgrade 9a389e8a6adf -> 3c4c9d0d9d61

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
);

CREATE INDEX idx_economy_config_version_created_at ON economy_config_version (created_at);

CREATE TABLE economy_control_plane_state (
    id SERIAL NOT NULL, 
    active_config_version_id INTEGER, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(active_config_version_id) REFERENCES economy_config_version (id)
);

CREATE INDEX ix_economy_control_plane_state_active_config_version_id ON economy_control_plane_state (active_config_version_id);

INSERT INTO economy_config_version (
            schema_version,
            config_payload,
            created_at,
            created_by,
            note,
            parent_version_id
        )
        SELECT
            1,
            jsonb_build_object(
                'global', '{}'::jsonb,
                'turnip_spot', jsonb_build_object(
                    'market_paused', COALESCE(
                        (SELECT is_paused FROM market_maker_state WHERE id = 1),
                        false
                    ),
                    'runtime_controls',
                        COALESCE(
                            (SELECT turnip_runtime_overrides FROM economy_control_state WHERE id = 1),
                            '{}'::jsonb
                        ) || jsonb_build_object(
                            'cash_injection_multiplier', COALESCE(
                                (SELECT cash_injection_multiplier FROM economy_control_state WHERE id = 1),
                                1.00
                            ),
                            'amm_trade_fee_bps', COALESCE(
                                (SELECT amm_trade_fee_bps FROM economy_control_state WHERE id = 1),
                                0
                            ),
                            'treasury_policy_enabled', COALESCE(
                                (SELECT treasury_policy_enabled FROM economy_control_state WHERE id = 1),
                                true
                            ),
                            'treasury_policy_k', (
                                SELECT treasury_policy_k FROM economy_control_state WHERE id = 1
                            ),
                            'treasury_policy_beta', (
                                SELECT treasury_policy_beta FROM economy_control_state WHERE id = 1
                            ),
                            'stored_decay_bps_per_day', COALESCE(
                                (SELECT stored_decay_bps_per_day FROM economy_control_state WHERE id = 1),
                                0
                            )
                        )
                ),
                'futures', jsonb_build_object(
                    'futures_paused', COALESCE(
                        (SELECT futures_paused FROM market_maker_state WHERE id = 1),
                        false
                    )
                ),
                'stock', '{}'::jsonb
            ),
            NOW(),
            'migration:3c4c9d0d9d61',
            'Bootstrap from legacy economy control state',
            NULL
        WHERE NOT EXISTS (SELECT 1 FROM economy_config_version);

INSERT INTO economy_control_plane_state (id, active_config_version_id, updated_at)
        SELECT 1, id, NOW()
        FROM economy_config_version
        WHERE NOT EXISTS (
            SELECT 1 FROM economy_control_plane_state WHERE id = 1
        )
        ORDER BY id DESC
        LIMIT 1;

ALTER TABLE economy_snapshot ADD COLUMN config_version_id INTEGER;

ALTER TABLE economy_snapshot ADD CONSTRAINT fk_economy_snapshot_config_version_id FOREIGN KEY(config_version_id) REFERENCES economy_config_version (id);

CREATE INDEX ix_economy_snapshot_config_version_id ON economy_snapshot (config_version_id);

ALTER TABLE turnip_market_snapshot ADD COLUMN config_version_id INTEGER;

ALTER TABLE turnip_market_snapshot ADD COLUMN event_composition_id INTEGER;

ALTER TABLE turnip_market_snapshot ADD COLUMN controller_state_id INTEGER;

ALTER TABLE turnip_market_snapshot ADD COLUMN farm_output_raw NUMERIC(38, 0);

ALTER TABLE turnip_market_snapshot ADD COLUMN farm_output_ema DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN ambient_supply DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN turnip_allocation_total NUMERIC(38, 0);

ALTER TABLE turnip_market_snapshot ADD COLUMN sink_target_destroy DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN sink_actual_destroy DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN sink_destroy_gap DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN sink_destroy_gap_ema DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN sink_bid_bias_pct DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN source_ask_bias_pct DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN sink_budget_multiplier DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN clamp_flags_json JSONB;

ALTER TABLE turnip_market_snapshot ADD CONSTRAINT fk_turnip_market_snapshot_config_version_id FOREIGN KEY(config_version_id) REFERENCES economy_config_version (id);

CREATE INDEX ix_turnip_market_snapshot_config_version_id ON turnip_market_snapshot (config_version_id);

UPDATE alembic_version SET version_num='3c4c9d0d9d61' WHERE alembic_version.version_num = '9a389e8a6adf';

-- Running upgrade 3c4c9d0d9d61 -> 7d4e1a2b8c91

CREATE TABLE turnip_market_controller_state (
    id SERIAL NOT NULL, 
    harvest_raw DECIMAL(38, 2) NOT NULL, 
    harvest_ema DECIMAL(38, 2) NOT NULL, 
    ambient_supply_level DECIMAL(38, 6) NOT NULL, 
    ambient_phase_short DECIMAL(38, 6) NOT NULL, 
    ambient_phase_long DECIMAL(38, 6) NOT NULL, 
    ambient_supply DECIMAL(38, 2) NOT NULL, 
    turnip_allocation_shadow NUMERIC(38, 0) NOT NULL, 
    target_sink_destroy DECIMAL(38, 2) NOT NULL, 
    actual_sink_destroy_last DECIMAL(38, 2) NOT NULL, 
    actual_treasury_buy_quantity NUMERIC(38, 0) NOT NULL, 
    actual_treasury_sell_quantity NUMERIC(38, 0) NOT NULL, 
    destroy_gap DECIMAL(38, 2) NOT NULL, 
    destroy_gap_ema DECIMAL(38, 2) NOT NULL, 
    destroy_gap_integral DECIMAL(38, 2) NOT NULL, 
    sink_bid_bias DECIMAL(38, 6) NOT NULL, 
    source_ask_bias DECIMAL(38, 6) NOT NULL, 
    sink_budget_multiplier DECIMAL(38, 6) NOT NULL, 
    clamp_flags_json JSONB, 
    captured_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX idx_turnip_market_controller_state_captured_at ON turnip_market_controller_state (captured_at);

ALTER TABLE turnip_market_snapshot ADD CONSTRAINT fk_turnip_market_snapshot_controller_state_id FOREIGN KEY(controller_state_id) REFERENCES turnip_market_controller_state (id);

CREATE INDEX ix_turnip_market_snapshot_controller_state_id ON turnip_market_snapshot (controller_state_id);

UPDATE alembic_version SET version_num='7d4e1a2b8c91' WHERE alembic_version.version_num = '3c4c9d0d9d61';

-- Running upgrade 7d4e1a2b8c91 -> 91b7de0c4fa2

ALTER TABLE turnip_market_snapshot ADD COLUMN raw_turnip_allocation_shadow NUMERIC(38, 0);

ALTER TABLE turnip_market_snapshot ADD COLUMN live_sink_bid_bias_pct DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN live_source_ask_bias_pct DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN live_sink_budget_multiplier DECIMAL(38, 6);

UPDATE alembic_version SET version_num='91b7de0c4fa2' WHERE alembic_version.version_num = '7d4e1a2b8c91';

-- Running upgrade 91b7de0c4fa2 -> 17ebc866a0cb

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
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_crop_insurance_expires_status ON crop_insurance_policy (expires_at, status);

CREATE INDEX ix_crop_insurance_policy_expires_at ON crop_insurance_policy (expires_at);

CREATE INDEX ix_crop_insurance_policy_seed_id ON crop_insurance_policy (seed_id);

CREATE INDEX ix_crop_insurance_policy_user_id ON crop_insurance_policy (user_id);

CREATE INDEX ix_crop_insurance_user_status ON crop_insurance_policy (user_id, status);

UPDATE alembic_version SET version_num='17ebc866a0cb' WHERE alembic_version.version_num = '91b7de0c4fa2';

-- Running upgrade 17ebc866a0cb -> 2a91c6d4e5f1

ALTER TABLE turnip_market_controller_state ADD COLUMN ambient_supply_ratio DECIMAL(38, 6);

ALTER TABLE turnip_market_controller_state ADD COLUMN allocation_ratio_vs_harvest DECIMAL(38, 6);

ALTER TABLE turnip_market_controller_state ADD COLUMN reference_price DECIMAL(38, 2);

ALTER TABLE turnip_market_controller_state ADD COLUMN oracle_mode VARCHAR(32);

ALTER TABLE turnip_market_controller_state ADD COLUMN price_error DECIMAL(38, 6);

ALTER TABLE turnip_market_controller_state ADD COLUMN price_error_ema DECIMAL(38, 6);

ALTER TABLE turnip_market_controller_state ADD COLUMN target_support_ratio DECIMAL(38, 6);

ALTER TABLE turnip_market_controller_state ADD COLUMN actual_support_ratio DECIMAL(38, 6);

ALTER TABLE turnip_market_controller_state ADD COLUMN support_gap DECIMAL(38, 6);

ALTER TABLE turnip_market_controller_state ADD COLUMN support_gap_ema DECIMAL(38, 6);

ALTER TABLE turnip_market_controller_state ADD COLUMN demand_pressure_integral DECIMAL(38, 6);

ALTER TABLE turnip_market_controller_state ADD COLUMN sink_capacity_baseline DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN ambient_supply_ratio DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN allocation_ratio_vs_harvest DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN reference_price DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN oracle_mode VARCHAR(32);

ALTER TABLE turnip_market_snapshot ADD COLUMN price_error DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN price_error_ema DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN target_support_ratio DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN actual_support_ratio DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_gap DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_gap_ema DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN sink_capacity_baseline DECIMAL(38, 2);

UPDATE alembic_version SET version_num='2a91c6d4e5f1' WHERE alembic_version.version_num = '17ebc866a0cb';

-- Running upgrade 2a91c6d4e5f1 -> 79dc0bda57b1

ALTER TABLE raid_profile ALTER COLUMN name TYPE VARCHAR(256);

UPDATE alembic_version SET version_num='79dc0bda57b1' WHERE alembic_version.version_num = '2a91c6d4e5f1';

-- Running upgrade 79dc0bda57b1 -> b1c2d3e4f5a6

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
);

CREATE INDEX idx_turnip_market_event_version_created_at ON turnip_market_event_version (created_at);

CREATE TABLE turnip_market_event_instance (
    id SERIAL NOT NULL, 
    event_version_id INTEGER NOT NULL, 
    starts_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    ends_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    priority INTEGER NOT NULL, 
    weight DECIMAL(38, 6) NOT NULL, 
    paused BOOLEAN NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(event_version_id) REFERENCES turnip_market_event_version (id)
);

CREATE INDEX idx_turnip_market_event_instance_window ON turnip_market_event_instance (starts_at, ends_at);

CREATE INDEX ix_turnip_market_event_instance_event_version_id ON turnip_market_event_instance (event_version_id);

CREATE TABLE turnip_market_event_composition (
    id SERIAL NOT NULL, 
    payload_hash VARCHAR(64) NOT NULL, 
    payload JSONB NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX idx_turnip_market_event_composition_created_at ON turnip_market_event_composition (created_at);

CREATE INDEX ix_turnip_market_event_composition_payload_hash ON turnip_market_event_composition (payload_hash);

CREATE TABLE turnip_market_guardrail_policy_version (
    id SERIAL NOT NULL, 
    mode VARCHAR(32) NOT NULL, 
    oracle_band_pct DECIMAL(38, 6) NOT NULL, 
    admission_band_pct DECIMAL(38, 6) NOT NULL, 
    execution_hard_band_pct DECIMAL(38, 6) NOT NULL, 
    max_order_quantity BIGINT NOT NULL, 
    max_market_order_quantity BIGINT NOT NULL, 
    max_sink_depth_turnips BIGINT NOT NULL, 
    max_source_depth_turnips BIGINT, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    created_by VARCHAR, 
    note VARCHAR, 
    parent_version_id INTEGER, 
    PRIMARY KEY (id), 
    FOREIGN KEY(parent_version_id) REFERENCES turnip_market_guardrail_policy_version (id)
);

CREATE INDEX idx_turnip_market_guardrail_policy_version_created_at ON turnip_market_guardrail_policy_version (created_at);

ALTER TABLE economy_control_plane_state ADD COLUMN active_guardrail_policy_version_id INTEGER;

CREATE INDEX ix_ecps_active_guardrail_policy_vid ON economy_control_plane_state (active_guardrail_policy_version_id);

ALTER TABLE economy_control_plane_state ADD CONSTRAINT fk_ecps_active_guardrail_policy_vid FOREIGN KEY(active_guardrail_policy_version_id) REFERENCES turnip_market_guardrail_policy_version (id);

ALTER TABLE turnip_market_snapshot ADD COLUMN guardrail_mode VARCHAR(32);

ALTER TABLE turnip_market_snapshot ADD COLUMN oracle_guardrail_hit BOOLEAN;

ALTER TABLE turnip_market_snapshot ADD COLUMN admission_guardrail_hit BOOLEAN;

ALTER TABLE turnip_market_snapshot ADD COLUMN execution_guardrail_hit BOOLEAN;

UPDATE alembic_version SET version_num='b1c2d3e4f5a6' WHERE alembic_version.version_num = '79dc0bda57b1';

-- Running upgrade b1c2d3e4f5a6 -> afd944cad12e

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_drift DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_vol DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_jump DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN scenario_supply_mult DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN scenario_demand_mult DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN scenario_npc_direction DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_seed_hash VARCHAR(64);

UPDATE alembic_version SET version_num='afd944cad12e' WHERE alembic_version.version_num = 'b1c2d3e4f5a6';

-- Running upgrade afd944cad12e -> 3ad854823a8c

ALTER TABLE turnip_market_snapshot ADD COLUMN last_raw_trade_price DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN last_qualified_trade_price DECIMAL(38, 2);

UPDATE alembic_version SET version_num='3ad854823a8c' WHERE alembic_version.version_num = 'afd944cad12e';

-- Running upgrade 3ad854823a8c -> 5c4c91f6a2d1

ALTER TABLE turnip_market_controller_state ADD COLUMN recovery_state VARCHAR(32) DEFAULT 'normal' NOT NULL;

ALTER TABLE turnip_market_controller_state ADD COLUMN controller_jump_residual DECIMAL(38, 6) DEFAULT '0' NOT NULL;

ALTER TABLE turnip_market_controller_state ADD COLUMN last_controller_jump_tick INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE turnip_market_controller_state ADD COLUMN recovery_entered_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE turnip_market_controller_state ADD COLUMN recovery_ticks_in_state INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE turnip_market_controller_state ALTER COLUMN recovery_state DROP DEFAULT;

ALTER TABLE turnip_market_controller_state ALTER COLUMN controller_jump_residual DROP DEFAULT;

ALTER TABLE turnip_market_controller_state ALTER COLUMN last_controller_jump_tick DROP DEFAULT;

ALTER TABLE turnip_market_controller_state ALTER COLUMN recovery_ticks_in_state DROP DEFAULT;

UPDATE alembic_version SET version_num='5c4c91f6a2d1' WHERE alembic_version.version_num = '3ad854823a8c';

-- Running upgrade 5c4c91f6a2d1 -> 6d1b7f2a1e4b

ALTER TABLE turnip_market_snapshot ADD COLUMN qualified_fill_seen_tick BOOLEAN DEFAULT false NOT NULL;

UPDATE turnip_market_snapshot
        SET qualified_fill_seen_tick = TRUE
        WHERE last_raw_trade_price IS NOT NULL
          AND last_qualified_trade_price IS NOT NULL
          AND last_raw_trade_price = last_qualified_trade_price;

ALTER TABLE turnip_market_snapshot ALTER COLUMN qualified_fill_seen_tick DROP DEFAULT;

UPDATE alembic_version SET version_num='6d1b7f2a1e4b' WHERE alembic_version.version_num = '5c4c91f6a2d1';

-- Running upgrade 6d1b7f2a1e4b -> 9a3d4c1e2f77

ALTER TABLE turnip_market_controller_state ADD COLUMN ticks_since_last_qualified_trade INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE turnip_market_controller_state ALTER COLUMN ticks_since_last_qualified_trade DROP DEFAULT;

UPDATE alembic_version SET version_num='9a3d4c1e2f77' WHERE alembic_version.version_num = '6d1b7f2a1e4b';

-- Running upgrade 9a3d4c1e2f77 -> f73fc99e1b4d

ALTER TABLE turnip_market_guardrail_policy_version ADD COLUMN max_limit_order_notional_vs_nav_ratio NUMERIC(12, 6) DEFAULT '0.020000' NOT NULL;

ALTER TABLE turnip_market_guardrail_policy_version ADD COLUMN max_market_order_notional_vs_turnover_ratio NUMERIC(12, 6) DEFAULT '0.250000' NOT NULL;

ALTER TABLE turnip_market_guardrail_policy_version ADD COLUMN max_taking_order_quantity_vs_visible_depth_ratio NUMERIC(12, 6) DEFAULT '0.750000' NOT NULL;

ALTER TABLE turnip_market_guardrail_policy_version ADD COLUMN sink_quote_budget_cash_ratio NUMERIC(12, 6) DEFAULT '0.100000' NOT NULL;

ALTER TABLE turnip_market_guardrail_policy_version ADD COLUMN source_quote_budget_inventory_ratio NUMERIC(12, 6) DEFAULT '0.100000' NOT NULL;

ALTER TABLE turnip_market_guardrail_policy_version DROP COLUMN max_order_quantity;

ALTER TABLE turnip_market_guardrail_policy_version DROP COLUMN max_market_order_quantity;

ALTER TABLE turnip_market_guardrail_policy_version DROP COLUMN max_sink_depth_turnips;

ALTER TABLE turnip_market_guardrail_policy_version DROP COLUMN max_source_depth_turnips;

UPDATE alembic_version SET version_num='f73fc99e1b4d' WHERE alembic_version.version_num = '9a3d4c1e2f77';

-- Running upgrade 9a3d4c1e2f77 -> b4c1d2e3f4a5

DROP INDEX ix_turnip_market_snapshot_controller_state_id;

ALTER TABLE turnip_market_snapshot DROP CONSTRAINT fk_turnip_market_snapshot_controller_state_id;

ALTER TABLE turnip_market_snapshot DROP COLUMN controller_state_id;

DROP TABLE turnip_market_controller_state;

INSERT INTO alembic_version (version_num) VALUES ('b4c1d2e3f4a5') RETURNING alembic_version.version_num;

-- Running upgrade b4c1d2e3f4a5 -> 840c203e66f9

ALTER TABLE wallet ADD COLUMN projection_enabled BOOLEAN DEFAULT true NOT NULL;

ALTER TABLE wallet ADD COLUMN allow_negative_balance BOOLEAN DEFAULT false NOT NULL;

ALTER TABLE wallet_transaction ADD COLUMN escrow_delta DECIMAL(38, 2) DEFAULT '0' NOT NULL;

ALTER TABLE wallet_transaction ALTER COLUMN balance_after DROP NOT NULL;

ALTER TABLE wallet_transaction ALTER COLUMN tx_group_id TYPE VARCHAR(100);

CREATE INDEX ix_wallet_transaction_user_id_cover_amount ON wallet_transaction (user_id) INCLUDE (amount, escrow_delta);

INSERT INTO wallet (
                user_id,
                balance,
                escrow_balance,
                projection_enabled,
                allow_negative_balance,
                total_credited,
                created_at
            )
            VALUES (
                NULL,
                0,
                0,
                false,
                true,
                0,
                now()
            )
            ON CONFLICT (user_id) DO NOTHING;

UPDATE wallet
            SET projection_enabled = true,
                allow_negative_balance = false;

UPDATE wallet
            SET projection_enabled = false,
                allow_negative_balance = true
            WHERE user_id IN (NULL, NULL);

WITH escrow_rows AS (
                SELECT
                    wt.id,
                    wt.user_id,
                    wt.escrow_after,
                    COUNT(*) FILTER (WHERE wt.escrow_after IS NOT NULL)
                        OVER (PARTITION BY wt.user_id ORDER BY wt.created_at, wt.id)
                        AS escrow_seq
                FROM wallet_transaction wt
            ),
            escrow_prev AS (
                SELECT
                    cur.id,
                    cur.escrow_after,
                    prev.escrow_after AS prev_escrow_after
                FROM escrow_rows cur
                LEFT JOIN escrow_rows prev
                  ON prev.user_id = cur.user_id
                 AND prev.escrow_seq = cur.escrow_seq - 1
                 AND prev.escrow_after IS NOT NULL
                WHERE cur.escrow_after IS NOT NULL
            )
            UPDATE wallet_transaction wt
            SET escrow_delta = CASE
                WHEN wt.escrow_after IS NULL THEN 0
                ELSE ep.escrow_after - COALESCE(ep.prev_escrow_after, 0)
            END
            FROM escrow_prev ep
            WHERE wt.id = ep.id;

UPDATE wallet_transaction
            SET counterparty_id = user_id,
                tx_group_id = 'legacy-self-' || id::text
            WHERE tx_group_id IS NULL
              AND counterparty_id IS NULL
              AND escrow_after IS NOT NULL;

DROP TABLE IF EXISTS wallet_legacy_single_legs;

CREATE TEMP TABLE wallet_legacy_single_legs AS
            SELECT
                id,
                user_id,
                amount,
                escrow_delta,
                tx_type,
                description,
                reference_id,
                memo,
                created_at
            FROM wallet_transaction
            WHERE tx_group_id IS NULL
              AND counterparty_id IS NULL
              AND escrow_after IS NULL;

UPDATE wallet_transaction wt
            SET counterparty_id = NULL,
                tx_group_id = 'legacy-adjust-' || wt.id::text
            WHERE wt.id IN (SELECT id FROM wallet_legacy_single_legs);

INSERT INTO wallet_transaction (
                user_id,
                amount,
                escrow_delta,
                balance_after,
                tx_type,
                description,
                reference_id,
                memo,
                counterparty_id,
                tx_group_id,
                escrow_after,
                created_at
            )
            SELECT
                NULL,
                -amount,
                -escrow_delta,
                NULL,
                tx_type,
                description,
                reference_id,
                memo,
                user_id,
                'legacy-adjust-' || id::text,
                NULL,
                created_at
            FROM wallet_legacy_single_legs;

DROP TABLE IF EXISTS wallet_projected_drift_reconcile;

CREATE TEMP TABLE wallet_projected_drift_reconcile AS
            WITH projected_wallets AS (
                SELECT user_id, balance, escrow_balance
                FROM wallet
                WHERE projection_enabled = true
            ),
            ledger_totals AS (
                SELECT
                    user_id,
                    COALESCE(SUM(amount), 0) AS ledger_balance,
                    COALESCE(SUM(escrow_delta), 0) AS ledger_escrow
                FROM wallet_transaction
                GROUP BY user_id
            )
            SELECT
                pw.user_id,
                pw.balance,
                pw.escrow_balance,
                pw.balance - COALESCE(lt.ledger_balance, 0) AS balance_diff,
                pw.escrow_balance - COALESCE(lt.ledger_escrow, 0) AS escrow_diff
            FROM projected_wallets pw
            LEFT JOIN ledger_totals lt ON lt.user_id = pw.user_id
            WHERE pw.balance <> COALESCE(lt.ledger_balance, 0)
               OR pw.escrow_balance <> COALESCE(lt.ledger_escrow, 0);

INSERT INTO wallet_transaction (
                user_id,
                amount,
                escrow_delta,
                balance_after,
                tx_type,
                description,
                reference_id,
                memo,
                counterparty_id,
                tx_group_id,
                escrow_after,
                created_at
            )
            SELECT
                user_id,
                balance_diff,
                escrow_diff,
                balance,
                'balance_adjustment',
                '系统迁移修复: wallet ledger projection reconcile',
                'wallet-ledger-projection-reconcile:' || user_id,
                'migration=840c203e66f9;reason=projected_drift_reconcile',
                NULL,
                'migration-reconcile-' || user_id,
                CASE WHEN escrow_diff <> 0 THEN escrow_balance ELSE NULL END,
                now()
            FROM wallet_projected_drift_reconcile;

INSERT INTO wallet_transaction (
                user_id,
                amount,
                escrow_delta,
                balance_after,
                tx_type,
                description,
                reference_id,
                memo,
                counterparty_id,
                tx_group_id,
                escrow_after,
                created_at
            )
            SELECT
                NULL,
                -balance_diff,
                -escrow_diff,
                NULL,
                'balance_adjustment',
                '系统迁移修复: wallet ledger projection reconcile',
                'wallet-ledger-projection-reconcile:' || user_id,
                'migration=840c203e66f9;reason=projected_drift_reconcile',
                user_id,
                'migration-reconcile-' || user_id,
                NULL,
                now()
            FROM wallet_projected_drift_reconcile;

WITH projected_wallets AS (
                SELECT user_id, balance, escrow_balance
                FROM wallet
                WHERE projection_enabled = true
            ),
            ledger_totals AS (
                SELECT
                    user_id,
                    COALESCE(SUM(amount), 0) AS ledger_balance,
                    COALESCE(SUM(escrow_delta), 0) AS ledger_escrow
                FROM wallet_transaction
                GROUP BY user_id
            )
            SELECT pw.user_id
            FROM projected_wallets pw
            LEFT JOIN ledger_totals lt ON lt.user_id = pw.user_id
            WHERE pw.balance <> COALESCE(lt.ledger_balance, 0)
               OR pw.escrow_balance <> COALESCE(lt.ledger_escrow, 0)
            LIMIT 5;

CREATE OR REPLACE FUNCTION wallet_guard_validate_tx_group(p_tx_group_id text)
    RETURNS void AS $$
    DECLARE
        tx_count integer;
        group_amount numeric;
        group_escrow_delta numeric;
        self_user_id text;
        self_counterparty_id text;
        self_amount numeric;
        self_escrow_delta numeric;
    BEGIN
        IF p_tx_group_id IS NULL THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id may not be null'
                USING ERRCODE = '23514';
        END IF;

        SELECT
            COUNT(*),
            COALESCE(SUM(amount), 0),
            COALESCE(SUM(escrow_delta), 0)
        INTO tx_count, group_amount, group_escrow_delta
        FROM wallet_transaction
        WHERE tx_group_id = p_tx_group_id;

        IF tx_count = 1 THEN
            SELECT user_id, counterparty_id, amount, escrow_delta
            INTO self_user_id, self_counterparty_id, self_amount, self_escrow_delta
            FROM wallet_transaction
            WHERE tx_group_id = p_tx_group_id
            LIMIT 1;

            IF self_user_id IS DISTINCT FROM self_counterparty_id
               OR self_amount + self_escrow_delta <> 0 THEN
                RAISE EXCEPTION
                    'wallet ledger guard: tx_group_id % is not a valid self-group',
                    p_tx_group_id
                    USING ERRCODE = '23514';
            END IF;
        ELSIF tx_count > 2 THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id % must not have more than 2 rows, got %',
                p_tx_group_id,
                tx_count
                USING ERRCODE = '23514';
        ELSIF EXISTS (
            SELECT 1
            FROM wallet_transaction wt
            WHERE wt.tx_group_id = p_tx_group_id
              AND (
                  SELECT COUNT(*)
                  FROM wallet_transaction peer
                  WHERE peer.tx_group_id = wt.tx_group_id
                    AND peer.id <> wt.id
                    AND peer.user_id = wt.counterparty_id
                    AND peer.counterparty_id = wt.user_id
              ) <> 1
        ) THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id % has broken counterparty links',
                p_tx_group_id
                USING ERRCODE = '23514';
        END IF;

        IF group_amount + group_escrow_delta <> 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id % must balance amount+escrow_delta to zero, got %',
                p_tx_group_id,
                group_amount + group_escrow_delta
                USING ERRCODE = '23514';
        END IF;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_validate_wallet(p_user_id text)
    RETURNS void AS $$
    DECLARE
        ledger_balance numeric;
        ledger_escrow numeric;
        wallet_allow_negative boolean;
    BEGIN
        IF p_user_id IS NULL THEN
            RETURN;
        END IF;

        SELECT
            "allow_negative_balance"
        INTO
            wallet_allow_negative
        FROM "wallet"
        WHERE "user_id" = p_user_id;

        IF NOT FOUND THEN
            IF EXISTS (
                SELECT 1
                FROM "wallet_transaction"
                WHERE "user_id" = p_user_id
            ) THEN
                RAISE EXCEPTION
                    'wallet ledger guard: wallet % missing while transactions exist',
                    p_user_id
                    USING ERRCODE = '23514';
            END IF;
            RETURN;
        END IF;

        SELECT COALESCE(SUM("amount"), 0)
        INTO ledger_balance
        FROM "wallet_transaction"
        WHERE "user_id" = p_user_id;

        SELECT COALESCE(SUM("escrow_delta"), 0)
        INTO ledger_escrow
        FROM "wallet_transaction"
        WHERE "user_id" = p_user_id;

        -- Best-effort guard for append-only wallets: this rechecks committed
        -- rows visible to this transaction, but intentionally does not take a
        -- per-wallet serialization lock. Extreme concurrent spends may still
        -- commit into a negative ledger balance and are handled by follow-up
        -- balance reads / recovery flows.
        IF NOT wallet_allow_negative AND ledger_balance < 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: wallet % may not go negative (ledger=%)',
                p_user_id,
                ledger_balance
                USING ERRCODE = '23514';
        END IF;

        IF ledger_escrow < 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: wallet % may not have negative escrow (ledger=%)',
                p_user_id,
                ledger_escrow
                USING ERRCODE = '23514';
        END IF;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_enforce_tx_group()
    RETURNS trigger AS $$
    BEGIN
        IF TG_OP = 'DELETE' THEN
            PERFORM wallet_guard_validate_tx_group(OLD.tx_group_id);
            RETURN NULL;
        END IF;

        PERFORM wallet_guard_validate_tx_group(NEW.tx_group_id);
        IF TG_OP = 'UPDATE' AND OLD.tx_group_id IS DISTINCT FROM NEW.tx_group_id THEN
            PERFORM wallet_guard_validate_tx_group(OLD.tx_group_id);
        END IF;
        RETURN NULL;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_enforce_tx_wallet()
    RETURNS trigger AS $$
    BEGIN
        IF TG_OP = 'DELETE' THEN
            PERFORM wallet_guard_validate_wallet(OLD.user_id);
            RETURN NULL;
        END IF;

        PERFORM wallet_guard_validate_wallet(NEW.user_id);
        IF TG_OP = 'UPDATE' AND OLD.user_id IS DISTINCT FROM NEW.user_id THEN
            PERFORM wallet_guard_validate_wallet(OLD.user_id);
        END IF;
        RETURN NULL;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_enforce_wallet()
    RETURNS trigger AS $$
    BEGIN
        IF TG_OP = 'DELETE' THEN
            PERFORM wallet_guard_validate_wallet(OLD.user_id);
            RETURN NULL;
        END IF;

        PERFORM wallet_guard_validate_wallet(NEW.user_id);
        RETURN NULL;
    END;
    $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS wallet_tx_group_guard ON wallet_transaction;

CREATE CONSTRAINT TRIGGER wallet_tx_group_guard
    AFTER INSERT OR UPDATE OF user_id, amount, escrow_delta, counterparty_id, tx_group_id OR DELETE ON wallet_transaction
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION wallet_guard_enforce_tx_group();

DROP TRIGGER IF EXISTS wallet_tx_wallet_guard ON wallet_transaction;

CREATE CONSTRAINT TRIGGER wallet_tx_wallet_guard
    AFTER INSERT OR UPDATE OF user_id, amount, escrow_delta, counterparty_id, tx_group_id OR DELETE ON wallet_transaction
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION wallet_guard_enforce_tx_wallet();

DROP TRIGGER IF EXISTS wallet_row_wallet_guard ON wallet;

CREATE CONSTRAINT TRIGGER wallet_row_wallet_guard
    AFTER INSERT OR UPDATE OF user_id, allow_negative_balance OR DELETE ON wallet
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION wallet_guard_enforce_wallet();

UPDATE alembic_version SET version_num='840c203e66f9' WHERE alembic_version.version_num = 'b4c1d2e3f4a5';

-- Running upgrade 840c203e66f9, f73fc99e1b4d -> 2f0c61b1c8a7

CREATE TABLE partner_client (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    client_id VARCHAR(128) NOT NULL, 
    client_secret_encrypted VARCHAR(4096) NOT NULL, 
    status VARCHAR(32) DEFAULT 'active' NOT NULL, 
    allowed_scopes JSONB DEFAULT '[]'::jsonb NOT NULL, 
    allowed_redirect_uris JSONB DEFAULT '[]'::jsonb NOT NULL, 
    webhook_url VARCHAR(2048), 
    webhook_secret_encrypted VARCHAR(4096), 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    CONSTRAINT ck_partner_client_status CHECK (status IN ('active', 'disabled', 'revoked'))
);

CREATE UNIQUE INDEX ix_partner_client_client_id ON partner_client (client_id);

CREATE INDEX ix_partner_client_user_id ON partner_client (user_id);

CREATE TABLE partner_refresh_token (
    id SERIAL NOT NULL, 
    token_id VARCHAR(128) NOT NULL, 
    partner_user_id VARCHAR NOT NULL, 
    client_id VARCHAR(128) NOT NULL, 
    token_hash VARCHAR(128) NOT NULL, 
    scope VARCHAR(2048) DEFAULT '' NOT NULL, 
    status VARCHAR(32) DEFAULT 'active' NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    last_used_at TIMESTAMP WITH TIME ZONE, 
    rotated_from_token_id VARCHAR(128), 
    revoked_at TIMESTAMP WITH TIME ZONE, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_partner_refresh_token_client_id ON partner_refresh_token (client_id);

CREATE UNIQUE INDEX ix_partner_refresh_token_hash ON partner_refresh_token (token_hash);

CREATE UNIQUE INDEX ix_partner_refresh_token_token_id ON partner_refresh_token (token_id);

CREATE TABLE oidc_refresh_token (
    id SERIAL NOT NULL, 
    token_id VARCHAR(128) NOT NULL, 
    partner_user_id VARCHAR NOT NULL, 
    end_user_id VARCHAR NOT NULL, 
    client_id VARCHAR(128) NOT NULL, 
    token_hash VARCHAR(128) NOT NULL, 
    scope VARCHAR(2048) DEFAULT 'openid profile' NOT NULL, 
    status VARCHAR(32) DEFAULT 'active' NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    last_used_at TIMESTAMP WITH TIME ZONE, 
    rotated_from_token_id VARCHAR(128), 
    revoked_at TIMESTAMP WITH TIME ZONE, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_oidc_refresh_token_client_id ON oidc_refresh_token (client_id);

CREATE UNIQUE INDEX ix_oidc_refresh_token_hash ON oidc_refresh_token (token_hash);

CREATE UNIQUE INDEX ix_oidc_refresh_token_token_id ON oidc_refresh_token (token_id);

DELETE FROM alembic_version WHERE alembic_version.version_num = '840c203e66f9';

UPDATE alembic_version SET version_num='2f0c61b1c8a7' WHERE alembic_version.version_num = 'f73fc99e1b4d';

-- Running upgrade 2f0c61b1c8a7 -> 6b1d7af48c31

CREATE TABLE payment_intent (
    intent_id VARCHAR(64) NOT NULL, 
    partner_user_id VARCHAR NOT NULL, 
    partner_client_id VARCHAR(128) NOT NULL, 
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
    checkout_token VARCHAR(64) NOT NULL, 
    status VARCHAR(64) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    authorized_at TIMESTAMP WITH TIME ZONE, 
    completed_at TIMESTAMP WITH TIME ZONE, 
    cancelled_at TIMESTAMP WITH TIME ZONE, 
    error_code VARCHAR(64), 
    error_message VARCHAR(2000), 
    PRIMARY KEY (intent_id)
);

CREATE INDEX ix_payment_intent_partner_user_id ON payment_intent (partner_user_id);

CREATE INDEX ix_payment_intent_partner_client_id ON payment_intent (partner_client_id);

CREATE INDEX ix_payment_intent_user_id ON payment_intent (user_id);

CREATE INDEX ix_payment_intent_partner_reference_id ON payment_intent (partner_reference_id);

CREATE INDEX ix_payment_intent_status ON payment_intent (status);

CREATE UNIQUE INDEX ux_payment_intent_checkout_token ON payment_intent (checkout_token);

CREATE UNIQUE INDEX ux_payment_intent_partner_reference_id ON payment_intent (partner_user_id, partner_reference_id);

UPDATE alembic_version SET version_num='6b1d7af48c31' WHERE alembic_version.version_num = '2f0c61b1c8a7';

-- Running upgrade 6b1d7af48c31 -> 8b2f3161df02

CREATE TABLE clearing_instruction (
    instruction_id VARCHAR(64) NOT NULL, 
    partner_user_id VARCHAR NOT NULL, 
    partner_client_id VARCHAR(128) NOT NULL, 
    intent_id VARCHAR(64), 
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
    reverse_of_instruction_id VARCHAR(64), 
    executed_at TIMESTAMP WITH TIME ZONE, 
    error_code VARCHAR(64), 
    error_message VARCHAR(2000), 
    PRIMARY KEY (instruction_id), 
    FOREIGN KEY(intent_id) REFERENCES payment_intent (intent_id)
);

CREATE INDEX ix_clearing_instruction_partner_user_id ON clearing_instruction (partner_user_id);

CREATE INDEX ix_clearing_instruction_partner_client_id ON clearing_instruction (partner_client_id);

CREATE INDEX ix_clearing_instruction_intent_id ON clearing_instruction (intent_id);

CREATE INDEX ix_clearing_instruction_status ON clearing_instruction (status);

CREATE UNIQUE INDEX ux_clearing_instruction_partner_reference_id ON clearing_instruction (partner_user_id, partner_reference_id);

CREATE TABLE clearing_batch (
    batch_id VARCHAR(64) NOT NULL, 
    partner_user_id VARCHAR NOT NULL, 
    partner_client_id VARCHAR(128) NOT NULL, 
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
    PRIMARY KEY (batch_id)
);

CREATE INDEX ix_clearing_batch_partner_user_id ON clearing_batch (partner_user_id);

CREATE INDEX ix_clearing_batch_partner_client_id ON clearing_batch (partner_client_id);

CREATE INDEX ix_clearing_batch_status ON clearing_batch (status);

CREATE UNIQUE INDEX ux_clearing_batch_batch_reference_id ON clearing_batch (partner_user_id, batch_reference_id);

CREATE TABLE clearing_batch_item (
    batch_item_id VARCHAR(64) NOT NULL, 
    batch_id VARCHAR(64) NOT NULL, 
    intent_id VARCHAR(64), 
    user_id VARCHAR NOT NULL, 
    amount DECIMAL(38, 2), 
    partner_reference_id VARCHAR(255) NOT NULL, 
    note VARCHAR(2000), 
    status VARCHAR(64) NOT NULL, 
    error_code VARCHAR(64), 
    error_message VARCHAR(2000), 
    PRIMARY KEY (batch_item_id), 
    FOREIGN KEY(batch_id) REFERENCES clearing_batch (batch_id), 
    FOREIGN KEY(intent_id) REFERENCES payment_intent (intent_id)
);

CREATE INDEX ix_clearing_batch_item_batch_id ON clearing_batch_item (batch_id);

CREATE INDEX ix_clearing_batch_item_status ON clearing_batch_item (status);

UPDATE alembic_version SET version_num='8b2f3161df02' WHERE alembic_version.version_num = '6b1d7af48c31';

-- Running upgrade 8b2f3161df02 -> 25a4322ca7f7

ALTER TABLE wallet_transaction ADD COLUMN metadata JSONB;

UPDATE alembic_version SET version_num='25a4322ca7f7' WHERE alembic_version.version_num = '8b2f3161df02';

-- Running upgrade 2f0c61b1c8a7 -> f57e633edcac

ALTER TABLE partner_client ADD COLUMN name VARCHAR(128);

ALTER TABLE partner_client ADD COLUMN client_scopes JSONB DEFAULT '[]'::jsonb NOT NULL;

ALTER TABLE partner_client ADD COLUMN user_scopes JSONB DEFAULT '[]'::jsonb NOT NULL;

UPDATE partner_client
        SET
            name = CONCAT('Client ', LEFT(client_id, 8)),
            client_scopes = allowed_scopes,
            user_scopes = '[]'::jsonb;

ALTER TABLE partner_client ALTER COLUMN name SET NOT NULL;

ALTER TABLE partner_client DROP COLUMN allowed_scopes;

INSERT INTO alembic_version (version_num) VALUES ('f57e633edcac') RETURNING alembic_version.version_num;

-- Running upgrade 25a4322ca7f7, f57e633edcac -> 4c3a4fbf0a6d

CREATE TABLE webhook_delivery (
    event_id VARCHAR(64) NOT NULL, 
    partner_user_id VARCHAR NOT NULL, 
    partner_client_id VARCHAR(128) NOT NULL, 
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
    PRIMARY KEY (event_id)
);

CREATE INDEX ix_webhook_delivery_partner_user_id ON webhook_delivery (partner_user_id);

CREATE INDEX ix_webhook_delivery_partner_client_id ON webhook_delivery (partner_client_id);

CREATE INDEX ix_webhook_delivery_status ON webhook_delivery (status);

CREATE INDEX ix_webhook_delivery_qstash_message_id ON webhook_delivery (qstash_message_id);

DELETE FROM alembic_version WHERE alembic_version.version_num = '25a4322ca7f7';

UPDATE alembic_version SET version_num='4c3a4fbf0a6d' WHERE alembic_version.version_num = 'f57e633edcac';

-- Running upgrade 25a4322ca7f7 -> 2c4a0c0d8f6b

CREATE TABLE idempotency_record (
    record_id VARCHAR(64) NOT NULL, 
    key VARCHAR(128) NOT NULL, 
    partner_user_id VARCHAR NOT NULL, 
    partner_client_id VARCHAR(128) NOT NULL, 
    endpoint VARCHAR(255) NOT NULL, 
    request_hash VARCHAR(64) NOT NULL, 
    response_status INTEGER, 
    response_body JSONB, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (record_id)
);

CREATE INDEX ix_idempotency_record_partner_user_id ON idempotency_record (partner_user_id);

CREATE UNIQUE INDEX ux_idempotency_record_partner_client_endpoint_key ON idempotency_record (partner_client_id, endpoint, key);

INSERT INTO alembic_version (version_num) VALUES ('2c4a0c0d8f6b') RETURNING alembic_version.version_num;

-- Running upgrade 2c4a0c0d8f6b, 4c3a4fbf0a6d -> e1d2874b1446

DELETE FROM alembic_version WHERE alembic_version.version_num = '2c4a0c0d8f6b';

UPDATE alembic_version SET version_num='e1d2874b1446' WHERE alembic_version.version_num = '4c3a4fbf0a6d';

-- Running upgrade e1d2874b1446 -> a7f3b2c1d4e5

ALTER TABLE turnip_market_snapshot ADD COLUMN raw_sink_bid_bias_pct DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN raw_source_ask_bias_pct DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN raw_sink_budget_multiplier DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN smoothed_drift_bias DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN smoothed_vol_bias DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN source_quota NUMERIC(38, 0);

ALTER TABLE turnip_market_snapshot ADD COLUMN sink_quota DECIMAL(24, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN turnip_injection NUMERIC(38, 0);

ALTER TABLE turnip_market_snapshot ADD COLUMN cash_injection DECIMAL(24, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN scenario_npc_activity_mult DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN scenario_npc_spread_tolerance DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN scenario_random_trader_boost DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN scenario_cash_alloc_drift DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN scenario_panic_severity DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN scenario_euphoria_severity DECIMAL(24, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN npc_reference_notional DECIMAL(24, 2);

UPDATE alembic_version SET version_num='a7f3b2c1d4e5' WHERE alembic_version.version_num = 'e1d2874b1446';

-- Running upgrade a7f3b2c1d4e5 -> b1a2c3d4e5f6

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_jump_residual DECIMAL(24, 6);

UPDATE alembic_version SET version_num='b1a2c3d4e5f6' WHERE alembic_version.version_num = 'a7f3b2c1d4e5';

-- Running upgrade b1a2c3d4e5f6 -> 9cd45c347213

ALTER TABLE turnip_market_snapshot ADD COLUMN guardrail_policy_version_id INTEGER;

ALTER TABLE turnip_market_snapshot ADD CONSTRAINT fk_turnip_market_snapshot_guardrail_policy_version_id FOREIGN KEY(guardrail_policy_version_id) REFERENCES turnip_market_guardrail_policy_version (id) ON DELETE RESTRICT;

CREATE INDEX ix_turnip_market_snapshot_guardrail_policy_version_id ON turnip_market_snapshot (guardrail_policy_version_id);

ALTER TABLE turnip_market_snapshot ADD COLUMN prev_snapshot_id INTEGER;

ALTER TABLE turnip_market_snapshot ADD CONSTRAINT fk_turnip_market_snapshot_prev_snapshot_id FOREIGN KEY(prev_snapshot_id) REFERENCES turnip_market_snapshot (id) ON DELETE RESTRICT;

CREATE UNIQUE INDEX ux_turnip_market_snapshot_prev_snapshot_id ON turnip_market_snapshot (prev_snapshot_id) WHERE prev_snapshot_id IS NOT NULL;

UPDATE alembic_version SET version_num='9cd45c347213' WHERE alembic_version.version_num = 'b1a2c3d4e5f6';

-- Running upgrade 7aa54c2ecbc9 -> 202604200210

ALTER TABLE market_order ALTER COLUMN quantity TYPE DECIMAL(38, 0);

ALTER TABLE market_order ALTER COLUMN filled_quantity TYPE DECIMAL(38, 0);

ALTER TABLE market_order_fill ALTER COLUMN quantity TYPE DECIMAL(38, 0);

INSERT INTO alembic_version (version_num) VALUES ('202604200210') RETURNING alembic_version.version_num;

-- Running upgrade 202604200210 -> 202604200255

ALTER TABLE raid_profile ALTER COLUMN total_loot_value TYPE DECIMAL(38, 0);

UPDATE alembic_version SET version_num='202604200255' WHERE alembic_version.version_num = '202604200210';

-- Running upgrade 202604200255 -> ab4e92d6c1f0

ALTER TABLE stock_pending_order ADD COLUMN settlement_policy VARCHAR(48) DEFAULT 'strict_anchor' NOT NULL;

ALTER TABLE stock_pending_order ALTER COLUMN settlement_policy DROP DEFAULT;

UPDATE alembic_version SET version_num='ab4e92d6c1f0' WHERE alembic_version.version_num = '202604200255';

-- Running upgrade ab4e92d6c1f0 -> ca32ed06219a

ALTER TABLE pal ADD COLUMN asset_id VARCHAR(128);

ALTER TABLE pal ADD COLUMN pending_gift_id UUID;

ALTER TABLE pal_egg ADD COLUMN asset_id VARCHAR(128);

ALTER TABLE pal_egg ADD COLUMN pending_gift_id UUID;

UPDATE pal SET asset_id = 'pal:' || id::text WHERE asset_id IS NULL;

UPDATE pal_egg SET asset_id = 'egg:' || id::text WHERE asset_id IS NULL;

ALTER TABLE pal ALTER COLUMN asset_id SET NOT NULL;

ALTER TABLE pal_egg ALTER COLUMN asset_id SET NOT NULL;

ALTER TABLE pal ADD CONSTRAINT ck_pal_asset_id_prefix CHECK (asset_id LIKE 'pal:%');

ALTER TABLE pal_egg ADD CONSTRAINT ck_pal_egg_asset_id_prefix CHECK (asset_id LIKE 'egg:%');

CREATE UNIQUE INDEX ix_pal_asset_id ON pal (asset_id);

CREATE INDEX ix_pal_pending_gift_id ON pal (pending_gift_id);

CREATE UNIQUE INDEX ix_pal_egg_asset_id ON pal_egg (asset_id);

CREATE INDEX ix_pal_egg_pending_gift_id ON pal_egg (pending_gift_id);

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
    FOREIGN KEY(from_user_id) REFERENCES wallet (user_id), 
    FOREIGN KEY(to_user_id) REFERENCES wallet (user_id), 
    CONSTRAINT ck_pending_gift_asset_prefix CHECK (asset_id LIKE asset_family || ':%')
);

CREATE UNIQUE INDEX ix_pending_gift_asset_pending_unique ON pending_gift (asset_id) WHERE status = 'pending';

CREATE INDEX ix_pending_gift_incoming ON pending_gift (to_user_id, status, created_at);

CREATE INDEX ix_pending_gift_outgoing ON pending_gift (from_user_id, status, created_at);

CREATE INDEX ix_pending_gift_expire ON pending_gift (status, expires_at);

UPDATE alembic_version SET version_num='ca32ed06219a' WHERE alembic_version.version_num = 'ab4e92d6c1f0';

-- Running upgrade ca32ed06219a -> 202604202320

DROP INDEX ix_pal_asset_id;

ALTER TABLE pal DROP CONSTRAINT ck_pal_asset_id_prefix;

ALTER TABLE pal DROP COLUMN asset_id;

ALTER TABLE pal ADD COLUMN asset_id VARCHAR(128) GENERATED ALWAYS AS ('pal:' || id::text) STORED NOT NULL;

ALTER TABLE pal ADD CONSTRAINT ck_pal_asset_id_prefix CHECK (asset_id LIKE 'pal:%');

CREATE UNIQUE INDEX ix_pal_asset_id ON pal (asset_id);

DROP INDEX ix_pal_egg_asset_id;

ALTER TABLE pal_egg DROP CONSTRAINT ck_pal_egg_asset_id_prefix;

ALTER TABLE pal_egg DROP COLUMN asset_id;

ALTER TABLE pal_egg ADD COLUMN asset_id VARCHAR(128) GENERATED ALWAYS AS ('egg:' || id::text) STORED NOT NULL;

ALTER TABLE pal_egg ADD CONSTRAINT ck_pal_egg_asset_id_prefix CHECK (asset_id LIKE 'egg:%');

CREATE UNIQUE INDEX ix_pal_egg_asset_id ON pal_egg (asset_id);

UPDATE alembic_version SET version_num='202604202320' WHERE alembic_version.version_num = 'ca32ed06219a';

-- Running upgrade 202604202320 -> 202604212029

CREATE TABLE undercover_session (
    id VARCHAR NOT NULL, 
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
);

CREATE INDEX ix_undercover_session_room_id ON undercover_session (room_id);

CREATE INDEX ix_undercover_session_creator_id ON undercover_session (creator_id);

CREATE INDEX ix_undercover_session_status ON undercover_session (status);

CREATE INDEX ix_undercover_session_phase ON undercover_session (phase);

CREATE INDEX ix_undercover_session_phase_deadline_at ON undercover_session (phase_deadline_at);

CREATE INDEX ix_undercover_session_created_at ON undercover_session (created_at);

CREATE INDEX ix_undercover_session_last_activity_at ON undercover_session (last_activity_at);

CREATE INDEX ix_undercover_session_room_status ON undercover_session (room_id, status);

CREATE INDEX ix_undercover_session_creator_status ON undercover_session (creator_id, status);

CREATE UNIQUE INDEX ux_undercover_session_room_active ON undercover_session (room_id) WHERE status in ('waiting', 'playing');

UPDATE alembic_version SET version_num='202604212029' WHERE alembic_version.version_num = '202604202320';

-- Running upgrade b1a2c3d4e5f6 -> c2d3e4f5a6b7

ALTER TABLE turnip_market_snapshot ADD COLUMN applied_jump_source VARCHAR(32);

ALTER TABLE turnip_market_snapshot ADD COLUMN applied_jump_kind VARCHAR(32);

ALTER TABLE turnip_market_snapshot ADD COLUMN applied_jump_size DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN applied_jump_persistence_ratio DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN applied_jump_persistent_shift DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN applied_jump_transient_shift DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN applied_jump_oracle_anchor_before DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ADD COLUMN applied_jump_oracle_anchor_after DECIMAL(38, 2);

INSERT INTO alembic_version (version_num) VALUES ('c2d3e4f5a6b7') RETURNING alembic_version.version_num;

-- Running upgrade 2c4a0c0d8f6b, 4c3a4fbf0a6d -> 91d62ab248b1

CREATE TABLE partner_managed_account (
    id SERIAL NOT NULL, 
    owner_user_id VARCHAR(255) NOT NULL, 
    managed_user_id VARCHAR(255) NOT NULL, 
    status VARCHAR(32) DEFAULT 'active' NOT NULL, 
    can_login BOOLEAN DEFAULT true NOT NULL, 
    created_by_user_id VARCHAR(255), 
    updated_by_user_id VARCHAR(255), 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX ux_partner_managed_account_managed_user_id ON partner_managed_account (managed_user_id);

CREATE UNIQUE INDEX ux_partner_managed_account_owner_managed_user_id ON partner_managed_account (owner_user_id, managed_user_id);

CREATE INDEX ix_partner_managed_account_owner_user_id ON partner_managed_account (owner_user_id);

CREATE TABLE issued_principal (
    principal_id VARCHAR(64) NOT NULL, 
    token_kind VARCHAR(32) NOT NULL, 
    owner_user_id VARCHAR(255) NOT NULL, 
    subject_user_id VARCHAR(255) NOT NULL, 
    effective_account_user_id VARCHAR(255) NOT NULL, 
    actor_user_id VARCHAR(255) NOT NULL, 
    client_id VARCHAR(128), 
    scope_snapshot JSONB NOT NULL, 
    issued_via VARCHAR(32) NOT NULL, 
    source_principal_id VARCHAR(64), 
    expires_at TIMESTAMP WITH TIME ZONE, 
    revoked_at TIMESTAMP WITH TIME ZONE, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (principal_id)
);

CREATE INDEX ix_issued_principal_owner_created_at ON issued_principal (owner_user_id, created_at);

CREATE INDEX ix_issued_principal_effective_created_at ON issued_principal (effective_account_user_id, created_at);

CREATE INDEX ix_issued_principal_client_created_at ON issued_principal (client_id, created_at);

CREATE INDEX ix_issued_principal_source_principal_id ON issued_principal (source_principal_id);

CREATE TABLE security_audit_event (
    event_id VARCHAR(64) NOT NULL, 
    principal_id VARCHAR(64), 
    action VARCHAR(100) NOT NULL, 
    result VARCHAR(32) NOT NULL, 
    target_type VARCHAR(64) NOT NULL, 
    target_id VARCHAR(255), 
    error_code VARCHAR(64), 
    metadata JSONB, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (event_id), 
    FOREIGN KEY(principal_id) REFERENCES issued_principal (principal_id)
);

CREATE INDEX ix_security_audit_event_principal_created_at ON security_audit_event (principal_id, created_at);

CREATE INDEX ix_security_audit_event_target ON security_audit_event (target_type, target_id);

ALTER TABLE wallet_transaction ADD COLUMN principal_id VARCHAR(64);

CREATE INDEX ix_wallet_transaction_principal_id ON wallet_transaction (principal_id);

ALTER TABLE wallet_transaction ADD CONSTRAINT fk_wallet_transaction_principal_id FOREIGN KEY(principal_id) REFERENCES issued_principal (principal_id);

ALTER TABLE partner_refresh_token ADD COLUMN principal_id VARCHAR(64);

CREATE INDEX ix_partner_refresh_token_principal_id ON partner_refresh_token (principal_id);

ALTER TABLE partner_refresh_token ADD CONSTRAINT fk_partner_refresh_token_principal_id FOREIGN KEY(principal_id) REFERENCES issued_principal (principal_id);

INSERT INTO alembic_version (version_num) VALUES ('91d62ab248b1') RETURNING alembic_version.version_num;

-- Running upgrade 9a7f0c1d2e3b -> 9f3a2b7c1d4e

ALTER TABLE raid_warehouse_item ADD COLUMN location VARCHAR(16);

ALTER TABLE raid_warehouse_item ADD COLUMN carry_order INTEGER;

UPDATE raid_warehouse_item
        SET
            location = CASE
                WHEN loadout_slot IS NULL THEN 'warehouse'
                WHEN loadout_slot = 'supply' THEN 'backpack'
                ELSE 'equipped'
            END,
            carry_order = CASE
                WHEN loadout_slot = 'supply' THEN loadout_order
                ELSE NULL
            END;

ALTER TABLE raid_warehouse_item ALTER COLUMN location SET NOT NULL;

CREATE INDEX ix_raid_warehouse_item_user_location_order ON raid_warehouse_item (user_id, location, carry_order);

CREATE UNIQUE INDEX ux_raid_warehouse_item_user_equipped_slot ON raid_warehouse_item (user_id, item_type) WHERE location = 'equipped';

ALTER TABLE raid_warehouse_item
        DROP CONSTRAINT IF EXISTS ck_raid_warehouse_item_loadout_pairing;

DROP INDEX IF EXISTS ux_raid_warehouse_item_user_loadout_equipment_slot;

DROP INDEX IF EXISTS ix_raid_warehouse_item_user_loadout;

ALTER TABLE raid_warehouse_item DROP COLUMN loadout_slot;

ALTER TABLE raid_warehouse_item DROP COLUMN loadout_order;

INSERT INTO alembic_version (version_num) VALUES ('9f3a2b7c1d4e') RETURNING alembic_version.version_num;

-- Running upgrade 91d62ab248b1 -> c0a4a7f1b2c3

INSERT INTO issued_principal (
                principal_id,
                token_kind,
                owner_user_id,
                subject_user_id,
                effective_account_user_id,
                actor_user_id,
                client_id,
                scope_snapshot,
                issued_via,
                source_principal_id,
                expires_at,
                revoked_at,
                created_at
            )
            VALUES (
                'prn_legacy_audit_backfill',
                'audit',
                'legacy_audit_backfill',
                'legacy_audit_backfill',
                'legacy_audit_backfill',
                'legacy_audit_backfill',
                NULL,
                '[]'::jsonb,
                'backfill',
                NULL,
                NULL,
                NULL,
                timezone('utc', now())
            )
            ON CONFLICT (principal_id) DO NOTHING;

UPDATE security_audit_event
            SET principal_id = 'prn_legacy_audit_backfill'
            WHERE principal_id IS NULL;

ALTER TABLE security_audit_event ALTER COLUMN principal_id SET NOT NULL;

UPDATE alembic_version SET version_num='c0a4a7f1b2c3' WHERE alembic_version.version_num = '91d62ab248b1';

-- Running upgrade 91d62ab248b1, b1a2c3d4e5f6 -> 41d2f3c4b5e6

CREATE TABLE turnip_scenario_template (
    id SERIAL NOT NULL, 
    name VARCHAR(120) NOT NULL, 
    description TEXT, 
    default_start_mode VARCHAR(16) NOT NULL, 
    stage_definition_json JSONB NOT NULL, 
    created_by VARCHAR(100), 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    deleted_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id), 
    CONSTRAINT ck_turnip_scenario_template_stage_definition_nonempty CHECK (jsonb_typeof(stage_definition_json) = 'array' AND jsonb_array_length(stage_definition_json) > 0)
);

CREATE INDEX idx_turnip_scenario_template_created_at ON turnip_scenario_template (created_at);

CREATE TABLE turnip_scenario_run (
    id SERIAL NOT NULL, 
    template_id INTEGER, 
    template_snapshot_json JSONB NOT NULL, 
    status VARCHAR(16) NOT NULL, 
    start_mode VARCHAR(16) NOT NULL, 
    scheduled_at TIMESTAMP WITH TIME ZONE, 
    started_at TIMESTAMP WITH TIME ZONE, 
    ended_at TIMESTAMP WITH TIME ZONE, 
    baseline_config_version_id INTEGER, 
    current_stage_index INTEGER NOT NULL, 
    heartbeat_interval_sec INTEGER NOT NULL, 
    last_heartbeat_at TIMESTAMP WITH TIME ZONE, 
    lease_expires_at TIMESTAMP WITH TIME ZONE, 
    created_by VARCHAR(100), 
    abort_reason TEXT, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(baseline_config_version_id) REFERENCES economy_config_version (id), 
    FOREIGN KEY(template_id) REFERENCES turnip_scenario_template (id)
);

CREATE INDEX idx_turnip_scenario_run_status_created_at ON turnip_scenario_run (status, created_at);

CREATE TABLE turnip_scenario_stage (
    id SERIAL NOT NULL, 
    run_id INTEGER NOT NULL, 
    stage_index INTEGER NOT NULL, 
    name VARCHAR(120) NOT NULL, 
    stage_type VARCHAR(32) NOT NULL, 
    mode VARCHAR(16) NOT NULL, 
    duration_sec INTEGER NOT NULL, 
    target_ref VARCHAR(32), 
    target_value DECIMAL(38, 6), 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(run_id) REFERENCES turnip_scenario_run (id)
);

CREATE UNIQUE INDEX idx_turnip_scenario_stage_run_stage_index ON turnip_scenario_stage (run_id, stage_index);

CREATE INDEX ix_turnip_scenario_stage_run_id ON turnip_scenario_stage (run_id);

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
    effective_overlay_hash VARCHAR(64) NOT NULL, 
    supervisor_version VARCHAR(120) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(stage_id) REFERENCES turnip_scenario_stage (id)
);

CREATE INDEX idx_turnip_scenario_overlay_revision_stage_created_at ON turnip_scenario_overlay_revision (stage_id, created_at);

CREATE INDEX ix_turnip_scenario_overlay_revision_stage_id ON turnip_scenario_overlay_revision (stage_id);

ALTER TABLE turnip_market_snapshot ADD COLUMN active_overlay_revision_id INTEGER;

ALTER TABLE turnip_market_snapshot ADD CONSTRAINT fk_turnip_market_snapshot_active_overlay_revision FOREIGN KEY(active_overlay_revision_id) REFERENCES turnip_scenario_overlay_revision (id);

CREATE INDEX ix_turnip_market_snapshot_active_overlay_revision_id ON turnip_market_snapshot (active_overlay_revision_id);

INSERT INTO alembic_version (version_num) VALUES ('41d2f3c4b5e6') RETURNING alembic_version.version_num;

-- Running upgrade 41d2f3c4b5e6, 9cd45c347213, 9f3a2b7c1d4e, c0a4a7f1b2c3, c2d3e4f5a6b7 -> f1e2d3c4b5a6

DELETE FROM alembic_version WHERE alembic_version.version_num = '41d2f3c4b5e6';

DELETE FROM alembic_version WHERE alembic_version.version_num = 'c2d3e4f5a6b7';

DELETE FROM alembic_version WHERE alembic_version.version_num = 'c0a4a7f1b2c3';

DELETE FROM alembic_version WHERE alembic_version.version_num = '9f3a2b7c1d4e';

UPDATE alembic_version SET version_num='f1e2d3c4b5a6' WHERE alembic_version.version_num = '9cd45c347213';

-- Running upgrade f1e2d3c4b5a6 -> bac72ce1cfab

ALTER TABLE turnip_market_snapshot ALTER COLUMN noise_jump_residual TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN raw_sink_bid_bias_pct TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN raw_source_ask_bias_pct TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN raw_sink_budget_multiplier TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN smoothed_drift_bias TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN smoothed_vol_bias TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN sink_quota TYPE DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ALTER COLUMN cash_injection TYPE DECIMAL(38, 2);

ALTER TABLE turnip_market_snapshot ALTER COLUMN scenario_npc_activity_mult TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN scenario_npc_spread_tolerance TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN scenario_random_trader_boost TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN scenario_cash_alloc_drift TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN scenario_panic_severity TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN scenario_euphoria_severity TYPE DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ALTER COLUMN npc_reference_notional TYPE DECIMAL(38, 2);

UPDATE alembic_version SET version_num='bac72ce1cfab' WHERE alembic_version.version_num = 'f1e2d3c4b5a6';

-- Running upgrade bac72ce1cfab -> fde31b9b8e21

ALTER TABLE turnip_scenario_run ADD COLUMN current_stage_started_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE turnip_scenario_run ADD COLUMN paused_at TIMESTAMP WITH TIME ZONE;

UPDATE turnip_scenario_run
        SET current_stage_started_at = started_at
        WHERE status IN ('running', 'paused')
          AND started_at IS NOT NULL
          AND current_stage_started_at IS NULL;

UPDATE alembic_version SET version_num='fde31b9b8e21' WHERE alembic_version.version_num = 'bac72ce1cfab';

-- Running upgrade fde31b9b8e21 -> 4e2a9c7b1f34

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_exec_drift DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_exec_state DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_anchor_price DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_price_gap DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_delta_regime DECIMAL(38, 6);

UPDATE alembic_version SET version_num='4e2a9c7b1f34' WHERE alembic_version.version_num = 'fde31b9b8e21';

-- Running upgrade 4e2a9c7b1f34 -> c6f1b8e2d4a0

ALTER TABLE turnip_market_snapshot DROP COLUMN sink_budget_multiplier;

ALTER TABLE turnip_market_snapshot DROP COLUMN source_ask_bias_pct;

ALTER TABLE turnip_market_snapshot DROP COLUMN sink_bid_bias_pct;

ALTER TABLE turnip_market_snapshot DROP COLUMN sink_capacity_baseline;

ALTER TABLE turnip_market_snapshot DROP COLUMN support_gap_ema;

ALTER TABLE turnip_market_snapshot DROP COLUMN support_gap;

ALTER TABLE turnip_market_snapshot DROP COLUMN actual_support_ratio;

ALTER TABLE turnip_market_snapshot DROP COLUMN target_support_ratio;

ALTER TABLE turnip_market_snapshot DROP COLUMN price_error_ema;

ALTER TABLE turnip_market_snapshot DROP COLUMN price_error;

ALTER TABLE turnip_market_snapshot DROP COLUMN sink_destroy_gap_ema;

ALTER TABLE turnip_market_snapshot DROP COLUMN sink_destroy_gap;

ALTER TABLE turnip_market_snapshot DROP COLUMN sink_actual_destroy;

ALTER TABLE turnip_market_snapshot DROP COLUMN sink_target_destroy;

ALTER TABLE turnip_market_snapshot DROP COLUMN allocation_ratio_vs_harvest;

ALTER TABLE turnip_market_snapshot DROP COLUMN ambient_supply_ratio;

ALTER TABLE turnip_market_snapshot DROP COLUMN ambient_supply;

UPDATE alembic_version SET version_num='c6f1b8e2d4a0' WHERE alembic_version.version_num = '4e2a9c7b1f34';

-- Running upgrade bac72ce1cfab -> 50d3b5a3a5f3

CREATE TABLE stock_pending_order (
    id SERIAL NOT NULL, 
    user_id VARCHAR NOT NULL, 
    idempotency_key VARCHAR(64) NOT NULL, 
    symbol VARCHAR NOT NULL, 
    action VARCHAR(32) NOT NULL, 
    status VARCHAR(32) NOT NULL, 
    failure_reason VARCHAR(64), 
    request_mode VARCHAR(16) NOT NULL, 
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
    settling_started_at TIMESTAMP WITH TIME ZONE, 
    settlement_attempt_count INTEGER NOT NULL, 
    settlement_worker_id VARCHAR(64), 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    settled_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id), 
    CONSTRAINT ck_stock_pending_order_request_mode_exclusive CHECK ((request_mode = 'amount' AND requested_amount IS NOT NULL AND requested_shares IS NULL) OR (request_mode = 'shares' AND requested_shares IS NOT NULL AND requested_amount IS NULL)), 
    CONSTRAINT uq_stock_pending_order_user_idempotency_key UNIQUE (user_id, idempotency_key)
);

CREATE INDEX ix_stock_pending_order_status_anchor_end ON stock_pending_order (status, anchor_market_minute_end);

CREATE INDEX ix_stock_pending_order_symbol ON stock_pending_order (symbol);

CREATE INDEX ix_stock_pending_order_symbol_status ON stock_pending_order (symbol, status);

CREATE INDEX ix_stock_pending_order_user_id ON stock_pending_order (user_id);

CREATE INDEX ix_stock_pending_order_user_status ON stock_pending_order (user_id, status);

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
    FOREIGN KEY(pending_order_id) REFERENCES stock_pending_order (id), 
    CONSTRAINT uq_stock_position_reservation_pending_order_id UNIQUE (pending_order_id)
);

CREATE INDEX ix_stock_position_reservation_symbol ON stock_position_reservation (symbol);

CREATE INDEX ix_stock_position_reservation_user_id ON stock_position_reservation (user_id);

CREATE INDEX ix_stock_position_reservation_user_symbol_type ON stock_position_reservation (user_id, symbol, position_type);

ALTER TABLE stock_trade_history ADD COLUMN pending_order_id INTEGER;

ALTER TABLE stock_trade_history ADD CONSTRAINT fk_stock_trade_history_pending_order_id FOREIGN KEY(pending_order_id) REFERENCES stock_pending_order (id);

CREATE INDEX ix_stock_trade_history_pending_order_id ON stock_trade_history (pending_order_id);

CREATE UNIQUE INDEX uq_stock_trade_history_pending_order_id_not_null ON stock_trade_history (pending_order_id) WHERE pending_order_id IS NOT NULL;

INSERT INTO alembic_version (version_num) VALUES ('50d3b5a3a5f3') RETURNING alembic_version.version_num;

-- Running upgrade 50d3b5a3a5f3, fde31b9b8e21 -> 7b3f8b9e4c1d

CREATE SCHEMA IF NOT EXISTS stock;

UPDATE alembic_version SET version_num='7b3f8b9e4c1d' WHERE alembic_version.version_num = '50d3b5a3a5f3';

-- Running upgrade 7b3f8b9e4c1d -> 0f2c7a9d8b31

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
);

CREATE TABLE stock.consumer_cursor (
    consumer_name VARCHAR(64) NOT NULL, 
    symbol VARCHAR(32) NOT NULL, 
    last_processed_candle_at TIMESTAMP WITH TIME ZONE, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (consumer_name, symbol)
);

CREATE TABLE stock.producer_heartbeat (
    producer_id VARCHAR(64) NOT NULL, 
    heartbeat_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    last_reconcile_started_at TIMESTAMP WITH TIME ZONE, 
    last_reconcile_finished_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (producer_id)
);

UPDATE alembic_version SET version_num='0f2c7a9d8b31' WHERE alembic_version.version_num = '7b3f8b9e4c1d';

-- Running upgrade 0f2c7a9d8b31 -> 4b1d9e7a6c2f

ALTER TABLE stock.producer_heartbeat ADD COLUMN mode VARCHAR(32);

ALTER TABLE stock.producer_heartbeat ADD COLUMN ws_connected BOOLEAN;

ALTER TABLE stock.producer_heartbeat ADD COLUMN ws_last_connected_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE stock.producer_heartbeat ADD COLUMN ws_last_message_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE stock.producer_heartbeat ADD COLUMN ws_subscription_count INTEGER;

ALTER TABLE stock.producer_heartbeat ADD COLUMN last_targeted_reconcile_started_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE stock.producer_heartbeat ADD COLUMN last_targeted_reconcile_finished_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE stock.producer_heartbeat ADD COLUMN last_targeted_reconcile_symbol_count INTEGER;

ALTER TABLE stock.producer_heartbeat ADD COLUMN last_gap_repair_started_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE stock.producer_heartbeat ADD COLUMN last_gap_repair_finished_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE stock.producer_heartbeat ADD COLUMN last_gap_repair_symbol_count INTEGER;

UPDATE alembic_version SET version_num='4b1d9e7a6c2f' WHERE alembic_version.version_num = '0f2c7a9d8b31';

-- Running upgrade 4b1d9e7a6c2f, c6f1b8e2d4a0 -> 8a9f0d3c1b2e

ALTER TABLE stock_pending_order ADD COLUMN cancel_fee_cash_amount DECIMAL(38, 2);

ALTER TABLE stock_pending_order ADD COLUMN cancel_fee_shares DECIMAL(38, 6);

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
    FOREIGN KEY(pending_order_id) REFERENCES stock_pending_order (id)
);

CREATE INDEX ix_stock_portfolio_adjustment_symbol ON stock_portfolio_adjustment (symbol);

CREATE INDEX ix_stock_portfolio_adjustment_pending_order_id ON stock_portfolio_adjustment (pending_order_id);

CREATE INDEX ix_stock_portfolio_adjustment_user_id ON stock_portfolio_adjustment (user_id);

CREATE INDEX ix_stock_portfolio_adjustment_user_symbol_type ON stock_portfolio_adjustment (user_id, symbol, position_type);

DELETE FROM alembic_version WHERE alembic_version.version_num = 'c6f1b8e2d4a0';

UPDATE alembic_version SET version_num='8a9f0d3c1b2e' WHERE alembic_version.version_num = '4b1d9e7a6c2f';

-- Running upgrade 8a9f0d3c1b2e -> c9b3e7f1a2d4

ALTER TABLE turnip_market_snapshot ADD COLUMN support_bucket_started_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE turnip_market_snapshot ADD COLUMN support_slow_reference_anchor DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_slow_discount DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_sink_capacity_turnips DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_sink_quota_base_notional DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_flow_pressure DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_user_taker_sell_into_sink_notional DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_flow_pressure_ema DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_feedback_target_ratio DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_actual_support_ratio DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_gap DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_gap_ema DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_pressure_integral DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_bid_bias_boost_raw DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_bid_bias_boost DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_budget_boost_raw DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_budget_boost DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN support_integral_decay_mode VARCHAR(32);

UPDATE alembic_version SET version_num='c9b3e7f1a2d4' WHERE alembic_version.version_num = '8a9f0d3c1b2e';

-- Running upgrade c2d3e4f5a6b7 -> 7d1b2c3e4f5a

ALTER TABLE raid_session ADD COLUMN current_seq INTEGER DEFAULT '0' NOT NULL;

LOCK TABLE raid_action_log IN SHARE ROW EXCLUSIVE MODE;
            LOCK TABLE raid_session IN SHARE ROW EXCLUSIVE MODE;;

WITH next_seq AS (
                SELECT
                    s.id AS session_id,
                    COALESCE(MAX(l.seq) + 1, 0) AS current_seq
                FROM raid_session AS s
                LEFT JOIN raid_action_log AS l
                    ON l.session_id = s.id
                GROUP BY s.id
            )
            UPDATE raid_session AS s
            SET current_seq = next_seq.current_seq
            FROM next_seq
            WHERE s.id = next_seq.session_id;

ALTER TABLE raid_session ALTER COLUMN current_seq DROP DEFAULT;

INSERT INTO alembic_version (version_num) VALUES ('7d1b2c3e4f5a') RETURNING alembic_version.version_num;

-- Running upgrade c9b3e7f1a2d4 -> eec5ff3a4c5f

CREATE TABLE raid_risk_control_state (
    user_id VARCHAR(64) NOT NULL, 
    last_evaluated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    precheck_band VARCHAR(16) NOT NULL, 
    effective_shadow_band VARCHAR(16) NOT NULL, 
    starts_6h INTEGER DEFAULT '0' NOT NULL, 
    starts_24h INTEGER DEFAULT '0' NOT NULL, 
    hot_30m_buckets_24h INTEGER DEFAULT '0' NOT NULL, 
    turnstile_passed_at TIMESTAMP WITH TIME ZONE, 
    turnstile_exempt_nonce VARCHAR(128), 
    signals_json JSONB DEFAULT '{}'::jsonb NOT NULL, 
    PRIMARY KEY (user_id)
);

CREATE INDEX ix_raid_session_user_created_at ON raid_session (user_id, created_at);

UPDATE alembic_version SET version_num='eec5ff3a4c5f' WHERE alembic_version.version_num = 'c9b3e7f1a2d4';

-- Running upgrade eec5ff3a4c5f -> 47f3c1d8a2b6

ALTER TABLE raid_session ADD COLUMN settled_player_action_count INTEGER;

ALTER TABLE raid_session ADD COLUMN settled_action_interval_stddev_seconds FLOAT;

UPDATE alembic_version SET version_num='47f3c1d8a2b6' WHERE alembic_version.version_num = 'eec5ff3a4c5f';

-- Running upgrade 47f3c1d8a2b6 -> 9e4b7c1d2a3f

ALTER TABLE raid_session ADD COLUMN risk_snapshot_json JSONB;

UPDATE alembic_version SET version_num='9e4b7c1d2a3f' WHERE alembic_version.version_num = '47f3c1d8a2b6';

-- Running upgrade c2d3e4f5a6b7 -> 20260420_03

CREATE TABLE arena_session (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    mode VARCHAR(20) NOT NULL, 
    status VARCHAR(20) NOT NULL, 
    current_round INTEGER NOT NULL, 
    turn_no INTEGER NOT NULL, 
    round_seed INTEGER NOT NULL, 
    state_json JSONB NOT NULL, 
    result_summary JSONB, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    ended_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_arena_session_user_id ON arena_session (user_id);

INSERT INTO alembic_version (version_num) VALUES ('20260420_03') RETURNING alembic_version.version_num;

-- Running upgrade c9b3e7f1a2d4 -> 4c7f8a1b2d3e

ALTER TABLE raid_warehouse_item ADD COLUMN equipped_slot VARCHAR(32);

-- Intentionally broad: tolerate unexpected historical equipped item types.
        UPDATE raid_warehouse_item
        SET equipped_slot = item_type
        WHERE location = 'equipped'
          AND equipped_slot IS NULL;

ALTER TABLE raid_warehouse_item ADD CONSTRAINT ck_raid_warehouse_item_equipped_slot_required CHECK ((location <> 'equipped') OR (equipped_slot IS NOT NULL));

CREATE UNIQUE INDEX ux_raid_warehouse_item_user_equipped_slot_new ON raid_warehouse_item (user_id, equipped_slot) WHERE location = 'equipped';

DROP INDEX ux_raid_warehouse_item_user_equipped_slot;

ALTER INDEX ux_raid_warehouse_item_user_equipped_slot_new
        RENAME TO ux_raid_warehouse_item_user_equipped_slot;

INSERT INTO alembic_version (version_num) VALUES ('4c7f8a1b2d3e') RETURNING alembic_version.version_num;

-- Running upgrade 4c7f8a1b2d3e -> d8f7c6a5b4e3

DO $$
        BEGIN
            IF EXISTS (
                SELECT item_id
                FROM raid_warehouse_item
                GROUP BY item_id
                HAVING count(*) > 1
            ) THEN
                RAISE EXCEPTION
                    'global duplicate raid warehouse item_id rows must be repaired before enforcing item_id primary key';
            END IF;
        END
        $$;;

ALTER TABLE raid_warehouse_item
        DROP CONSTRAINT raid_warehouse_item_pkey;;

DO $$
        BEGIN
            IF to_regclass('ux_raid_warehouse_item_item_id') IS NOT NULL THEN
                ALTER TABLE raid_warehouse_item
                ADD CONSTRAINT raid_warehouse_item_pkey
                PRIMARY KEY USING INDEX ux_raid_warehouse_item_item_id;
            ELSE
                ALTER TABLE raid_warehouse_item
                ADD CONSTRAINT raid_warehouse_item_pkey
                PRIMARY KEY (item_id);
            END IF;
        END
        $$;;

UPDATE alembic_version SET version_num='d8f7c6a5b4e3' WHERE alembic_version.version_num = '4c7f8a1b2d3e';

-- Running upgrade c9b3e7f1a2d4 -> 202604170145

ALTER TABLE turnip_scenario_stage ADD COLUMN regime_target DECIMAL(38, 6);

ALTER TABLE turnip_scenario_overlay_revision ADD COLUMN scenario_patch_json JSONB DEFAULT '{}' NOT NULL;

ALTER TABLE turnip_scenario_overlay_revision ADD COLUMN entry_patch_json JSONB DEFAULT '{}' NOT NULL;

ALTER TABLE turnip_scenario_overlay_revision ADD COLUMN neutralize_progress_ratio DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN noise_regime_raw_signal DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN scenario_regime_bias DECIMAL(38, 6);

ALTER TABLE turnip_market_snapshot ADD COLUMN effective_regime DECIMAL(38, 6);

INSERT INTO alembic_version (version_num) VALUES ('202604170145') RETURNING alembic_version.version_num;

-- Running upgrade 202604170145, 20260420_03, 202604212029, 4c7f8a1b2d3e, 7d1b2c3e4f5a, 9e4b7c1d2a3f -> 20260424_01

ALTER TABLE currency_rate_state ADD COLUMN macro_anchor_rate NUMERIC(38, 8) DEFAULT '1' NOT NULL;

ALTER TABLE currency_rate_state ADD COLUMN macro_anchor_updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL;

ALTER TABLE currency_rate_state ADD COLUMN short_term_premium NUMERIC(38, 8) DEFAULT '0' NOT NULL;

ALTER TABLE currency_rate_state ADD COLUMN bank_backstop_hour_start TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL;

ALTER TABLE currency_rate_state ADD COLUMN bank_backstop_hour_used NUMERIC(38, 2) DEFAULT '0' NOT NULL;

UPDATE currency_rate_state SET macro_anchor_rate = current_rate;

DELETE FROM alembic_version WHERE alembic_version.version_num = '7d1b2c3e4f5a';

DELETE FROM alembic_version WHERE alembic_version.version_num = '9e4b7c1d2a3f';

DELETE FROM alembic_version WHERE alembic_version.version_num = '202604170145';

DELETE FROM alembic_version WHERE alembic_version.version_num = '20260420_03';

UPDATE alembic_version SET version_num='20260424_01' WHERE alembic_version.version_num = '202604212029';

-- Running upgrade 20260424_01 -> 20260428_01

ALTER TABLE pal ADD COLUMN elite_tier INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE pal ALTER COLUMN elite_tier DROP DEFAULT;

UPDATE alembic_version SET version_num='20260428_01' WHERE alembic_version.version_num = '20260424_01';

-- Running upgrade 20260428_01 -> 20260429_01

COMMIT;

CREATE INDEX CONCURRENTLY IF NOT EXISTS
            ix_wallet_transaction_user_tx_type_cover_amount
            ON wallet_transaction (user_id, tx_type)
            INCLUDE (amount);

CREATE INDEX CONCURRENTLY IF NOT EXISTS
            ix_wallet_transaction_created_at_cover_user_amount
            ON wallet_transaction (created_at)
            INCLUDE (user_id, amount);

CREATE INDEX CONCURRENTLY IF NOT EXISTS
            ix_wallet_transaction_tx_type_created_at_cover_amount
            ON wallet_transaction (tx_type, created_at)
            INCLUDE (amount);

CREATE INDEX CONCURRENTLY IF NOT EXISTS
            ix_pal_overview_stats
            ON pal (user_id)
            INCLUDE (rarity);

CREATE INDEX CONCURRENTLY IF NOT EXISTS
            ix_pal_egg_overview_stats
            ON pal_egg (user_id, status)
            INCLUDE (price_paid, egg_tier);

CREATE INDEX CONCURRENTLY IF NOT EXISTS
            ix_turnip_order_book_lookup
            ON turnip_order (side, status, expires_at, limit_price, created_at);

BEGIN;

UPDATE alembic_version SET version_num='20260429_01' WHERE alembic_version.version_num = '20260428_01';

-- Running upgrade 20260429_01 -> d8c4a91f2b7e

UPDATE wallet
            SET projection_enabled = false,
                allow_negative_balance = false
            WHERE user_id = '__turnip_spot_treasury__';

UPDATE alembic_version SET version_num='d8c4a91f2b7e' WHERE alembic_version.version_num = '20260429_01';

-- Running upgrade d8c4a91f2b7e -> 20260430_01

CREATE TABLE pal_adoption_record (
    id SERIAL NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    pal_id INTEGER NOT NULL, 
    adoption_date DATE NOT NULL, 
    cost BIGINT NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(pal_id) REFERENCES pal (id), 
    FOREIGN KEY(user_id) REFERENCES wallet (user_id)
);

CREATE INDEX ix_pal_adoption_user_date_created ON pal_adoption_record (user_id, adoption_date, created_at);

CREATE INDEX ix_pal_adoption_pal_id ON pal_adoption_record (pal_id);

UPDATE alembic_version SET version_num='20260430_01' WHERE alembic_version.version_num = 'd8c4a91f2b7e';

-- Running upgrade 20260430_01 -> 4891fc442751

COMMIT;

CREATE INDEX CONCURRENTLY ix_pal_adoption_pool_user_id_id ON pal (user_id, id) WHERE pending_gift_id IS NULL AND locked_for_order_id IS NULL;

BEGIN;

UPDATE alembic_version SET version_num='4891fc442751' WHERE alembic_version.version_num = '20260430_01';

-- Running upgrade 4891fc442751 -> 3a9f6c2d8e10

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
);

CREATE INDEX ix_economy_overview_component_snapshot_expires_at ON economy_overview_component_snapshot (expires_at);

UPDATE alembic_version SET version_num='3a9f6c2d8e10' WHERE alembic_version.version_num = '4891fc442751';

-- Running upgrade 3a9f6c2d8e10 -> a4d9e2f1c8b7

CREATE TABLE polls (
    id VARCHAR NOT NULL, 
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
);

CREATE INDEX ix_polls_created_at ON polls (created_at);

CREATE INDEX ix_polls_creator_id ON polls (creator_id);

CREATE INDEX ix_polls_creator_created ON polls (creator_id, created_at);

CREATE INDEX ix_polls_expires_at ON polls (expires_at);

CREATE INDEX ix_polls_room_status_expires ON polls (room_id, status, expires_at);

CREATE INDEX ix_polls_scope_status_expires ON polls (scope, status, expires_at);

CREATE INDEX ix_polls_status ON polls (status);

CREATE TABLE poll_options (
    id VARCHAR NOT NULL, 
    poll_id VARCHAR NOT NULL, 
    position INTEGER NOT NULL, 
    label VARCHAR(120) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(poll_id) REFERENCES polls (id) ON DELETE CASCADE, 
    CONSTRAINT ux_poll_options_poll_position UNIQUE (poll_id, position)
);

CREATE INDEX ix_poll_options_poll_id ON poll_options (poll_id);

CREATE TABLE poll_votes (
    id VARCHAR NOT NULL, 
    poll_id VARCHAR NOT NULL, 
    option_id VARCHAR NOT NULL, 
    voter_user_id VARCHAR(100) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(option_id) REFERENCES poll_options (id) ON DELETE CASCADE, 
    FOREIGN KEY(poll_id) REFERENCES polls (id) ON DELETE CASCADE, 
    CONSTRAINT ux_poll_votes_poll_voter_option UNIQUE (poll_id, voter_user_id, option_id)
);

CREATE INDEX ix_poll_votes_option_id ON poll_votes (option_id);

CREATE INDEX ix_poll_votes_poll_id ON poll_votes (poll_id);

CREATE INDEX ix_poll_votes_voter_user_id ON poll_votes (voter_user_id);

UPDATE alembic_version SET version_num='a4d9e2f1c8b7' WHERE alembic_version.version_num = '3a9f6c2d8e10';

-- Running upgrade a4d9e2f1c8b7 -> 3383fc9b8b83

DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1
                FROM pg_constraint
                WHERE conrelid = 'poll_options'::regclass
                  AND conname = 'ux_poll_options_poll_id_id'
            ) THEN
                ALTER TABLE poll_options
                    ADD CONSTRAINT ux_poll_options_poll_id_id
                    UNIQUE (poll_id, id);
            END IF;
        END $$;;

DO $$
        DECLARE
            old_constraint_name text;
        BEGIN
            FOR old_constraint_name IN
                SELECT con.conname
                FROM pg_constraint con
                WHERE con.contype = 'f'
                  AND con.conrelid = 'poll_votes'::regclass
                  AND con.confrelid = 'poll_options'::regclass
                  AND con.conkey = ARRAY[
                    (
                        SELECT attnum
                        FROM pg_attribute
                        WHERE attrelid = 'poll_votes'::regclass
                          AND attname = 'option_id'
                    )
                  ]::smallint[]
            LOOP
                EXECUTE format(
                    'ALTER TABLE poll_votes DROP CONSTRAINT %I',
                    old_constraint_name
                );
            END LOOP;
        END $$;;

DO $$
        DECLARE
            existing_constraint_name text;
        BEGIN
            SELECT con.conname
            INTO existing_constraint_name
            FROM pg_constraint con
            WHERE con.contype = 'f'
              AND con.conrelid = 'poll_votes'::regclass
              AND con.confrelid = 'poll_options'::regclass
              AND con.conkey = ARRAY[
                (
                    SELECT attnum
                    FROM pg_attribute
                    WHERE attrelid = 'poll_votes'::regclass
                      AND attname = 'poll_id'
                ),
                (
                    SELECT attnum
                    FROM pg_attribute
                    WHERE attrelid = 'poll_votes'::regclass
                      AND attname = 'option_id'
                )
              ]::smallint[]
              AND con.confkey = ARRAY[
                (
                    SELECT attnum
                    FROM pg_attribute
                    WHERE attrelid = 'poll_options'::regclass
                      AND attname = 'poll_id'
                ),
                (
                    SELECT attnum
                    FROM pg_attribute
                    WHERE attrelid = 'poll_options'::regclass
                      AND attname = 'id'
                )
              ]::smallint[]
            LIMIT 1;

            IF existing_constraint_name IS NULL THEN
                ALTER TABLE poll_votes
                    ADD CONSTRAINT fk_poll_votes_poll_option_pair
                    FOREIGN KEY (poll_id, option_id)
                    REFERENCES poll_options (poll_id, id)
                    ON DELETE CASCADE;
            ELSIF existing_constraint_name <> 'fk_poll_votes_poll_option_pair'
                AND NOT EXISTS (
                    SELECT 1
                    FROM pg_constraint
                    WHERE conrelid = 'poll_votes'::regclass
                      AND conname = 'fk_poll_votes_poll_option_pair'
                )
            THEN
                EXECUTE format(
                    'ALTER TABLE poll_votes RENAME CONSTRAINT %I TO fk_poll_votes_poll_option_pair',
                    existing_constraint_name
                );
            END IF;
        END $$;;

UPDATE alembic_version SET version_num='3383fc9b8b83' WHERE alembic_version.version_num = 'a4d9e2f1c8b7';

-- Running upgrade 3383fc9b8b83 -> 9abdf23f0e45

CREATE TABLE undercover_word_pair (
    id VARCHAR NOT NULL, 
    word_a VARCHAR(32) NOT NULL, 
    word_b VARCHAR(32) NOT NULL, 
    canonical_word_a VARCHAR(128) NOT NULL, 
    canonical_word_b VARCHAR(128) NOT NULL, 
    submitter_user_id VARCHAR NOT NULL, 
    is_active BOOLEAN NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    CONSTRAINT ux_undercover_word_pair_canonical UNIQUE (canonical_word_a, canonical_word_b)
);

CREATE INDEX ix_undercover_word_pair_active_created_id ON undercover_word_pair (is_active, created_at, id);

CREATE INDEX ix_undercover_word_pair_submitter_user_id ON undercover_word_pair (submitter_user_id);

UPDATE alembic_version SET version_num='9abdf23f0e45' WHERE alembic_version.version_num = '3383fc9b8b83';

-- Running upgrade 9abdf23f0e45 -> c2f4a0d9b8e1

CREATE TABLE poll_comments (
    id VARCHAR NOT NULL, 
    poll_id VARCHAR NOT NULL, 
    author_id VARCHAR(100) NOT NULL, 
    content VARCHAR(1000) NOT NULL, 
    quote_comment_id VARCHAR, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(poll_id) REFERENCES polls (id) ON DELETE CASCADE, 
    FOREIGN KEY(quote_comment_id) REFERENCES poll_comments (id) ON DELETE SET NULL
);

CREATE INDEX ix_poll_comments_author_created ON poll_comments (author_id, created_at);

CREATE INDEX ix_poll_comments_author_id ON poll_comments (author_id);

CREATE INDEX ix_poll_comments_created_at ON poll_comments (created_at);

CREATE INDEX ix_poll_comments_poll_created ON poll_comments (poll_id, created_at);

CREATE INDEX ix_poll_comments_poll_id ON poll_comments (poll_id);

CREATE INDEX ix_poll_comments_quote_comment_id ON poll_comments (quote_comment_id);

CREATE TABLE poll_comment_reactions (
    id VARCHAR NOT NULL, 
    comment_id VARCHAR NOT NULL, 
    user_id VARCHAR(100) NOT NULL, 
    emoji VARCHAR(16) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(comment_id) REFERENCES poll_comments (id) ON DELETE CASCADE, 
    CONSTRAINT ux_poll_comment_reactions_comment_user_emoji UNIQUE (comment_id, user_id, emoji)
);

CREATE INDEX ix_poll_comment_reactions_comment_id ON poll_comment_reactions (comment_id);

CREATE INDEX ix_poll_comment_reactions_user_id ON poll_comment_reactions (user_id);

UPDATE alembic_version SET version_num='c2f4a0d9b8e1' WHERE alembic_version.version_num = '9abdf23f0e45';

-- Running upgrade 3a9f6c2d8e10 -> 0b7e6c2d4f91

CREATE TABLE farm_weather_hour (
    id SERIAL NOT NULL, 
    starts_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    weather_type VARCHAR(20) NOT NULL, 
    yield_factor DECIMAL(38, 6) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX ix_farm_weather_hour_starts_at ON farm_weather_hour (starts_at);

ALTER TABLE turnip_seed ADD COLUMN weather_score DECIMAL(38, 6) DEFAULT '0' NOT NULL;

ALTER TABLE turnip_seed ADD COLUMN weather_observed_hours DECIMAL(38, 6) DEFAULT '0' NOT NULL;

ALTER TABLE turnip_seed ADD COLUMN last_weather_accounted_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE turnip_seed ADD COLUMN locked_weather_yield_factor DECIMAL(38, 6) DEFAULT '1.000000' NOT NULL;

ALTER TABLE turnip_seed ADD COLUMN batch_yield_factor DECIMAL(38, 6) DEFAULT '1.000000' NOT NULL;

ALTER TABLE turnip_seed ALTER COLUMN weather_score DROP DEFAULT;

ALTER TABLE turnip_seed ALTER COLUMN weather_observed_hours DROP DEFAULT;

ALTER TABLE turnip_seed ALTER COLUMN locked_weather_yield_factor DROP DEFAULT;

ALTER TABLE turnip_seed ALTER COLUMN batch_yield_factor DROP DEFAULT;

INSERT INTO alembic_version (version_num) VALUES ('0b7e6c2d4f91') RETURNING alembic_version.version_num;

-- Running upgrade 0b7e6c2d4f91 -> 20260507_01

CREATE TABLE undercover_event (
    id SERIAL NOT NULL, 
    session_id VARCHAR(64) NOT NULL, 
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
    FOREIGN KEY(session_id) REFERENCES undercover_session (id) ON DELETE CASCADE
);

CREATE INDEX ix_undercover_event_event_type ON undercover_event (event_type);

CREATE UNIQUE INDEX idx_undercover_event_session_seq ON undercover_event (session_id, seq);

CREATE INDEX idx_undercover_event_session_created ON undercover_event (session_id, created_at);

CREATE INDEX idx_undercover_event_public_message_pending ON undercover_event (public_message_status, created_at);

UPDATE alembic_version SET version_num='20260507_01' WHERE alembic_version.version_num = '0b7e6c2d4f91';

-- Running upgrade 20260507_01 -> 7d2c4f9a8b61

ALTER TABLE turnip_seed ADD COLUMN pal_harvest_bonus_score DECIMAL(38, 6) DEFAULT '0' NOT NULL;

ALTER TABLE turnip_seed ADD COLUMN locked_pal_harvest_bonus DECIMAL(38, 6) DEFAULT '0' NOT NULL;

ALTER TABLE turnip_seed ADD COLUMN pal_harvest_bonus_locked BOOLEAN DEFAULT false NOT NULL;

ALTER TABLE turnip_seed ALTER COLUMN pal_harvest_bonus_score DROP DEFAULT;

ALTER TABLE turnip_seed ALTER COLUMN locked_pal_harvest_bonus DROP DEFAULT;

ALTER TABLE turnip_seed ALTER COLUMN pal_harvest_bonus_locked DROP DEFAULT;

UPDATE alembic_version SET version_num='7d2c4f9a8b61' WHERE alembic_version.version_num = '20260507_01';

-- Running upgrade 7d2c4f9a8b61 -> 0c93158d0270

UPDATE turnip_seed
        SET
            quantity = CEIL(quantity / 3::numeric),
            seed_price = ROUND(seed_price * 3, 2);

UPDATE alembic_version SET version_num='0c93158d0270' WHERE alembic_version.version_num = '7d2c4f9a8b61';

-- Running upgrade 3a9f6c2d8e10 -> 7c8f2d1a9e34

CREATE SCHEMA IF NOT EXISTS games;

CREATE TABLE games.ponzi_sessions (
    id VARCHAR NOT NULL, 
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
);

CREATE INDEX ix_games_ponzi_sessions_room_id ON games.ponzi_sessions (room_id);

CREATE INDEX idx_ponzi_sessions_room_status ON games.ponzi_sessions (room_id, status);

CREATE INDEX idx_ponzi_sessions_updated_at ON games.ponzi_sessions (updated_at);

CREATE TABLE games.ponzi_events (
    id SERIAL NOT NULL, 
    session_id VARCHAR(64) NOT NULL, 
    seq INTEGER NOT NULL, 
    event_type VARCHAR(64) NOT NULL, 
    actor_user_id VARCHAR(128), 
    public_payload JSONB NOT NULL, 
    private_payload JSONB NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(session_id) REFERENCES games.ponzi_sessions (id) ON DELETE CASCADE
);

CREATE INDEX ix_games_ponzi_events_event_type ON games.ponzi_events (event_type);

CREATE UNIQUE INDEX idx_ponzi_events_session_seq ON games.ponzi_events (session_id, seq);

CREATE INDEX idx_ponzi_events_session_created ON games.ponzi_events (session_id, created_at);

INSERT INTO alembic_version (version_num) VALUES ('7c8f2d1a9e34') RETURNING alembic_version.version_num;

-- Running upgrade 7c8f2d1a9e34 -> af7b3c6d9e10

ALTER TABLE games.ponzi_events ADD COLUMN public_message_status VARCHAR(16) DEFAULT 'skipped' NOT NULL;

ALTER TABLE games.ponzi_events ADD COLUMN public_message_attempts INTEGER DEFAULT '0' NOT NULL;

ALTER TABLE games.ponzi_events ADD COLUMN public_message_sent_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE games.ponzi_events ADD COLUMN public_message_error TEXT;

CREATE INDEX idx_ponzi_events_public_message_pending ON games.ponzi_events (public_message_status, created_at);

ALTER TABLE games.ponzi_events ALTER COLUMN public_message_status DROP DEFAULT;

ALTER TABLE games.ponzi_events ALTER COLUMN public_message_attempts DROP DEFAULT;

UPDATE alembic_version SET version_num='af7b3c6d9e10' WHERE alembic_version.version_num = '7c8f2d1a9e34';

-- Running upgrade 0c93158d0270, af7b3c6d9e10, c2f4a0d9b8e1, d8f7c6a5b4e3 -> b4d9e2a8c6f1

CREATE TABLE blackjack_sessions (
    id VARCHAR NOT NULL, 
    user_id VARCHAR NOT NULL, 
    status VARCHAR NOT NULL, 
    state_json JSONB NOT NULL, 
    result_json JSONB, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    finished_at TIMESTAMP WITH TIME ZONE, 
    PRIMARY KEY (id)
);

CREATE INDEX ix_blackjack_sessions_created_at ON blackjack_sessions (created_at);

CREATE INDEX ix_blackjack_sessions_status ON blackjack_sessions (status);

CREATE INDEX ix_blackjack_sessions_user_id ON blackjack_sessions (user_id);

CREATE INDEX ix_blackjack_sessions_user_status_created ON blackjack_sessions (user_id, status, created_at);

CREATE UNIQUE INDEX ux_blackjack_sessions_user_active ON blackjack_sessions (user_id) WHERE status = 'playing';

DELETE FROM alembic_version WHERE alembic_version.version_num = '0c93158d0270';

DELETE FROM alembic_version WHERE alembic_version.version_num = 'af7b3c6d9e10';

DELETE FROM alembic_version WHERE alembic_version.version_num = 'c2f4a0d9b8e1';

UPDATE alembic_version SET version_num='b4d9e2a8c6f1' WHERE alembic_version.version_num = 'd8f7c6a5b4e3';

-- Running upgrade 0c93158d0270 -> 9284e108daef

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
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX ix_discord_account_binding_discord_user_id ON discord_account_binding (discord_user_id);

CREATE UNIQUE INDEX ix_discord_account_binding_user_id ON discord_account_binding (user_id);

INSERT INTO alembic_version (version_num) VALUES ('9284e108daef') RETURNING alembic_version.version_num;

-- Running upgrade 20260507_01 -> 6a4f2b8c9d13

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
    PRIMARY KEY (invocation_id)
);

CREATE INDEX ix_external_cmd_inv_config ON external_command_invocation (config_id);

CREATE INDEX ix_external_cmd_inv_room ON external_command_invocation (room_id);

CREATE INDEX ix_external_cmd_inv_sender ON external_command_invocation (sender_id);

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
);

CREATE INDEX ix_external_command_effect_invocation_id ON external_command_effect (invocation_id);

CREATE UNIQUE INDEX ux_external_cmd_effect_invocation_index ON external_command_effect (invocation_id, effect_index);

INSERT INTO alembic_version (version_num) VALUES ('6a4f2b8c9d13') RETURNING alembic_version.version_num;

-- Running upgrade 6a4f2b8c9d13 -> 88adcbabc481

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
    FOREIGN KEY(invocation_id) REFERENCES external_command_invocation (invocation_id)
);

CREATE INDEX ix_external_cmd_session_config ON external_command_session (config_id);

CREATE INDEX ix_external_cmd_session_room ON external_command_session (room_id);

CREATE INDEX ix_external_cmd_session_status ON external_command_session (status);

UPDATE alembic_version SET version_num='88adcbabc481' WHERE alembic_version.version_num = '6a4f2b8c9d13';

-- Running upgrade 88adcbabc481, 9284e108daef, b4d9e2a8c6f1 -> 4f88d4f2b0ac

CREATE SCHEMA IF NOT EXISTS games;

CREATE TABLE games.lottery_draw (
    id SERIAL NOT NULL, 
    draw_date DATE NOT NULL, 
    draw_at TIMESTAMP WITH TIME ZONE, 
    sales_close_at TIMESTAMP WITH TIME ZONE, 
    status VARCHAR NOT NULL, 
    ticket_price NUMERIC(24, 2) NOT NULL, 
    prize_contribution_rate NUMERIC(24, 2) NOT NULL, 
    house_retention_rate NUMERIC(24, 2) NOT NULL, 
    pool_before_sales NUMERIC(24, 2) NOT NULL, 
    sales_amount NUMERIC(24, 2) NOT NULL, 
    prize_contribution NUMERIC(24, 2) NOT NULL, 
    house_retention NUMERIC(24, 2) NOT NULL, 
    pool_before_draw NUMERIC(24, 2) NOT NULL, 
    allocated_prize_amount NUMERIC(24, 2) NOT NULL, 
    paid_prize_amount NUMERIC(24, 2) NOT NULL, 
    carryover_amount NUMERIC(24, 2) NOT NULL, 
    winning_numbers JSONB, 
    draw_hash VARCHAR(64), 
    previous_draw_hash VARCHAR(64), 
    algorithm_version VARCHAR(64), 
    transcript_json JSONB, 
    failure_code VARCHAR(64), 
    failure_message VARCHAR, 
    notifications_sent_at TIMESTAMP WITH TIME ZONE, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    CONSTRAINT uq_lottery_draw_draw_date UNIQUE (draw_date)
);

CREATE INDEX ix_lottery_draw_draw_at ON games.lottery_draw (draw_at);

CREATE INDEX ix_lottery_draw_status ON games.lottery_draw (status);

CREATE INDEX ix_lottery_draw_status_draw_at ON games.lottery_draw (status, draw_at);

CREATE TABLE games.lottery_ticket (
    id SERIAL NOT NULL, 
    draw_id INTEGER NOT NULL, 
    user_id VARCHAR NOT NULL, 
    numbers JSONB NOT NULL, 
    multiplier INTEGER NOT NULL, 
    unit_price NUMERIC(24, 2) NOT NULL, 
    total_price NUMERIC(24, 2) NOT NULL, 
    status VARCHAR NOT NULL, 
    wallet_transaction_id INTEGER, 
    purchased_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    match_count INTEGER, 
    prize_tier VARCHAR(16), 
    idempotency_key VARCHAR(128) NOT NULL, 
    line_index INTEGER NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(draw_id) REFERENCES games.lottery_draw (id), 
    FOREIGN KEY(wallet_transaction_id) REFERENCES wallet_transaction (id), 
    CONSTRAINT uq_lottery_ticket_user_idempotency_line UNIQUE (user_id, idempotency_key, line_index)
);

CREATE INDEX ix_lottery_ticket_draw_id ON games.lottery_ticket (draw_id);

CREATE INDEX ix_lottery_ticket_draw_status ON games.lottery_ticket (draw_id, status);

CREATE INDEX ix_lottery_ticket_draw_user_purchased ON games.lottery_ticket (draw_id, user_id, purchased_at);

CREATE INDEX ix_lottery_ticket_status ON games.lottery_ticket (status);

CREATE INDEX ix_lottery_ticket_user_id ON games.lottery_ticket (user_id);

CREATE TABLE games.lottery_payout (
    id SERIAL NOT NULL, 
    draw_id INTEGER NOT NULL, 
    ticket_id INTEGER NOT NULL, 
    user_id VARCHAR NOT NULL, 
    prize_tier VARCHAR(16) NOT NULL, 
    gross_amount NUMERIC(24, 2) NOT NULL, 
    tax_amount NUMERIC(24, 2) NOT NULL, 
    net_amount NUMERIC(24, 2) NOT NULL, 
    wallet_transaction_id INTEGER, 
    tax_transaction_id INTEGER, 
    reference_id VARCHAR(128) NOT NULL, 
    created_at TIMESTAMP WITH TIME ZONE NOT NULL, 
    PRIMARY KEY (id), 
    FOREIGN KEY(draw_id) REFERENCES games.lottery_draw (id), 
    FOREIGN KEY(tax_transaction_id) REFERENCES wallet_transaction (id), 
    FOREIGN KEY(ticket_id) REFERENCES games.lottery_ticket (id), 
    FOREIGN KEY(wallet_transaction_id) REFERENCES wallet_transaction (id), 
    CONSTRAINT uq_lottery_payout_ticket_id UNIQUE (ticket_id)
);

CREATE INDEX ix_lottery_payout_draw_user ON games.lottery_payout (draw_id, user_id);

CREATE INDEX ix_lottery_payout_draw_id ON games.lottery_payout (draw_id);

CREATE INDEX ix_lottery_payout_ticket_id ON games.lottery_payout (ticket_id);

CREATE INDEX ix_lottery_payout_user_id ON games.lottery_payout (user_id);

DELETE FROM alembic_version WHERE alembic_version.version_num = '88adcbabc481';

DELETE FROM alembic_version WHERE alembic_version.version_num = '9284e108daef';

UPDATE alembic_version SET version_num='4f88d4f2b0ac' WHERE alembic_version.version_num = 'b4d9e2a8c6f1';

-- Running upgrade 4f88d4f2b0ac -> 8d71b6c3e2fa

ALTER TABLE games.lottery_ticket DROP CONSTRAINT uq_lottery_ticket_user_idempotency_line;

ALTER TABLE games.lottery_ticket ADD CONSTRAINT uq_lottery_ticket_draw_user_idempotency_line UNIQUE (draw_id, user_id, idempotency_key, line_index);

UPDATE alembic_version SET version_num='8d71b6c3e2fa' WHERE alembic_version.version_num = '4f88d4f2b0ac';

-- Running upgrade 8d71b6c3e2fa -> a29f4d83c1e5

ALTER TABLE games.lottery_ticket
        ALTER COLUMN numbers TYPE INTEGER[]
        USING translate(numbers::text, '[]', '{}')::INTEGER[];

ALTER TABLE games.lottery_draw
        ALTER COLUMN winning_numbers TYPE INTEGER[]
        USING CASE
            WHEN winning_numbers IS NULL OR winning_numbers::text = 'null' THEN NULL
            ELSE translate(winning_numbers::text, '[]', '{}')::INTEGER[]
        END;

UPDATE alembic_version SET version_num='a29f4d83c1e5' WHERE alembic_version.version_num = '8d71b6c3e2fa';

-- Running upgrade a29f4d83c1e5 -> c0d5e8a7b9f1

ALTER TABLE games.lottery_draw ADD COLUMN settlement_stats_json JSONB;

WITH draw_stats AS (
            SELECT
                draw.id AS draw_id,
                COUNT(ticket.id)::integer AS ticket_count,
                COALESCE(SUM(ticket.multiplier), 0)::integer AS ticket_unit_count,
                COUNT(payout.id)::integer AS winner_count,
                COALESCE(SUM(ticket.multiplier) FILTER (WHERE payout.id IS NOT NULL), 0)::integer
                    AS winner_unit_count
            FROM games.lottery_draw AS draw
            LEFT JOIN games.lottery_ticket AS ticket
                ON ticket.draw_id = draw.id
                AND ticket.status = 'active'
            LEFT JOIN games.lottery_payout AS payout
                ON payout.ticket_id = ticket.id
                AND payout.gross_amount > 0
            WHERE draw.status = 'settled'
            GROUP BY draw.id
        ),
        tier_rows AS (
            SELECT
                payout.draw_id,
                payout.prize_tier,
                COUNT(payout.id)::integer AS ticket_count,
                COALESCE(SUM(ticket.multiplier), 0)::integer AS unit_count,
                TO_CHAR(COALESCE(SUM(payout.gross_amount), 0), 'FM999999999999999990.00')
                    AS gross_amount
            FROM games.lottery_payout AS payout
            JOIN games.lottery_ticket AS ticket
                ON ticket.id = payout.ticket_id
            WHERE payout.gross_amount > 0
            GROUP BY payout.draw_id, payout.prize_tier
        ),
        tier_stats AS (
            SELECT
                tier_rows.draw_id,
                JSONB_OBJECT_AGG(
                    tier_rows.prize_tier,
                    JSONB_BUILD_OBJECT(
                        'ticket_count', tier_rows.ticket_count,
                        'unit_count', tier_rows.unit_count,
                        'gross_amount', tier_rows.gross_amount
                    )
                ) AS value
            FROM tier_rows
            GROUP BY tier_rows.draw_id
        )
        UPDATE games.lottery_draw AS draw
        SET settlement_stats_json = JSONB_BUILD_OBJECT(
            'ticket_count', draw_stats.ticket_count,
            'ticket_unit_count', draw_stats.ticket_unit_count,
            'winner_count', draw_stats.winner_count,
            'winner_unit_count', draw_stats.winner_unit_count,
            'tier_stats', COALESCE(tier_stats.value, '{}'::jsonb)
        )
        FROM draw_stats
        LEFT JOIN tier_stats ON tier_stats.draw_id = draw_stats.draw_id
        WHERE draw.id = draw_stats.draw_id;

UPDATE alembic_version SET version_num='c0d5e8a7b9f1' WHERE alembic_version.version_num = 'a29f4d83c1e5';

-- Running upgrade c0d5e8a7b9f1 -> f7a2c9d4e6b1

ALTER TABLE games.lottery_draw ADD COLUMN ticket_count INTEGER DEFAULT 0 NOT NULL;

ALTER TABLE games.lottery_draw ADD COLUMN ticket_unit_count INTEGER DEFAULT 0 NOT NULL;

WITH ticket_sales AS (
            SELECT
                ticket.draw_id,
                COUNT(ticket.id)::integer AS ticket_count,
                COALESCE(SUM(ticket.multiplier), 0)::integer AS ticket_unit_count,
                COALESCE(SUM(ticket.total_price), 0)::numeric(24, 2) AS sales_amount
            FROM games.lottery_ticket AS ticket
            WHERE ticket.status = 'active'
            GROUP BY ticket.draw_id
        )
        UPDATE games.lottery_draw AS draw
        SET
            ticket_count = ticket_sales.ticket_count,
            ticket_unit_count = ticket_sales.ticket_unit_count,
            sales_amount = CASE
                WHEN draw.status IN ('open', 'sales_closed', 'drawing')
                    THEN ticket_sales.sales_amount
                ELSE draw.sales_amount
            END,
            prize_contribution = CASE
                WHEN draw.status IN ('open', 'sales_closed', 'drawing')
                    THEN ROUND(ticket_sales.sales_amount * draw.prize_contribution_rate, 2)
                ELSE draw.prize_contribution
            END,
            house_retention = CASE
                WHEN draw.status IN ('open', 'sales_closed', 'drawing')
                    THEN ROUND(
                        ticket_sales.sales_amount
                        - ROUND(ticket_sales.sales_amount * draw.prize_contribution_rate, 2),
                        2
                    )
                ELSE draw.house_retention
            END,
            pool_before_draw = CASE
                WHEN draw.status IN ('open', 'sales_closed', 'drawing')
                    THEN ROUND(
                        draw.pool_before_sales
                        + ROUND(ticket_sales.sales_amount * draw.prize_contribution_rate, 2),
                        2
                    )
                ELSE draw.pool_before_draw
            END
        FROM ticket_sales
        WHERE draw.id = ticket_sales.draw_id;

UPDATE alembic_version SET version_num='f7a2c9d4e6b1' WHERE alembic_version.version_num = 'c0d5e8a7b9f1';

-- Running upgrade f7a2c9d4e6b1 -> 42801c7412ef

ALTER TABLE wallet ADD COLUMN snapshot_balance DECIMAL(38, 2) DEFAULT '0' NOT NULL;

ALTER TABLE wallet ADD COLUMN snapshot_escrow_balance DECIMAL(38, 2) DEFAULT '0' NOT NULL;

ALTER TABLE wallet ADD COLUMN snapshot_tx_id BIGINT DEFAULT '0' NOT NULL;

CREATE INDEX ix_wallet_transaction_user_id_id_snapshot_tail ON wallet_transaction (user_id, id) INCLUDE (amount, escrow_delta);

UPDATE alembic_version SET version_num='42801c7412ef' WHERE alembic_version.version_num = 'f7a2c9d4e6b1';

-- Running upgrade 42801c7412ef -> 9d4e7f2a6b8c

ALTER TABLE games.lottery_draw ADD COLUMN ticket_commitment_input TEXT;

UPDATE alembic_version SET version_num='9d4e7f2a6b8c' WHERE alembic_version.version_num = '42801c7412ef';

-- Running upgrade 9d4e7f2a6b8c -> 3934b0a4f74b

ALTER TABLE turnip_inventory ADD COLUMN pending_gift_id UUID;

CREATE INDEX ix_turnip_inventory_pending_gift_id ON turnip_inventory (pending_gift_id);

ALTER TABLE raid_warehouse_item ADD COLUMN pending_gift_id UUID;

CREATE INDEX ix_raid_warehouse_item_pending_gift_id ON raid_warehouse_item (pending_gift_id);

UPDATE raid_warehouse_item
        SET pending_gift_id = CAST(SUBSTRING(user_id FROM 10) AS UUID),
            user_id = p.from_user_id
        FROM pending_gift p
        WHERE raid_warehouse_item.user_id LIKE '__gift__:%'
          AND CAST(SUBSTRING(raid_warehouse_item.user_id FROM 10) AS UUID) = p.gift_id;

UPDATE turnip_inventory
        SET pending_gift_id = CAST(SUBSTRING(user_id FROM 10) AS UUID),
            user_id = p.from_user_id
        FROM pending_gift p
        WHERE turnip_inventory.user_id LIKE '__gift__:%'
          AND CAST(SUBSTRING(turnip_inventory.user_id FROM 10) AS UUID) = p.gift_id;

DELETE FROM wallet WHERE user_id LIKE '__gift__:%';

UPDATE alembic_version SET version_num='3934b0a4f74b' WHERE alembic_version.version_num = '9d4e7f2a6b8c';

-- Running upgrade 3934b0a4f74b -> 444309409dcf

LOCK TABLE "arena_session" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "auth_session" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "battle_records" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "clearing_batch" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "clearing_batch_item" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "clearing_instruction" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "crop_insurance_policy" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "currency_exchange_order" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "discord_account_binding" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "dzmm_account" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "external_command_invocation" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "external_command_session" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "futures_order" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "futures_order_fill" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "futures_position" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "futures_transaction" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "game_currency_account" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "game_currency_transaction" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "games"."lottery_payout" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "games"."lottery_ticket" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "games"."ponzi_events" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "idempotency_record" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "land" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "land_assignment" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "market_order" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "oidc_refresh_token" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "pal" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "pal_adoption_record" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "pal_egg" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "partner_client" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "partner_managed_account" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "partner_refresh_token" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "payment_intent" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "pending_gift" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "poll_comment_reactions" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "poll_votes" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "private_rooms" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "raid_map_progress" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "raid_profile" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "raid_risk_control_state" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "raid_session" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "raid_warehouse_item" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "red_envelope" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "red_envelope_claim" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "resource_production" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "season_settlement" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "stock_account" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "stock_pending_order" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "stock_portfolio" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "stock_portfolio_adjustment" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "stock_position_reservation" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "stock_trade_history" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "stock_trigger" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "strand_object" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "turnip_inventory" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "turnip_order" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "turnip_seed" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "turnip_transaction" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "undercover_event" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "undercover_word_pair" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "user_achievement" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "user_achievement_progress" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "user_credential" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "user_history" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "user_item" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "user_notification" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "user_passkey" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "users" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "wallet" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "wallet_transaction" IN ACCESS EXCLUSIVE MODE;

LOCK TABLE "webhook_delivery" IN ACCESS EXCLUSIVE MODE;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__futures_treasury__', '__futures_treasury__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__futures_mm_treasury__', '__futures_mm_treasury__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__futures_insurance_fund__', '__futures_insurance_fund__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__futures_hedge_treasury__', '__futures_hedge_treasury__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__raid_exchange_treasury__', '__raid_exchange_treasury__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__turnip_amm_treasury__', '__turnip_amm_treasury__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__turnip_spot_treasury__', '__turnip_spot_treasury__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__insurance_treasury__', '__insurance_treasury__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__wallet_adjustment_offset__', '__wallet_adjustment_offset__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__lottery_treasury__', '__lottery_treasury__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__futures_hedge__', '__futures_hedge__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

INSERT INTO users (user_id, full_name, created_at, updated_at, message_count, deleted_count, recalled_count) VALUES ('__futures_mm__', '__futures_mm__', now(), now(), 0, 0, 0) ON CONFLICT (user_id) DO NOTHING;

ALTER TABLE crop_insurance_policy DROP CONSTRAINT crop_insurance_policy_user_id_fkey;

ALTER TABLE crop_insurance_policy ADD CONSTRAINT crop_insurance_policy_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE land DROP CONSTRAINT land_user_id_fkey;

ALTER TABLE land ADD CONSTRAINT land_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE land_assignment DROP CONSTRAINT land_assignment_user_id_fkey;

ALTER TABLE land_assignment ADD CONSTRAINT land_assignment_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE market_order DROP CONSTRAINT market_order_user_id_fkey;

ALTER TABLE market_order ADD CONSTRAINT market_order_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE pal DROP CONSTRAINT pal_user_id_fkey;

ALTER TABLE pal ADD CONSTRAINT pal_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE pal_adoption_record DROP CONSTRAINT pal_adoption_record_user_id_fkey;

ALTER TABLE pal_adoption_record ADD CONSTRAINT pal_adoption_record_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE pal_egg DROP CONSTRAINT pal_egg_user_id_fkey;

ALTER TABLE pal_egg ADD CONSTRAINT pal_egg_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE resource_production DROP CONSTRAINT resource_production_user_id_fkey;

ALTER TABLE resource_production ADD CONSTRAINT resource_production_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE turnip_inventory DROP CONSTRAINT turnip_inventory_user_id_fkey;

ALTER TABLE turnip_inventory ADD CONSTRAINT turnip_inventory_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE turnip_order DROP CONSTRAINT turnip_order_user_id_fkey;

ALTER TABLE turnip_order ADD CONSTRAINT turnip_order_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE turnip_seed DROP CONSTRAINT turnip_seed_user_id_fkey;

ALTER TABLE turnip_seed ADD CONSTRAINT turnip_seed_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE turnip_transaction DROP CONSTRAINT turnip_transaction_user_id_fkey;

ALTER TABLE turnip_transaction ADD CONSTRAINT turnip_transaction_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE user_item DROP CONSTRAINT user_item_user_id_fkey;

ALTER TABLE user_item ADD CONSTRAINT user_item_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE wallet_transaction DROP CONSTRAINT wallet_transaction_user_id_fkey;

ALTER TABLE wallet_transaction ADD CONSTRAINT wallet_transaction_user_id_fkey FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE pending_gift DROP CONSTRAINT pending_gift_from_user_id_fkey;

ALTER TABLE pending_gift ADD CONSTRAINT pending_gift_from_user_id_fkey FOREIGN KEY(from_user_id) REFERENCES users (user_id);

ALTER TABLE pending_gift DROP CONSTRAINT pending_gift_to_user_id_fkey;

ALTER TABLE pending_gift ADD CONSTRAINT pending_gift_to_user_id_fkey FOREIGN KEY(to_user_id) REFERENCES users (user_id);

ALTER TABLE partner_managed_account ADD CONSTRAINT fk_partner_managed_account_owner_user_id FOREIGN KEY(owner_user_id) REFERENCES users (user_id);

ALTER TABLE partner_managed_account ADD CONSTRAINT fk_partner_managed_account_managed_user_id FOREIGN KEY(managed_user_id) REFERENCES users (user_id);

ALTER TABLE arena_session ADD CONSTRAINT fk_arena_session_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE auth_session ADD CONSTRAINT fk_auth_session_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE battle_records ADD CONSTRAINT fk_battle_records_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE clearing_batch ADD CONSTRAINT fk_clearing_batch_partner_user_id_users FOREIGN KEY(partner_user_id) REFERENCES users (user_id);

ALTER TABLE clearing_batch_item ADD CONSTRAINT fk_clearing_batch_item_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE clearing_instruction ADD CONSTRAINT fk_clearing_instruction_partner_user_id_users FOREIGN KEY(partner_user_id) REFERENCES users (user_id);

ALTER TABLE clearing_instruction ADD CONSTRAINT fk_clearing_instruction_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE currency_exchange_order ADD CONSTRAINT fk_currency_exchange_order_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE discord_account_binding ADD CONSTRAINT fk_discord_account_binding_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE dzmm_account ADD CONSTRAINT fk_dzmm_account_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE external_command_invocation ADD CONSTRAINT fk_external_command_invocation_sender_id_users FOREIGN KEY(sender_id) REFERENCES users (user_id);

ALTER TABLE external_command_session ADD CONSTRAINT fk_external_command_session_sender_id_users FOREIGN KEY(sender_id) REFERENCES users (user_id);

ALTER TABLE futures_order ADD CONSTRAINT fk_futures_order_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE futures_order_fill ADD CONSTRAINT fk_futures_order_fill_buyer_id_users FOREIGN KEY(buyer_id) REFERENCES users (user_id);

ALTER TABLE futures_order_fill ADD CONSTRAINT fk_futures_order_fill_seller_id_users FOREIGN KEY(seller_id) REFERENCES users (user_id);

ALTER TABLE futures_position ADD CONSTRAINT fk_futures_position_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE futures_transaction ADD CONSTRAINT fk_futures_transaction_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE game_currency_account ADD CONSTRAINT fk_game_currency_account_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE game_currency_transaction ADD CONSTRAINT fk_game_currency_transaction_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE idempotency_record ADD CONSTRAINT fk_idempotency_record_partner_user_id_users FOREIGN KEY(partner_user_id) REFERENCES users (user_id);

ALTER TABLE oidc_refresh_token ADD CONSTRAINT fk_oidc_refresh_token_end_user_id_users FOREIGN KEY(end_user_id) REFERENCES users (user_id);

ALTER TABLE oidc_refresh_token ADD CONSTRAINT fk_oidc_refresh_token_partner_user_id_users FOREIGN KEY(partner_user_id) REFERENCES users (user_id);

ALTER TABLE partner_client ADD CONSTRAINT fk_partner_client_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE partner_managed_account ADD CONSTRAINT fk_partner_managed_account_created_by_user_id_users FOREIGN KEY(created_by_user_id) REFERENCES users (user_id);

ALTER TABLE partner_managed_account ADD CONSTRAINT fk_partner_managed_account_updated_by_user_id_users FOREIGN KEY(updated_by_user_id) REFERENCES users (user_id);

ALTER TABLE partner_refresh_token ADD CONSTRAINT fk_partner_refresh_token_partner_user_id_users FOREIGN KEY(partner_user_id) REFERENCES users (user_id);

ALTER TABLE payment_intent ADD CONSTRAINT fk_payment_intent_partner_user_id_users FOREIGN KEY(partner_user_id) REFERENCES users (user_id);

ALTER TABLE payment_intent ADD CONSTRAINT fk_payment_intent_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE poll_comment_reactions ADD CONSTRAINT fk_poll_comment_reactions_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE poll_votes ADD CONSTRAINT fk_poll_votes_voter_user_id_users FOREIGN KEY(voter_user_id) REFERENCES users (user_id);

ALTER TABLE private_rooms ADD CONSTRAINT fk_private_rooms_bot_user_id_users FOREIGN KEY(bot_user_id) REFERENCES users (user_id);

ALTER TABLE private_rooms ADD CONSTRAINT fk_private_rooms_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE raid_map_progress ADD CONSTRAINT fk_raid_map_progress_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE raid_profile ADD CONSTRAINT fk_raid_profile_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE raid_risk_control_state ADD CONSTRAINT fk_raid_risk_control_state_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE raid_session ADD CONSTRAINT fk_raid_session_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE raid_warehouse_item ADD CONSTRAINT fk_raid_warehouse_item_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE red_envelope ADD CONSTRAINT fk_red_envelope_sender_id_users FOREIGN KEY(sender_id) REFERENCES users (user_id);

ALTER TABLE red_envelope_claim ADD CONSTRAINT fk_red_envelope_claim_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE season_settlement ADD CONSTRAINT fk_season_settlement_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE stock_account ADD CONSTRAINT fk_stock_account_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE stock_pending_order ADD CONSTRAINT fk_stock_pending_order_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE stock_portfolio ADD CONSTRAINT fk_stock_portfolio_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE stock_portfolio_adjustment ADD CONSTRAINT fk_stock_portfolio_adjustment_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE stock_position_reservation ADD CONSTRAINT fk_stock_position_reservation_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE stock_trade_history ADD CONSTRAINT fk_stock_trade_history_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE stock_trigger ADD CONSTRAINT fk_stock_trigger_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE strand_object ADD CONSTRAINT fk_strand_object_owner_user_id_users FOREIGN KEY(owner_user_id) REFERENCES users (user_id);

ALTER TABLE undercover_event ADD CONSTRAINT fk_undercover_event_actor_user_id_users FOREIGN KEY(actor_user_id) REFERENCES users (user_id);

ALTER TABLE undercover_word_pair ADD CONSTRAINT fk_undercover_word_pair_submitter_user_id_users FOREIGN KEY(submitter_user_id) REFERENCES users (user_id);

ALTER TABLE user_achievement ADD CONSTRAINT fk_user_achievement_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE user_achievement_progress ADD CONSTRAINT fk_user_achievement_progress_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE user_credential ADD CONSTRAINT fk_user_credential_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE user_history ADD CONSTRAINT fk_user_history_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE user_notification ADD CONSTRAINT fk_user_notification_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE user_passkey ADD CONSTRAINT fk_user_passkey_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE wallet ADD CONSTRAINT fk_wallet_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE webhook_delivery ADD CONSTRAINT fk_webhook_delivery_partner_user_id_users FOREIGN KEY(partner_user_id) REFERENCES users (user_id);

ALTER TABLE games.lottery_payout ADD CONSTRAINT fk_lottery_payout_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE games.lottery_ticket ADD CONSTRAINT fk_lottery_ticket_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

ALTER TABLE games.ponzi_events ADD CONSTRAINT fk_ponzi_events_actor_user_id_users FOREIGN KEY(actor_user_id) REFERENCES users (user_id);

UPDATE alembic_version SET version_num='444309409dcf' WHERE alembic_version.version_num = '3934b0a4f74b';

-- Running upgrade 444309409dcf -> 6e4b2f7a1c9d

CREATE OR REPLACE FUNCTION notify_wallet_transaction_inserted()
        RETURNS trigger AS $$
        BEGIN
            PERFORM pg_notify('wallet_transaction_inserted', NEW.id::text);
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;;

DROP TRIGGER IF EXISTS wallet_transaction_inserted_trigger ON wallet_transaction;

CREATE TRIGGER wallet_transaction_inserted_trigger
        AFTER INSERT ON wallet_transaction
        FOR EACH ROW
        EXECUTE FUNCTION notify_wallet_transaction_inserted();;

DROP TRIGGER IF EXISTS wallet_tx_group_guard ON wallet_transaction;

CREATE CONSTRAINT TRIGGER wallet_tx_group_guard
        AFTER INSERT OR UPDATE OF user_id, amount, escrow_delta, counterparty_id, tx_group_id OR DELETE ON wallet_transaction
        DEFERRABLE INITIALLY DEFERRED
        FOR EACH ROW
        EXECUTE FUNCTION wallet_guard_enforce_tx_group();

DROP TRIGGER IF EXISTS wallet_tx_wallet_guard ON wallet_transaction;

CREATE CONSTRAINT TRIGGER wallet_tx_wallet_guard
        AFTER INSERT OR UPDATE OF user_id, amount, escrow_delta, counterparty_id, tx_group_id OR DELETE ON wallet_transaction
        DEFERRABLE INITIALLY DEFERRED
        FOR EACH ROW
        EXECUTE FUNCTION wallet_guard_enforce_tx_wallet();

DROP TRIGGER IF EXISTS wallet_row_wallet_guard ON wallet;

CREATE CONSTRAINT TRIGGER wallet_row_wallet_guard
        AFTER INSERT OR UPDATE OF user_id, balance, escrow_balance, projection_enabled, allow_negative_balance OR DELETE ON wallet
        DEFERRABLE INITIALLY DEFERRED
        FOR EACH ROW
        EXECUTE FUNCTION wallet_guard_enforce_wallet();

UPDATE alembic_version SET version_num='6e4b2f7a1c9d' WHERE alembic_version.version_num = '444309409dcf';

-- Running upgrade 6e4b2f7a1c9d -> 5ab3e4f257a8

ALTER TABLE games.lottery_draw ALTER COLUMN ticket_price TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_draw ALTER COLUMN prize_contribution_rate TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_draw ALTER COLUMN house_retention_rate TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_draw ALTER COLUMN pool_before_sales TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_draw ALTER COLUMN sales_amount TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_draw ALTER COLUMN prize_contribution TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_draw ALTER COLUMN house_retention TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_draw ALTER COLUMN pool_before_draw TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_draw ALTER COLUMN allocated_prize_amount TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_draw ALTER COLUMN paid_prize_amount TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_draw ALTER COLUMN carryover_amount TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_payout ALTER COLUMN gross_amount TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_payout ALTER COLUMN tax_amount TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_payout ALTER COLUMN net_amount TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_ticket ALTER COLUMN unit_price TYPE NUMERIC(38, 2);

ALTER TABLE games.lottery_ticket ALTER COLUMN total_price TYPE NUMERIC(38, 2);

UPDATE alembic_version SET version_num='5ab3e4f257a8' WHERE alembic_version.version_num = '6e4b2f7a1c9d';

-- Running upgrade 6e4b2f7a1c9d -> 3d16e52aae72

ALTER TABLE blackjack_sessions ADD COLUMN room_id VARCHAR;

DROP INDEX ux_blackjack_sessions_user_active;

ALTER TABLE blackjack_sessions ALTER COLUMN user_id DROP NOT NULL;

UPDATE blackjack_sessions
        SET room_id = user_id,
            user_id = NULL
        WHERE id LIKE 'bj_table_%';

CREATE INDEX ix_blackjack_sessions_room_id ON blackjack_sessions (room_id);

CREATE INDEX ix_blackjack_sessions_room_status_created ON blackjack_sessions (room_id, status, created_at);

CREATE UNIQUE INDEX ux_blackjack_sessions_user_active ON blackjack_sessions (user_id) WHERE status = 'playing' AND user_id IS NOT NULL;

CREATE UNIQUE INDEX ux_blackjack_sessions_room_active ON blackjack_sessions (room_id) WHERE status = 'playing' AND room_id IS NOT NULL;

ALTER TABLE blackjack_sessions ADD CONSTRAINT fk_blackjack_sessions_user_id_users FOREIGN KEY(user_id) REFERENCES users (user_id);

INSERT INTO alembic_version (version_num) VALUES ('3d16e52aae72') RETURNING alembic_version.version_num;

-- Running upgrade 3d16e52aae72, 5ab3e4f257a8 -> 7f2c9a8d1b6e

ALTER TABLE pal ADD COLUMN archived_source_season INTEGER;

ALTER TABLE pal ADD COLUMN archived_source_pal_id BIGINT;

ALTER TABLE pal ADD CONSTRAINT ck_pal_archived_source_pair CHECK ((archived_source_season IS NULL) = (archived_source_pal_id IS NULL));

CREATE UNIQUE INDEX ux_pal_archived_source ON pal (archived_source_season, archived_source_pal_id) WHERE archived_source_season IS NOT NULL AND archived_source_pal_id IS NOT NULL;

DELETE FROM alembic_version WHERE alembic_version.version_num = '3d16e52aae72';

UPDATE alembic_version SET version_num='7f2c9a8d1b6e' WHERE alembic_version.version_num = '5ab3e4f257a8';

-- Running upgrade 7f2c9a8d1b6e -> 9b6e5d4c3a2f

CREATE OR REPLACE FUNCTION wallet_guard_validate_wallet(p_user_id text)
    RETURNS void AS $$
    DECLARE
        ledger_balance numeric;
        ledger_escrow numeric;
        wallet_allow_negative boolean;
    BEGIN
        IF p_user_id IS NULL THEN
            RETURN;
        END IF;

        SELECT
            "allow_negative_balance"
        INTO
            wallet_allow_negative
        FROM "wallet"
        WHERE "user_id" = p_user_id;

        IF NOT FOUND THEN
            IF EXISTS (
                SELECT 1
                FROM "wallet_transaction"
                WHERE "user_id" = p_user_id
            ) THEN
                RAISE EXCEPTION
                    'wallet ledger guard: wallet % missing while transactions exist',
                    p_user_id
                    USING ERRCODE = '23514';
            END IF;
            RETURN;
        END IF;

        SELECT COALESCE(SUM("amount"), 0)
        INTO ledger_balance
        FROM "wallet_transaction"
        WHERE "user_id" = p_user_id;

        SELECT COALESCE(SUM("escrow_delta"), 0)
        INTO ledger_escrow
        FROM "wallet_transaction"
        WHERE "user_id" = p_user_id;

        -- Best-effort guard for append-only wallets: this rechecks committed
        -- rows visible to this transaction, but intentionally does not take a
        -- per-wallet serialization lock. Extreme concurrent spends may still
        -- commit into a negative ledger balance and are handled by follow-up
        -- balance reads / recovery flows.
        IF NOT wallet_allow_negative AND ledger_balance < 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: wallet % may not go negative (ledger=%)',
                p_user_id,
                ledger_balance
                USING ERRCODE = '23514';
        END IF;

        IF ledger_escrow < 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: wallet % may not have negative escrow (ledger=%)',
                p_user_id,
                ledger_escrow
                USING ERRCODE = '23514';
        END IF;
    END;
    $$ LANGUAGE plpgsql;

UPDATE alembic_version SET version_num='9b6e5d4c3a2f' WHERE alembic_version.version_num = '7f2c9a8d1b6e';

-- Running upgrade 9b6e5d4c3a2f -> a4c2e9d1b0f3

ALTER TABLE wallet ALTER COLUMN projection_enabled SET DEFAULT false;

CREATE OR REPLACE FUNCTION wallet_guard_validate_wallet(p_user_id text)
    RETURNS void AS $$
    DECLARE
        ledger_balance numeric;
        ledger_escrow numeric;
        wallet_allow_negative boolean;
    BEGIN
        IF p_user_id IS NULL THEN
            RETURN;
        END IF;

        SELECT
            "allow_negative_balance"
        INTO
            wallet_allow_negative
        FROM "wallet"
        WHERE "user_id" = p_user_id;

        IF NOT FOUND THEN
            IF EXISTS (
                SELECT 1
                FROM "wallet_transaction"
                WHERE "user_id" = p_user_id
            ) THEN
                RAISE EXCEPTION
                    'wallet ledger guard: wallet % missing while transactions exist',
                    p_user_id
                    USING ERRCODE = '23514';
            END IF;
            RETURN;
        END IF;

        SELECT COALESCE(SUM("amount"), 0)
        INTO ledger_balance
        FROM "wallet_transaction"
        WHERE "user_id" = p_user_id;

        SELECT COALESCE(SUM("escrow_delta"), 0)
        INTO ledger_escrow
        FROM "wallet_transaction"
        WHERE "user_id" = p_user_id;

        -- Best-effort guard for append-only wallets: this rechecks committed
        -- rows visible to this transaction, but intentionally does not take a
        -- per-wallet serialization lock. Extreme concurrent spends may still
        -- commit into a negative ledger balance and are handled by follow-up
        -- balance reads / recovery flows.
        IF NOT wallet_allow_negative AND ledger_balance < 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: wallet % may not go negative (ledger=%)',
                p_user_id,
                ledger_balance
                USING ERRCODE = '23514';
        END IF;

        IF ledger_escrow < 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: wallet % may not have negative escrow (ledger=%)',
                p_user_id,
                ledger_escrow
                USING ERRCODE = '23514';
        END IF;
    END;
    $$ LANGUAGE plpgsql;

UPDATE "wallet"
        SET "projection_enabled" = false
        WHERE NOT ("user_id" = ANY(ARRAY['__futures_mm_treasury__', '__futures_insurance_fund__', '__futures_hedge_treasury__', '__raid_exchange_treasury__', '__insurance_treasury__', '__turnip_amm_treasury__']::text[]))
          AND "projection_enabled" IS DISTINCT FROM false;

UPDATE "wallet"
        SET "projection_enabled" = true
        WHERE "user_id" = ANY(ARRAY['__futures_mm_treasury__', '__futures_insurance_fund__', '__futures_hedge_treasury__', '__raid_exchange_treasury__', '__insurance_treasury__', '__turnip_amm_treasury__']::text[])
          AND "projection_enabled" IS DISTINCT FROM true;

UPDATE alembic_version SET version_num='a4c2e9d1b0f3' WHERE alembic_version.version_num = '9b6e5d4c3a2f';

-- Running upgrade a4c2e9d1b0f3 -> b6c7d8e9f012

ALTER TABLE turnip_seed ADD COLUMN growth_required_hours DECIMAL(38, 6);

ALTER TABLE turnip_seed ADD COLUMN growth_progress_hours DECIMAL(38, 6);

ALTER TABLE turnip_seed ADD COLUMN last_growth_accounted_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE turnip_seed ADD COLUMN latest_effective_time_bonus DECIMAL(38, 6) DEFAULT '0.000000' NOT NULL;

ALTER TABLE turnip_seed ADD COLUMN latest_effective_harvest_bonus DECIMAL(38, 6) DEFAULT '0.000000' NOT NULL;

ALTER TABLE turnip_seed ADD COLUMN matured_at TIMESTAMP WITH TIME ZONE;

UPDATE turnip_seed
        SET growth_required_hours = GREATEST(
                EXTRACT(EPOCH FROM (matures_at - planted_at)) / 3600.0,
                0.016667
            )::numeric(38, 6);

WITH active_farm_cursors AS (
            SELECT
                user_id,
                MIN(COALESCE(last_tick_at, assigned_at)) AS farm_accounted_at
            FROM land_assignment
            WHERE assignment_type = 'farm'
                AND released_at IS NULL
            GROUP BY user_id
        ),
        growth_accounting_clock AS (
            SELECT
                turnip_seed.id,
                CASE
                    WHEN turnip_seed.status = 'growing' THEN
                        GREATEST(
                            turnip_seed.planted_at,
                            LEAST(
                                CURRENT_TIMESTAMP,
                                turnip_seed.matures_at,
                                COALESCE(
                                    active_farm_cursors.farm_accounted_at,
                                    CURRENT_TIMESTAMP
                                )
                            )
                        )
                    ELSE turnip_seed.matures_at
                END AS accounted_at
            FROM turnip_seed
            LEFT JOIN active_farm_cursors
                ON active_farm_cursors.user_id = turnip_seed.user_id
        )
        UPDATE turnip_seed
        SET growth_progress_hours = CASE
                WHEN turnip_seed.status = 'growing' THEN
                    LEAST(
                        turnip_seed.growth_required_hours,
                        GREATEST(
                            0,
                            turnip_seed.growth_required_hours
                            - GREATEST(
                                EXTRACT(
                                    EPOCH FROM (
                                        turnip_seed.matures_at
                                        - growth_accounting_clock.accounted_at
                                    )
                                ) / 3600.0,
                                0
                            )
                        )
                    )::numeric(38, 6)
                ELSE turnip_seed.growth_required_hours
            END,
            last_growth_accounted_at = growth_accounting_clock.accounted_at,
            matured_at = CASE
                WHEN turnip_seed.status IN ('mature', 'harvested', 'wilted') THEN
                    turnip_seed.matures_at
                ELSE NULL
            END
        FROM growth_accounting_clock
        WHERE turnip_seed.id = growth_accounting_clock.id;

WITH missing_weather AS (
            SELECT
                id,
                GREATEST(
                    COALESCE(last_weather_accounted_at, planted_at),
                    planted_at
                ) AS missing_start,
                last_growth_accounted_at AS missing_end,
                GREATEST(
                    growth_progress_hours - COALESCE(weather_observed_hours, 0),
                    0
                ) AS missing_progress_hours
            FROM turnip_seed
            WHERE growth_progress_hours > COALESCE(weather_observed_hours, 0)
                AND last_growth_accounted_at > GREATEST(
                    COALESCE(last_weather_accounted_at, planted_at),
                    planted_at
                )
        ),
        weather_backfill AS (
            SELECT
                missing_weather.id,
                CASE
                    WHEN SUM(segments.overlap_hours) > 0 THEN
                        missing_weather.missing_progress_hours
                        * SUM(
                            COALESCE(farm_weather_hour.yield_factor, 1.000000)
                            * segments.overlap_hours
                        )
                        / SUM(segments.overlap_hours)
                    ELSE missing_weather.missing_progress_hours
                END AS missing_weather_score
            FROM missing_weather
            CROSS JOIN LATERAL (
                SELECT
                    hours.hour_start,
                    EXTRACT(
                        EPOCH FROM (
                            LEAST(
                                hours.hour_start + interval '1 hour',
                                missing_weather.missing_end
                            )
                            - GREATEST(hours.hour_start, missing_weather.missing_start)
                        )
                    ) / 3600.0 AS overlap_hours
                FROM generate_series(
                    date_trunc('hour', missing_weather.missing_start),
                    date_trunc(
                        'hour',
                        missing_weather.missing_end - interval '1 microsecond'
                    ),
                    interval '1 hour'
                ) AS hours(hour_start)
            ) AS segments
            LEFT JOIN farm_weather_hour
                ON farm_weather_hour.starts_at = segments.hour_start
            WHERE segments.overlap_hours > 0
            GROUP BY missing_weather.id, missing_weather.missing_progress_hours
        )
        UPDATE turnip_seed
        SET weather_score = (
                turnip_seed.weather_score + weather_backfill.missing_weather_score
            )::numeric(38, 6),
            weather_observed_hours = GREATEST(
                turnip_seed.weather_observed_hours,
                turnip_seed.growth_progress_hours
            )::numeric(38, 6),
            last_weather_accounted_at = turnip_seed.last_growth_accounted_at
        FROM weather_backfill
        WHERE turnip_seed.id = weather_backfill.id;

ALTER TABLE turnip_seed ALTER COLUMN growth_required_hours SET NOT NULL;

ALTER TABLE turnip_seed ALTER COLUMN growth_progress_hours SET NOT NULL;

ALTER TABLE turnip_seed ALTER COLUMN last_growth_accounted_at SET NOT NULL;

ALTER TABLE turnip_seed ALTER COLUMN latest_effective_time_bonus DROP DEFAULT;

ALTER TABLE turnip_seed ALTER COLUMN latest_effective_harvest_bonus DROP DEFAULT;

DROP INDEX IF EXISTS ix_turnip_seed_status_matures;

DROP INDEX IF EXISTS turnip_seed_status_matures_at_idx;

CREATE INDEX ix_turnip_seed_status_matured ON turnip_seed (status, matured_at);

ALTER TABLE turnip_seed DROP COLUMN last_weather_accounted_at;

ALTER TABLE turnip_seed DROP COLUMN weather_observed_hours;

ALTER TABLE turnip_seed DROP COLUMN matures_at;

UPDATE alembic_version SET version_num='b6c7d8e9f012' WHERE alembic_version.version_num = 'a4c2e9d1b0f3';

-- Running upgrade 7f2c9a8d1b6e -> 55359ab4a6cf

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

DO $$
        DECLARE
            rec record;
        BEGIN
            FOR rec IN
                SELECT nsp.nspname AS schema_name,
                       src.relname AS table_name,
                       con.conname AS constraint_name
                FROM pg_constraint con
                JOIN pg_class src ON src.oid = con.conrelid
                JOIN pg_namespace nsp ON nsp.oid = src.relnamespace
                JOIN pg_class dst ON dst.oid = con.confrelid
                JOIN pg_namespace dst_nsp ON dst_nsp.oid = dst.relnamespace
                WHERE con.contype = 'f'
                  AND dst.relname = 'issued_principal'
                  AND dst_nsp.nspname = nsp.nspname
                  AND EXISTS (
                      SELECT 1
                      FROM unnest(con.conkey) AS col(attnum)
                      JOIN pg_attribute att
                        ON att.attrelid = con.conrelid
                       AND att.attnum = col.attnum
                      WHERE att.attname = 'principal_id'
                  )
                  AND src.relname IN (
                      'wallet_transaction',
                      'security_audit_event',
                      'partner_refresh_token'
                  )
            LOOP
                EXECUTE format(
                    'ALTER TABLE %I.%I DROP CONSTRAINT IF EXISTS %I',
                    rec.schema_name,
                    rec.table_name,
                    rec.constraint_name
                );
            END LOOP;
        END $$;;

ALTER TABLE clearing_instruction DROP CONSTRAINT IF EXISTS clearing_instruction_intent_id_fkey;

ALTER TABLE clearing_batch_item DROP CONSTRAINT IF EXISTS clearing_batch_item_batch_id_fkey;

ALTER TABLE clearing_batch_item DROP CONSTRAINT IF EXISTS clearing_batch_item_intent_id_fkey;

ALTER TABLE poll_options DROP CONSTRAINT IF EXISTS poll_options_poll_id_fkey;

ALTER TABLE poll_votes DROP CONSTRAINT IF EXISTS poll_votes_poll_id_fkey;

ALTER TABLE poll_votes DROP CONSTRAINT IF EXISTS fk_poll_votes_poll_option_pair;

ALTER TABLE poll_comments DROP CONSTRAINT IF EXISTS poll_comments_poll_id_fkey;

ALTER TABLE poll_comments DROP CONSTRAINT IF EXISTS poll_comments_quote_comment_id_fkey;

ALTER TABLE poll_comment_reactions DROP CONSTRAINT IF EXISTS poll_comment_reactions_comment_id_fkey;

ALTER TABLE undercover_event DROP CONSTRAINT IF EXISTS undercover_event_session_id_fkey;

ALTER TABLE games.ponzi_events DROP CONSTRAINT IF EXISTS ponzi_events_session_id_fkey;

ALTER TABLE partner_client ALTER COLUMN client_id TYPE uuid USING (
    CASE
        WHEN client_id IS NULL THEN NULL
        ELSE client_id::uuid
    END
    );

ALTER TABLE partner_refresh_token ALTER COLUMN token_id TYPE uuid USING (
    CASE
        WHEN token_id IS NULL THEN NULL
        ELSE token_id::uuid
    END
    );

ALTER TABLE partner_refresh_token ALTER COLUMN client_id TYPE uuid USING (
    CASE
        WHEN client_id IS NULL THEN NULL
        ELSE client_id::uuid
    END
    );

ALTER TABLE partner_refresh_token ALTER COLUMN rotated_from_token_id TYPE uuid USING (
    CASE
        WHEN rotated_from_token_id IS NULL THEN NULL
        ELSE rotated_from_token_id::uuid
    END
    );

ALTER TABLE oidc_refresh_token ALTER COLUMN token_id TYPE uuid USING (
    CASE
        WHEN token_id IS NULL THEN NULL
        ELSE token_id::uuid
    END
    );

ALTER TABLE oidc_refresh_token ALTER COLUMN client_id TYPE uuid USING (
    CASE
        WHEN client_id IS NULL THEN NULL
        ELSE client_id::uuid
    END
    );

ALTER TABLE oidc_refresh_token ALTER COLUMN rotated_from_token_id TYPE uuid USING (
    CASE
        WHEN rotated_from_token_id IS NULL THEN NULL
        ELSE rotated_from_token_id::uuid
    END
    );

ALTER TABLE llm_usage_log ALTER COLUMN id TYPE uuid USING (
    CASE
        WHEN id IS NULL THEN NULL
        ELSE id::uuid
    END
    );

ALTER TABLE game_escrow ALTER COLUMN escrow_token TYPE uuid USING (
    CASE
        WHEN escrow_token IS NULL THEN NULL
        ELSE escrow_token::uuid
    END
    );

ALTER TABLE games.ponzi_sessions ALTER COLUMN id TYPE uuid USING (
    CASE
        WHEN id IS NULL THEN NULL
        ELSE id::uuid
    END
    );

ALTER TABLE games.ponzi_events ALTER COLUMN session_id TYPE uuid USING (
    CASE
        WHEN session_id IS NULL THEN NULL
        ELSE session_id::uuid
    END
    );

ALTER TABLE currency_exchange_order ALTER COLUMN id TYPE uuid USING (
    CASE
        WHEN id IS NULL THEN NULL
        ELSE id::uuid
    END
    );

ALTER TABLE polls ALTER COLUMN id TYPE uuid USING (
    CASE
        WHEN id IS NULL THEN NULL
        ELSE id::uuid
    END
    );

ALTER TABLE poll_options ALTER COLUMN id TYPE uuid USING (
    CASE
        WHEN id IS NULL THEN NULL
        ELSE id::uuid
    END
    );

ALTER TABLE poll_options ALTER COLUMN poll_id TYPE uuid USING (
    CASE
        WHEN poll_id IS NULL THEN NULL
        ELSE poll_id::uuid
    END
    );

ALTER TABLE poll_votes ALTER COLUMN id TYPE uuid USING (
    CASE
        WHEN id IS NULL THEN NULL
        ELSE id::uuid
    END
    );

ALTER TABLE poll_votes ALTER COLUMN poll_id TYPE uuid USING (
    CASE
        WHEN poll_id IS NULL THEN NULL
        ELSE poll_id::uuid
    END
    );

ALTER TABLE poll_votes ALTER COLUMN option_id TYPE uuid USING (
    CASE
        WHEN option_id IS NULL THEN NULL
        ELSE option_id::uuid
    END
    );

ALTER TABLE poll_comments ALTER COLUMN id TYPE uuid USING (
    CASE
        WHEN id IS NULL THEN NULL
        ELSE id::uuid
    END
    );

ALTER TABLE poll_comments ALTER COLUMN poll_id TYPE uuid USING (
    CASE
        WHEN poll_id IS NULL THEN NULL
        ELSE poll_id::uuid
    END
    );

ALTER TABLE poll_comments ALTER COLUMN quote_comment_id TYPE uuid USING (
    CASE
        WHEN quote_comment_id IS NULL THEN NULL
        ELSE quote_comment_id::uuid
    END
    );

ALTER TABLE poll_comment_reactions ALTER COLUMN id TYPE uuid USING (
    CASE
        WHEN id IS NULL THEN NULL
        ELSE id::uuid
    END
    );

ALTER TABLE poll_comment_reactions ALTER COLUMN comment_id TYPE uuid USING (
    CASE
        WHEN comment_id IS NULL THEN NULL
        ELSE comment_id::uuid
    END
    );

ALTER TABLE undercover_word_pair ALTER COLUMN id TYPE uuid USING (
    CASE
        WHEN id IS NULL THEN NULL
        ELSE id::uuid
    END
    );

ALTER TABLE undercover_session ALTER COLUMN id TYPE uuid USING (
    CASE
        WHEN id IS NULL THEN NULL
        ELSE id::uuid
    END
    );

ALTER TABLE undercover_event ALTER COLUMN session_id TYPE uuid USING (
    CASE
        WHEN session_id IS NULL THEN NULL
        ELSE session_id::uuid
    END
    );

ALTER TABLE issued_principal ALTER COLUMN principal_id TYPE uuid USING (
    CASE
        WHEN principal_id IS NULL THEN NULL
        WHEN principal_id ~ '^[0-9a-fA-F]{32}$' THEN principal_id::uuid
        WHEN principal_id ~ '^[0-9a-fA-F-]{36}$' THEN principal_id::uuid
        WHEN principal_id LIKE 'prn_%'
            AND SUBSTRING(principal_id FROM 5) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(principal_id FROM 5)::uuid
        
        WHEN principal_id = 'prn_legacy_audit_backfill' THEN '00000000-0000-7000-8000-000000000001'::uuid
        WHEN principal_id = '__system reverse__' THEN '00000000-0000-7000-8000-000000000002'::uuid
        ELSE principal_id::uuid
    END
    );

ALTER TABLE issued_principal ALTER COLUMN source_principal_id TYPE uuid USING (
    CASE
        WHEN source_principal_id IS NULL THEN NULL
        WHEN source_principal_id ~ '^[0-9a-fA-F]{32}$' THEN source_principal_id::uuid
        WHEN source_principal_id ~ '^[0-9a-fA-F-]{36}$' THEN source_principal_id::uuid
        WHEN source_principal_id LIKE 'prn_%'
            AND SUBSTRING(source_principal_id FROM 5) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(source_principal_id FROM 5)::uuid
        
        WHEN source_principal_id = 'prn_legacy_audit_backfill' THEN '00000000-0000-7000-8000-000000000001'::uuid
        WHEN source_principal_id = '__system reverse__' THEN '00000000-0000-7000-8000-000000000002'::uuid
        ELSE source_principal_id::uuid
    END
    );

ALTER TABLE issued_principal ALTER COLUMN client_id TYPE uuid USING (
    CASE
        WHEN client_id IS NULL THEN NULL
        ELSE client_id::uuid
    END
    );

ALTER TABLE security_audit_event ALTER COLUMN event_id TYPE uuid USING (
    CASE
        WHEN event_id IS NULL THEN NULL
        WHEN event_id ~ '^[0-9a-fA-F]{32}$' THEN event_id::uuid
        WHEN event_id ~ '^[0-9a-fA-F-]{36}$' THEN event_id::uuid
        WHEN event_id LIKE 'sae_%'
            AND SUBSTRING(event_id FROM 5) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(event_id FROM 5)::uuid
        
        ELSE event_id::uuid
    END
    );

ALTER TABLE security_audit_event ALTER COLUMN principal_id TYPE uuid USING (
    CASE
        WHEN principal_id IS NULL THEN NULL
        WHEN principal_id ~ '^[0-9a-fA-F]{32}$' THEN principal_id::uuid
        WHEN principal_id ~ '^[0-9a-fA-F-]{36}$' THEN principal_id::uuid
        WHEN principal_id LIKE 'prn_%'
            AND SUBSTRING(principal_id FROM 5) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(principal_id FROM 5)::uuid
        
        WHEN principal_id = 'prn_legacy_audit_backfill' THEN '00000000-0000-7000-8000-000000000001'::uuid
        WHEN principal_id = '__system reverse__' THEN '00000000-0000-7000-8000-000000000002'::uuid
        ELSE principal_id::uuid
    END
    );

ALTER TABLE wallet_transaction ADD COLUMN principal_id_uuid uuid;

UPDATE wallet_transaction
        SET principal_id_uuid = (CASE
        WHEN principal_id IS NULL THEN NULL
        WHEN principal_id ~ '^[0-9a-fA-F]{32}$' THEN principal_id::uuid
        WHEN principal_id ~ '^[0-9a-fA-F-]{36}$' THEN principal_id::uuid
        WHEN principal_id LIKE 'prn_%'
            AND SUBSTRING(principal_id FROM 5) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(principal_id FROM 5)::uuid
        
        WHEN principal_id = 'prn_legacy_audit_backfill' THEN '00000000-0000-7000-8000-000000000001'::uuid
        WHEN principal_id = '__system reverse__' THEN '00000000-0000-7000-8000-000000000002'::uuid
        ELSE principal_id::uuid
    END)
        WHERE principal_id IS NOT NULL;

DROP INDEX IF EXISTS ix_wallet_transaction_principal_id;

ALTER TABLE wallet_transaction DROP COLUMN principal_id;

ALTER TABLE wallet_transaction RENAME COLUMN principal_id_uuid TO principal_id;

CREATE INDEX ix_wallet_transaction_principal_id ON wallet_transaction (principal_id);

ALTER TABLE partner_refresh_token ALTER COLUMN principal_id TYPE uuid USING (
    CASE
        WHEN principal_id IS NULL THEN NULL
        WHEN principal_id ~ '^[0-9a-fA-F]{32}$' THEN principal_id::uuid
        WHEN principal_id ~ '^[0-9a-fA-F-]{36}$' THEN principal_id::uuid
        WHEN principal_id LIKE 'prn_%'
            AND SUBSTRING(principal_id FROM 5) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(principal_id FROM 5)::uuid
        
        WHEN principal_id = 'prn_legacy_audit_backfill' THEN '00000000-0000-7000-8000-000000000001'::uuid
        WHEN principal_id = '__system reverse__' THEN '00000000-0000-7000-8000-000000000002'::uuid
        ELSE principal_id::uuid
    END
    );

ALTER TABLE payment_intent ALTER COLUMN intent_id TYPE uuid USING (
    CASE
        WHEN intent_id IS NULL THEN NULL
        WHEN intent_id ~ '^[0-9a-fA-F]{32}$' THEN intent_id::uuid
        WHEN intent_id ~ '^[0-9a-fA-F-]{36}$' THEN intent_id::uuid
        WHEN intent_id LIKE 'pi_%'
            AND SUBSTRING(intent_id FROM 4) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(intent_id FROM 4)::uuid
        
        ELSE intent_id::uuid
    END
    );

ALTER TABLE payment_intent ALTER COLUMN checkout_token TYPE uuid USING (
    CASE
        WHEN checkout_token IS NULL THEN NULL
        WHEN checkout_token ~ '^[0-9a-fA-F]{32}$' THEN checkout_token::uuid
        WHEN checkout_token ~ '^[0-9a-fA-F-]{36}$' THEN checkout_token::uuid
        WHEN checkout_token LIKE 'co_%'
            AND SUBSTRING(checkout_token FROM 4) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(checkout_token FROM 4)::uuid
        
        ELSE checkout_token::uuid
    END
    );

ALTER TABLE payment_intent ALTER COLUMN partner_client_id TYPE uuid USING (
    CASE
        WHEN partner_client_id IS NULL THEN NULL
        ELSE partner_client_id::uuid
    END
    );

ALTER TABLE clearing_instruction ALTER COLUMN instruction_id TYPE uuid USING (
    CASE
        WHEN instruction_id IS NULL THEN NULL
        WHEN instruction_id ~ '^[0-9a-fA-F]{32}$' THEN instruction_id::uuid
        WHEN instruction_id ~ '^[0-9a-fA-F-]{36}$' THEN instruction_id::uuid
        WHEN instruction_id LIKE 'ci_%'
            AND SUBSTRING(instruction_id FROM 4) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(instruction_id FROM 4)::uuid
        
        ELSE instruction_id::uuid
    END
    );

ALTER TABLE clearing_instruction ALTER COLUMN intent_id TYPE uuid USING (
    CASE
        WHEN intent_id IS NULL THEN NULL
        WHEN intent_id ~ '^[0-9a-fA-F]{32}$' THEN intent_id::uuid
        WHEN intent_id ~ '^[0-9a-fA-F-]{36}$' THEN intent_id::uuid
        WHEN intent_id LIKE 'pi_%'
            AND SUBSTRING(intent_id FROM 4) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(intent_id FROM 4)::uuid
        
        ELSE intent_id::uuid
    END
    );

ALTER TABLE clearing_instruction ALTER COLUMN reverse_of_instruction_id TYPE uuid USING (
    CASE
        WHEN reverse_of_instruction_id IS NULL THEN NULL
        WHEN reverse_of_instruction_id ~ '^[0-9a-fA-F]{32}$' THEN reverse_of_instruction_id::uuid
        WHEN reverse_of_instruction_id ~ '^[0-9a-fA-F-]{36}$' THEN reverse_of_instruction_id::uuid
        WHEN reverse_of_instruction_id LIKE 'ci_%'
            AND SUBSTRING(reverse_of_instruction_id FROM 4) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(reverse_of_instruction_id FROM 4)::uuid
        
        ELSE reverse_of_instruction_id::uuid
    END
    );

ALTER TABLE clearing_instruction ALTER COLUMN partner_client_id TYPE uuid USING (
    CASE
        WHEN partner_client_id IS NULL THEN NULL
        ELSE partner_client_id::uuid
    END
    );

ALTER TABLE clearing_batch ALTER COLUMN batch_id TYPE uuid USING (
    CASE
        WHEN batch_id IS NULL THEN NULL
        WHEN batch_id ~ '^[0-9a-fA-F]{32}$' THEN batch_id::uuid
        WHEN batch_id ~ '^[0-9a-fA-F-]{36}$' THEN batch_id::uuid
        WHEN batch_id LIKE 'cb_%'
            AND SUBSTRING(batch_id FROM 4) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(batch_id FROM 4)::uuid
        
        ELSE batch_id::uuid
    END
    );

ALTER TABLE clearing_batch ALTER COLUMN partner_client_id TYPE uuid USING (
    CASE
        WHEN partner_client_id IS NULL THEN NULL
        ELSE partner_client_id::uuid
    END
    );

ALTER TABLE clearing_batch_item ALTER COLUMN batch_item_id TYPE uuid USING (
    CASE
        WHEN batch_item_id IS NULL THEN NULL
        WHEN batch_item_id ~ '^[0-9a-fA-F]{32}$' THEN batch_item_id::uuid
        WHEN batch_item_id ~ '^[0-9a-fA-F-]{36}$' THEN batch_item_id::uuid
        ELSE uuid_generate_v5('318963d8-e71e-498d-a3c6-6d7112faaedc'::uuid, 'legacy-clearing-batch-item:' || batch_item_id)
    END
    );

ALTER TABLE clearing_batch_item ALTER COLUMN batch_id TYPE uuid USING (
    CASE
        WHEN batch_id IS NULL THEN NULL
        WHEN batch_id ~ '^[0-9a-fA-F]{32}$' THEN batch_id::uuid
        WHEN batch_id ~ '^[0-9a-fA-F-]{36}$' THEN batch_id::uuid
        WHEN batch_id LIKE 'cb_%'
            AND SUBSTRING(batch_id FROM 4) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(batch_id FROM 4)::uuid
        
        ELSE batch_id::uuid
    END
    );

ALTER TABLE clearing_batch_item ALTER COLUMN intent_id TYPE uuid USING (
    CASE
        WHEN intent_id IS NULL THEN NULL
        WHEN intent_id ~ '^[0-9a-fA-F]{32}$' THEN intent_id::uuid
        WHEN intent_id ~ '^[0-9a-fA-F-]{36}$' THEN intent_id::uuid
        WHEN intent_id LIKE 'pi_%'
            AND SUBSTRING(intent_id FROM 4) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(intent_id FROM 4)::uuid
        
        ELSE intent_id::uuid
    END
    );

ALTER TABLE webhook_delivery ALTER COLUMN event_id TYPE uuid USING (
    CASE
        WHEN event_id IS NULL THEN NULL
        WHEN event_id ~ '^[0-9a-fA-F]{32}$' THEN event_id::uuid
        WHEN event_id ~ '^[0-9a-fA-F-]{36}$' THEN event_id::uuid
        ELSE uuid_generate_v5('318963d8-e71e-498d-a3c6-6d7112faaedc'::uuid, 'legacy-webhook-event:' || event_id)
    END
    );

ALTER TABLE webhook_delivery ALTER COLUMN partner_client_id TYPE uuid USING (
    CASE
        WHEN partner_client_id IS NULL THEN NULL
        ELSE partner_client_id::uuid
    END
    );

ALTER TABLE idempotency_record ALTER COLUMN record_id TYPE uuid USING (
    CASE
        WHEN record_id IS NULL THEN NULL
        WHEN record_id ~ '^[0-9a-fA-F]{32}$' THEN record_id::uuid
        WHEN record_id ~ '^[0-9a-fA-F-]{36}$' THEN record_id::uuid
        WHEN record_id LIKE 'idem_%'
            AND SUBSTRING(record_id FROM 6) ~ '^[0-9a-fA-F]{32}$|^[0-9a-fA-F-]{36}$'
            THEN SUBSTRING(record_id FROM 6)::uuid
        
        ELSE record_id::uuid
    END
    );

ALTER TABLE security_audit_event ADD CONSTRAINT security_audit_event_principal_id_fkey FOREIGN KEY(principal_id) REFERENCES issued_principal (principal_id);

ALTER TABLE wallet_transaction ADD CONSTRAINT fk_wallet_transaction_principal_id FOREIGN KEY(principal_id) REFERENCES issued_principal (principal_id);

ALTER TABLE partner_refresh_token ADD CONSTRAINT fk_partner_refresh_token_principal_id FOREIGN KEY(principal_id) REFERENCES issued_principal (principal_id);

ALTER TABLE clearing_instruction ADD CONSTRAINT clearing_instruction_intent_id_fkey FOREIGN KEY(intent_id) REFERENCES payment_intent (intent_id);

ALTER TABLE clearing_batch_item ADD CONSTRAINT clearing_batch_item_batch_id_fkey FOREIGN KEY(batch_id) REFERENCES clearing_batch (batch_id);

ALTER TABLE clearing_batch_item ADD CONSTRAINT clearing_batch_item_intent_id_fkey FOREIGN KEY(intent_id) REFERENCES payment_intent (intent_id);

ALTER TABLE poll_options ADD CONSTRAINT poll_options_poll_id_fkey FOREIGN KEY(poll_id) REFERENCES polls (id) ON DELETE CASCADE;

ALTER TABLE poll_votes ADD CONSTRAINT poll_votes_poll_id_fkey FOREIGN KEY(poll_id) REFERENCES polls (id) ON DELETE CASCADE;

ALTER TABLE poll_votes ADD CONSTRAINT fk_poll_votes_poll_option_pair FOREIGN KEY(poll_id, option_id) REFERENCES poll_options (poll_id, id) ON DELETE CASCADE;

ALTER TABLE poll_comments ADD CONSTRAINT poll_comments_poll_id_fkey FOREIGN KEY(poll_id) REFERENCES polls (id) ON DELETE CASCADE;

ALTER TABLE poll_comments ADD CONSTRAINT poll_comments_quote_comment_id_fkey FOREIGN KEY(quote_comment_id) REFERENCES poll_comments (id) ON DELETE SET NULL;

ALTER TABLE poll_comment_reactions ADD CONSTRAINT poll_comment_reactions_comment_id_fkey FOREIGN KEY(comment_id) REFERENCES poll_comments (id) ON DELETE CASCADE;

ALTER TABLE undercover_event ADD CONSTRAINT undercover_event_session_id_fkey FOREIGN KEY(session_id) REFERENCES undercover_session (id) ON DELETE CASCADE;

ALTER TABLE games.ponzi_events ADD CONSTRAINT ponzi_events_session_id_fkey FOREIGN KEY(session_id) REFERENCES games.ponzi_sessions (id) ON DELETE CASCADE;

INSERT INTO alembic_version (version_num) VALUES ('55359ab4a6cf') RETURNING alembic_version.version_num;

-- Running upgrade a4c2e9d1b0f3 -> 6d5b8e2f91c4

ALTER TABLE games.lottery_draw ADD COLUMN closed_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE games.lottery_draw ADD COLUMN draw_started_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE games.lottery_draw ADD COLUMN drawn_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE games.lottery_draw ADD COLUMN settlement_started_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE games.lottery_draw ADD COLUMN settled_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE games.lottery_draw ADD COLUMN notification_started_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE games.lottery_draw ADD COLUMN retry_after TIMESTAMP WITH TIME ZONE;

ALTER TABLE games.lottery_draw ADD COLUMN stage_lease_expires_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE games.lottery_draw ADD COLUMN retry_count INTEGER DEFAULT 0 NOT NULL;

ALTER TABLE games.lottery_draw ADD COLUMN stage_owner VARCHAR(128);

ALTER TABLE user_notification ADD COLUMN reference_id VARCHAR(255);

CREATE UNIQUE INDEX ux_user_notification_reference_id ON user_notification (reference_id) WHERE reference_id IS NOT NULL;

INSERT INTO alembic_version (version_num) VALUES ('6d5b8e2f91c4') RETURNING alembic_version.version_num;

-- Running upgrade 6d5b8e2f91c4 -> 9a7c3e5d1b24

ALTER TABLE games.lottery_draw ADD COLUMN settlement_cursor_ticket_id INTEGER;

ALTER TABLE games.lottery_draw ADD COLUMN settlement_completed_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE games.lottery_draw ADD COLUMN notification_cursor_user_id VARCHAR(255);

ALTER TABLE games.lottery_draw ADD COLUMN notification_completed_at TIMESTAMP WITH TIME ZONE;

UPDATE alembic_version SET version_num='9a7c3e5d1b24' WHERE alembic_version.version_num = '6d5b8e2f91c4';

-- Running upgrade a4c2e9d1b0f3 -> b5d6c7e8f901

CREATE OR REPLACE FUNCTION wallet_guard_validate_tx_group(p_tx_group_id text)
    RETURNS void AS $$
    DECLARE
        tx_count integer;
        group_amount numeric;
        group_escrow_delta numeric;
        self_user_id text;
        self_counterparty_id text;
        self_amount numeric;
        self_escrow_delta numeric;
    BEGIN
        IF p_tx_group_id IS NULL THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id may not be null'
                USING ERRCODE = '23514';
        END IF;

        SELECT
            COUNT(*),
            COALESCE(SUM(amount), 0),
            COALESCE(SUM(escrow_delta), 0)
        INTO tx_count, group_amount, group_escrow_delta
        FROM wallet_transaction
        WHERE tx_group_id = p_tx_group_id;

        IF tx_count = 1 THEN
            SELECT user_id, counterparty_id, amount, escrow_delta
            INTO self_user_id, self_counterparty_id, self_amount, self_escrow_delta
            FROM wallet_transaction
            WHERE tx_group_id = p_tx_group_id
            LIMIT 1;

            IF self_user_id IS DISTINCT FROM self_counterparty_id
               OR self_amount + self_escrow_delta <> 0 THEN
                RAISE EXCEPTION
                    'wallet ledger guard: tx_group_id % is not a valid self-group',
                    p_tx_group_id
                    USING ERRCODE = '23514';
            END IF;
        ELSIF tx_count > 2 THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id % must not have more than 2 rows, got %',
                p_tx_group_id,
                tx_count
                USING ERRCODE = '23514';
        ELSIF EXISTS (
            SELECT 1
            FROM wallet_transaction wt
            WHERE wt.tx_group_id = p_tx_group_id
              AND (
                  SELECT COUNT(*)
                  FROM wallet_transaction peer
                  WHERE peer.tx_group_id = wt.tx_group_id
                    AND peer.id <> wt.id
                    AND peer.user_id = wt.counterparty_id
                    AND peer.counterparty_id = wt.user_id
              ) <> 1
        ) THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id % has broken counterparty links',
                p_tx_group_id
                USING ERRCODE = '23514';
        END IF;

        IF group_amount + group_escrow_delta <> 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: tx_group_id % must balance amount+escrow_delta to zero, got %',
                p_tx_group_id,
                group_amount + group_escrow_delta
                USING ERRCODE = '23514';
        END IF;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_validate_wallet(p_user_id text)
    RETURNS void AS $$
    DECLARE
        ledger_balance numeric;
        ledger_escrow numeric;
        wallet_allow_negative boolean;
    BEGIN
        IF p_user_id IS NULL THEN
            RETURN;
        END IF;

        SELECT
            "allow_negative_balance"
        INTO
            wallet_allow_negative
        FROM "wallet"
        WHERE "user_id" = p_user_id;

        IF NOT FOUND THEN
            IF EXISTS (
                SELECT 1
                FROM "wallet_transaction"
                WHERE "user_id" = p_user_id
            ) THEN
                RAISE EXCEPTION
                    'wallet ledger guard: wallet % missing while transactions exist',
                    p_user_id
                    USING ERRCODE = '23514';
            END IF;
            RETURN;
        END IF;

        SELECT COALESCE(SUM("amount"), 0)
        INTO ledger_balance
        FROM "wallet_transaction"
        WHERE "user_id" = p_user_id;

        SELECT COALESCE(SUM("escrow_delta"), 0)
        INTO ledger_escrow
        FROM "wallet_transaction"
        WHERE "user_id" = p_user_id;

        -- Best-effort guard for append-only wallets: this rechecks committed
        -- rows visible to this transaction, but intentionally does not take a
        -- per-wallet serialization lock. Extreme concurrent spends may still
        -- commit into a negative ledger balance and are handled by follow-up
        -- balance reads / recovery flows.
        IF NOT wallet_allow_negative AND ledger_balance < 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: wallet % may not go negative (ledger=%)',
                p_user_id,
                ledger_balance
                USING ERRCODE = '23514';
        END IF;

        IF ledger_escrow < 0 THEN
            RAISE EXCEPTION
                'wallet ledger guard: wallet % may not have negative escrow (ledger=%)',
                p_user_id,
                ledger_escrow
                USING ERRCODE = '23514';
        END IF;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_enforce_tx_group()
    RETURNS trigger AS $$
    BEGIN
        IF TG_OP = 'DELETE' THEN
            PERFORM wallet_guard_validate_tx_group(OLD.tx_group_id);
            RETURN NULL;
        END IF;

        PERFORM wallet_guard_validate_tx_group(NEW.tx_group_id);
        IF TG_OP = 'UPDATE' AND OLD.tx_group_id IS DISTINCT FROM NEW.tx_group_id THEN
            PERFORM wallet_guard_validate_tx_group(OLD.tx_group_id);
        END IF;
        RETURN NULL;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_enforce_tx_wallet()
    RETURNS trigger AS $$
    BEGIN
        IF TG_OP = 'DELETE' THEN
            PERFORM wallet_guard_validate_wallet(OLD.user_id);
            RETURN NULL;
        END IF;

        PERFORM wallet_guard_validate_wallet(NEW.user_id);
        IF TG_OP = 'UPDATE' AND OLD.user_id IS DISTINCT FROM NEW.user_id THEN
            PERFORM wallet_guard_validate_wallet(OLD.user_id);
        END IF;
        RETURN NULL;
    END;
    $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION wallet_guard_enforce_wallet()
    RETURNS trigger AS $$
    BEGIN
        IF TG_OP = 'DELETE' THEN
            PERFORM wallet_guard_validate_wallet(OLD.user_id);
            RETURN NULL;
        END IF;

        PERFORM wallet_guard_validate_wallet(NEW.user_id);
        RETURN NULL;
    END;
    $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS wallet_tx_group_guard ON wallet_transaction;

CREATE CONSTRAINT TRIGGER wallet_tx_group_guard
    AFTER INSERT OR UPDATE OF user_id, amount, escrow_delta, counterparty_id, tx_group_id OR DELETE ON wallet_transaction
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION wallet_guard_enforce_tx_group();

DROP TRIGGER IF EXISTS wallet_tx_wallet_guard ON wallet_transaction;

CREATE CONSTRAINT TRIGGER wallet_tx_wallet_guard
    AFTER INSERT OR UPDATE OF user_id, amount, escrow_delta, counterparty_id, tx_group_id OR DELETE ON wallet_transaction
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION wallet_guard_enforce_tx_wallet();

DROP TRIGGER IF EXISTS wallet_row_wallet_guard ON wallet;

CREATE CONSTRAINT TRIGGER wallet_row_wallet_guard
    AFTER INSERT OR UPDATE OF user_id, allow_negative_balance OR DELETE ON wallet
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION wallet_guard_enforce_wallet();

ALTER TABLE wallet DROP COLUMN projection_enabled;

ALTER TABLE wallet DROP COLUMN escrow_balance;

ALTER TABLE wallet DROP COLUMN balance;

INSERT INTO alembic_version (version_num) VALUES ('b5d6c7e8f901') RETURNING alembic_version.version_num;

-- Running upgrade b5d6c7e8f901 -> c6a4d0e9f2b1

INSERT INTO "wallet" (
                "user_id",
                "allow_negative_balance",
                "snapshot_balance",
                "snapshot_escrow_balance",
                "snapshot_tx_id",
                "total_credited",
                "created_at"
            )
            SELECT '8e4f7199-4cdc-47ec-a579-6801073fad79', true, 0, 0, 0, 0, now()
            WHERE EXISTS (
                SELECT 1 FROM "users"
                WHERE "user_id" = '8e4f7199-4cdc-47ec-a579-6801073fad79'
            )
            ON CONFLICT ("user_id") DO UPDATE
            SET "allow_negative_balance" = true;

ALTER TABLE "wallet_transaction" DISABLE TRIGGER "wallet_tx_group_guard";

ALTER TABLE "wallet_transaction" DISABLE TRIGGER "wallet_tx_wallet_guard";

CREATE TEMP TABLE wallet_season_initial_funding_repair ON COMMIT DROP AS
            SELECT
                "id",
                "user_id",
                "amount",
                "escrow_delta",
                "description",
                "reference_id",
                "memo",
                "principal_id",
                "metadata",
                "created_at",
                'season5_initial_' || "id"::text AS repair_tx_group_id
            FROM "wallet_transaction"
            WHERE "tx_type" = 'balance_adjustment'
              AND "description" = 'S5赛季初始资金'
              AND "counterparty_id" IS NULL
              AND "tx_group_id" IS NULL;

UPDATE "wallet_transaction" AS tx
            SET
                "tx_type" = 'season_initial_funding',
                "counterparty_id" = '8e4f7199-4cdc-47ec-a579-6801073fad79',
                "tx_group_id" = repair.repair_tx_group_id,
                "balance_after" = NULL,
                "escrow_after" = NULL
            FROM wallet_season_initial_funding_repair AS repair
            WHERE tx."id" = repair."id";

INSERT INTO "wallet_transaction" (
                "user_id",
                "amount",
                "escrow_delta",
                "balance_after",
                "escrow_after",
                "tx_type",
                "description",
                "reference_id",
                "memo",
                "counterparty_id",
                "tx_group_id",
                "principal_id",
                "metadata",
                "created_at"
            )
            SELECT
                '8e4f7199-4cdc-47ec-a579-6801073fad79',
                -repair."amount",
                -repair."escrow_delta",
                NULL,
                NULL,
                'season_initial_funding',
                repair."description",
                repair."reference_id",
                repair."memo",
                repair."user_id",
                repair.repair_tx_group_id,
                repair."principal_id",
                repair."metadata",
                repair."created_at"
            FROM wallet_season_initial_funding_repair AS repair;

ALTER TABLE "wallet_transaction" ENABLE TRIGGER "wallet_tx_group_guard";

ALTER TABLE "wallet_transaction" ENABLE TRIGGER "wallet_tx_wallet_guard";

SELECT wallet_guard_validate_tx_group(repair.repair_tx_group_id)
            FROM wallet_season_initial_funding_repair AS repair;

SELECT wallet_guard_validate_wallet('8e4f7199-4cdc-47ec-a579-6801073fad79');

SELECT wallet_guard_validate_wallet(repair."user_id")
            FROM wallet_season_initial_funding_repair AS repair;

UPDATE alembic_version SET version_num='c6a4d0e9f2b1' WHERE alembic_version.version_num = 'b5d6c7e8f901';

-- Running upgrade c6a4d0e9f2b1 -> d2a7f4c8e1b9

SET CONSTRAINTS ALL IMMEDIATE;

ALTER TABLE "wallet_transaction" DROP CONSTRAINT IF EXISTS "ck_wallet_transaction_peer_fields";

ALTER TABLE "wallet_transaction" ALTER COLUMN "counterparty_id" SET NOT NULL;

ALTER TABLE "wallet_transaction" ALTER COLUMN "tx_group_id" SET NOT NULL;

UPDATE alembic_version SET version_num='d2a7f4c8e1b9' WHERE alembic_version.version_num = 'c6a4d0e9f2b1';

-- Running upgrade d2a7f4c8e1b9 -> 2e8889a6923c

CREATE INDEX ix_pal_egg_breeding_ready ON pal_egg (breeding_ready_at) WHERE status = 'breeding';

CREATE INDEX ix_pal_egg_hatching_ready ON pal_egg (hatches_at) WHERE status = 'hatching';

UPDATE alembic_version SET version_num='2e8889a6923c' WHERE alembic_version.version_num = 'd2a7f4c8e1b9';

-- Running upgrade 2e8889a6923c -> 3a1b5c7d9e2f

ALTER TABLE partner_client ADD COLUMN client_type VARCHAR(32) DEFAULT 'confidential' NOT NULL;

ALTER TABLE partner_client ALTER COLUMN client_secret_encrypted DROP NOT NULL;

ALTER TABLE partner_client ADD CONSTRAINT ck_partner_client_client_type CHECK (client_type IN ('confidential', 'public'));

UPDATE alembic_version SET version_num='3a1b5c7d9e2f' WHERE alembic_version.version_num = '2e8889a6923c';

-- Running upgrade 3a1b5c7d9e2f, 55359ab4a6cf, 9a7c3e5d1b24, b6c7d8e9f012 -> 2f9c0b1d8a6e

ALTER TABLE pal ADD COLUMN current_breeding_egg_id BIGINT;

ALTER TABLE pal ADD COLUMN current_breeding_until TIMESTAMP WITH TIME ZONE;

CREATE INDEX ix_pal_current_breeding_egg_id ON pal (current_breeding_egg_id);

ALTER TABLE pal ADD CONSTRAINT fk_pal_current_breeding_egg_id_pal_egg FOREIGN KEY(current_breeding_egg_id) REFERENCES pal_egg (id);

UPDATE pal
        SET
          current_breeding_egg_id = egg.id,
          current_breeding_until = egg.breeding_ready_at
        FROM pal_egg AS egg
        WHERE egg.status = 'breeding'
          AND egg.parent1_id = pal.id;

UPDATE pal
        SET
          current_breeding_egg_id = egg.id,
          current_breeding_until = egg.breeding_ready_at
        FROM pal_egg AS egg
        WHERE egg.status = 'breeding'
          AND egg.parent2_id = pal.id;

DELETE FROM alembic_version WHERE alembic_version.version_num = '3a1b5c7d9e2f';

DELETE FROM alembic_version WHERE alembic_version.version_num = '55359ab4a6cf';

DELETE FROM alembic_version WHERE alembic_version.version_num = '9a7c3e5d1b24';

UPDATE alembic_version SET version_num='2f9c0b1d8a6e' WHERE alembic_version.version_num = 'b6c7d8e9f012';

COMMIT;

