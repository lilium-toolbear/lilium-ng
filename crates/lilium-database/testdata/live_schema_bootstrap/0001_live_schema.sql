SET check_function_bodies = false;
SET row_security = off;

DROP TEXT SEARCH CONFIGURATION IF EXISTS public.zhparser;
CREATE TEXT SEARCH CONFIGURATION public.zhparser (COPY = pg_catalog.simple);

-- Functions from live schema
CREATE FUNCTION public.ensure_time_partitions(p_table_name text DEFAULT NULL::text, p_anchor timestamp with time zone DEFAULT timezone('UTC'::text, clock_timestamp()), p_apply boolean DEFAULT true) RETURNS TABLE(table_name text, parent_name text, child_name text, partition_start timestamp with time zone, partition_end timestamp with time zone, applied boolean)
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
$$;


--
-- Name: messages_content_tsv_trigger(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.messages_content_tsv_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
  NEW.content_tsv := to_tsvector('zhparser', COALESCE(NEW.content_text, ''));
  RETURN NEW;
END
$$;


--
-- Name: messages_fts_trigger(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.messages_fts_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
        BEGIN
            NEW.content_tsv := to_tsvector(
                'zhparser'::regconfig,
                COALESCE(NEW.content_text, '')
            );
            RETURN NEW;
        END;
        $$;


--
-- Name: notify_futures_tick(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.notify_futures_tick() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
        $$;


--
-- Name: notify_message_inserted(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.notify_message_inserted() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
        BEGIN
            PERFORM pg_notify('message_inserted', NEW.message_id);
            RETURN NEW;
        END;
        $$;


--
-- Name: notify_outgoing_command_inserted(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.notify_outgoing_command_inserted() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
        BEGIN
            -- Send notification on 'outgoing_command_inserted' channel
            -- Payload is JSON with command ID and account_user_id for filtering
            PERFORM pg_notify(
                'outgoing_command_inserted',
                json_build_object('id', NEW.id, 'account_user_id', NEW.account_user_id)::text
            );
            RETURN NEW;
        END;
        $$;


--
-- Name: notify_outgoing_command_updated(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.notify_outgoing_command_updated() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
        $$;


--
-- Name: notify_turnip_tick(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.notify_turnip_tick() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
        BEGIN
            PERFORM pg_notify('turnip_tick', '');
            RETURN NEW;
        END;
        $$;


--
-- Name: notify_wallet_transaction_inserted(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.notify_wallet_transaction_inserted() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
        BEGIN
            PERFORM pg_notify('wallet_transaction_inserted', NEW.id::text);
            RETURN NEW;
        END;
        $$;


--
-- Name: notify_websocket_event_inserted(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.notify_websocket_event_inserted() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
        BEGIN
            PERFORM pg_notify('websocket_event_inserted', NEW.id::text);
            RETURN NEW;
        END;
        $$;


--
-- Name: users_fts_trigger(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.users_fts_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
                BEGIN
                    NEW.name_tsv := to_tsvector('zhparser'::regconfig,
                                                COALESCE(NEW.full_name, ''));
                    RETURN NEW;
                END;
                $$;


--
-- Name: users_name_tsv_trigger(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.users_name_tsv_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
  NEW.name_tsv := to_tsvector('zhparser', COALESCE(NEW.full_name, ''));
  RETURN NEW;
END
$$;


--
-- Name: wallet_guard_enforce_total_zero(); Type: FUNCTION; Schema: public; Owner: -
--


-- Tables from live schema

-- Table: public.users







CREATE TABLE public.users (
    user_id character varying NOT NULL,
    full_name character varying,
    avatar_url character varying,
    bio character varying,
    birthday character varying,
    birthday_public boolean DEFAULT false,
    quirk character varying,
    is_bot boolean DEFAULT false,
    gender character varying,
    metadata jsonb,
    raw_data jsonb,
    last_seen timestamp with time zone,
    message_count integer NOT NULL,
    deleted_count integer NOT NULL,
    recalled_count integer NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    name_tsv tsvector,
    avatar_file character varying
);



ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (user_id);



CREATE INDEX idx_users_last_seen ON public.users USING btree (last_seen DESC);



CREATE INDEX idx_users_message_count ON public.users USING btree (message_count DESC);



CREATE INDEX idx_users_name_tsv ON public.users USING gin (name_tsv);



CREATE INDEX ix_users_full_name ON public.users USING btree (full_name);



CREATE TRIGGER users_fts_update BEFORE INSERT OR UPDATE ON public.users FOR EACH ROW EXECUTE FUNCTION public.users_fts_trigger();



CREATE TRIGGER users_name_tsv_update BEFORE INSERT OR UPDATE ON public.users FOR EACH ROW EXECUTE FUNCTION public.users_name_tsv_trigger();





-- Table: public.rooms







CREATE TABLE public.rooms (
    room_id character varying NOT NULL,
    title character varying NOT NULL,
    chat_type character varying,
    avatar_url character varying,
    member_count integer,
    tags text[],
    is_public boolean DEFAULT false,
    creator_id character varying,
    last_message_at timestamp with time zone,
    first_message_at timestamp with time zone,
    backfill_until timestamp with time zone,
    history_complete boolean DEFAULT false NOT NULL,
    message_count integer NOT NULL,
    deleted_count integer NOT NULL,
    recalled_count integer NOT NULL,
    edited_count integer NOT NULL,
    image_count integer NOT NULL,
    is_active boolean DEFAULT false NOT NULL,
    dissolved_at timestamp with time zone,
    raw_data jsonb,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    account_ids text[] DEFAULT '{}'::text[] NOT NULL
);



ALTER TABLE ONLY public.rooms
    ADD CONSTRAINT rooms_pkey PRIMARY KEY (room_id);



CREATE INDEX idx_rooms_is_active ON public.rooms USING btree (is_active) WHERE (is_active = true);



CREATE INDEX idx_rooms_last_message_at ON public.rooms USING btree (last_message_at DESC);



CREATE INDEX ix_rooms_chat_type ON public.rooms USING btree (chat_type);



CREATE INDEX ix_rooms_dissolved_at ON public.rooms USING btree (dissolved_at);



CREATE INDEX ix_rooms_message_count ON public.rooms USING btree (message_count);





-- Table: public.room_members







CREATE TABLE public.room_members (
    room_id character varying NOT NULL,
    user_id character varying NOT NULL,
    role character varying,
    joined_at timestamp with time zone,
    raw_data jsonb,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    left_at timestamp with time zone
);



ALTER TABLE ONLY public.room_members
    ADD CONSTRAINT room_members_pkey PRIMARY KEY (room_id, user_id);



CREATE INDEX ix_room_members_role ON public.room_members USING btree (role);



CREATE INDEX ix_room_members_room_id ON public.room_members USING btree (room_id);



CREATE INDEX ix_room_members_user_id ON public.room_members USING btree (user_id);





-- Table: public.dzmm_account







CREATE TABLE public.dzmm_account (
    user_id character varying NOT NULL,
    user_profile jsonb NOT NULL,
    email character varying,
    password character varying,
    signin_code character varying,
    cookies character varying,
    is_enabled boolean NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    signin_code_image bytea,
    signin_code_image_mime character varying
);



ALTER TABLE ONLY public.dzmm_account
    ADD CONSTRAINT dzmm_account_pkey PRIMARY KEY (user_id);



ALTER TABLE ONLY public.dzmm_account
    ADD CONSTRAINT fk_dzmm_account_user_id_users FOREIGN KEY (user_id) REFERENCES public.users(user_id);





-- Table: public.websocket_connections







CREATE TABLE public.websocket_connections (
    lock_id bigint NOT NULL,
    account_user_id character varying NOT NULL,
    connected_at timestamp with time zone NOT NULL,
    last_heartbeat timestamp with time zone NOT NULL
);



CREATE SEQUENCE public.websocket_connections_lock_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;



ALTER SEQUENCE public.websocket_connections_lock_id_seq OWNED BY public.websocket_connections.lock_id;



ALTER TABLE ONLY public.websocket_connections ALTER COLUMN lock_id SET DEFAULT nextval('public.websocket_connections_lock_id_seq'::regclass);



ALTER TABLE ONLY public.websocket_connections
    ADD CONSTRAINT websocket_connections_pkey PRIMARY KEY (lock_id);



CREATE INDEX ix_websocket_connections_account_user_id ON public.websocket_connections USING btree (account_user_id);



ALTER TABLE ONLY public.websocket_connections
    ADD CONSTRAINT websocket_connections_account_user_id_fkey FOREIGN KEY (account_user_id) REFERENCES public.dzmm_account(user_id);





-- Table: public.outgoing_commands







CREATE TABLE public.outgoing_commands (
    id integer NOT NULL,
    created_at timestamp with time zone NOT NULL,
    account_user_id character varying NOT NULL,
    event character varying NOT NULL,
    data jsonb NOT NULL,
    require_ack boolean NOT NULL,
    status character varying NOT NULL,
    processed_at timestamp with time zone,
    ack_response jsonb,
    error_message character varying,
    attempt_count integer NOT NULL,
    max_attempts integer NOT NULL
);



CREATE SEQUENCE public.outgoing_commands_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;



ALTER SEQUENCE public.outgoing_commands_id_seq OWNED BY public.outgoing_commands.id;



ALTER TABLE ONLY public.outgoing_commands ALTER COLUMN id SET DEFAULT nextval('public.outgoing_commands_id_seq'::regclass);



ALTER TABLE ONLY public.outgoing_commands
    ADD CONSTRAINT outgoing_commands_pkey PRIMARY KEY (id);



CREATE INDEX idx_outgoing_commands_pending_account_id ON public.outgoing_commands USING btree (account_user_id, id) WHERE ((status)::text = 'pending'::text);



CREATE INDEX ix_outgoing_commands_account_user_id ON public.outgoing_commands USING btree (account_user_id);



CREATE INDEX ix_outgoing_commands_created_at ON public.outgoing_commands USING btree (created_at);



CREATE INDEX ix_outgoing_commands_status ON public.outgoing_commands USING btree (status);



CREATE TRIGGER outgoing_command_inserted_trigger AFTER INSERT ON public.outgoing_commands FOR EACH ROW EXECUTE FUNCTION public.notify_outgoing_command_inserted();



CREATE TRIGGER outgoing_command_updated_trigger AFTER UPDATE ON public.outgoing_commands FOR EACH ROW EXECUTE FUNCTION public.notify_outgoing_command_updated();





-- Table: public.event_processor_offsets







CREATE TABLE public.event_processor_offsets (
    processor_id character varying NOT NULL,
    last_processed_id integer NOT NULL,
    last_processed_at timestamp with time zone,
    updated_at timestamp with time zone NOT NULL,
    last_processed_timestamp timestamp with time zone
);



ALTER TABLE ONLY public.event_processor_offsets
    ADD CONSTRAINT event_processor_offsets_pkey PRIMARY KEY (processor_id);





-- Table: public.messages






CREATE TABLE public.messages (
    message_id text NOT NULL,
    room_id text NOT NULL,
    sent_at timestamp with time zone NOT NULL,
    sent_by text NOT NULL,
    content_type text NOT NULL,
    content_text text,
    content_tsv tsvector,
    attachment_url text,
    attachment_file text,
    sticker_id text,
    alt_text text,
    metadata jsonb,
    raw_data jsonb NOT NULL,
    source text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone,
    is_deleted boolean DEFAULT false NOT NULL,
    deleted_at timestamp with time zone,
    deleted_by text,
    is_recalled boolean DEFAULT false NOT NULL,
    is_edited boolean DEFAULT false NOT NULL,
    history jsonb,
    reference_message_id text,
    reference_data jsonb
)
PARTITION BY RANGE (sent_at);



ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_partitioned_pkey PRIMARY KEY (message_id, sent_at);



CREATE INDEX idx_messages_content_tsv ON ONLY public.messages USING gin (content_tsv);



CREATE INDEX idx_messages_room_id_sent_at ON ONLY public.messages USING btree (room_id, sent_at);



CREATE INDEX idx_messages_sent_by_sent_at_id ON ONLY public.messages USING btree (sent_by, sent_at, message_id);



CREATE INDEX idx_messages_source_created_at_id ON ONLY public.messages USING btree (source, created_at, message_id);



CREATE INDEX ix_messages_attachment_file ON ONLY public.messages USING btree (attachment_file);



CREATE INDEX ix_messages_content_type ON ONLY public.messages USING btree (content_type);



CREATE INDEX ix_messages_deleted_by ON ONLY public.messages USING btree (deleted_by);



CREATE INDEX ix_messages_is_deleted ON ONLY public.messages USING btree (is_deleted);



CREATE INDEX ix_messages_is_edited ON ONLY public.messages USING btree (is_edited);



CREATE INDEX ix_messages_is_recalled ON ONLY public.messages USING btree (is_recalled);



CREATE INDEX ix_messages_reference_message_id ON ONLY public.messages USING btree (reference_message_id);



CREATE INDEX ix_messages_room_id ON ONLY public.messages USING btree (room_id);



CREATE INDEX ix_messages_sent_at ON ONLY public.messages USING btree (sent_at);



CREATE INDEX ix_messages_sent_by ON ONLY public.messages USING btree (sent_by);



CREATE INDEX ix_messages_source ON ONLY public.messages USING btree (source);



CREATE INDEX ix_messages_sticker_id ON ONLY public.messages USING btree (sticker_id);



CREATE TRIGGER message_inserted_trigger AFTER INSERT ON public.messages FOR EACH ROW EXECUTE FUNCTION public.notify_message_inserted();



CREATE TRIGGER messages_fts_update BEFORE INSERT OR UPDATE ON public.messages FOR EACH ROW EXECUTE FUNCTION public.messages_fts_trigger();





-- Table: public.websocket_events






CREATE TABLE public.websocket_events (
    id bigint NOT NULL,
    "timestamp" timestamp with time zone NOT NULL,
    user_id text NOT NULL,
    event text NOT NULL,
    data jsonb NOT NULL
)
PARTITION BY RANGE ("timestamp");



ALTER TABLE public.websocket_events ALTER COLUMN id ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME public.websocket_events_partitioned_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



ALTER TABLE ONLY public.websocket_events
    ADD CONSTRAINT websocket_events_partitioned_pkey PRIMARY KEY (id, "timestamp");



CREATE INDEX ix_websocket_events_event_timestamp_id ON ONLY public.websocket_events USING btree (event, "timestamp", id);



CREATE INDEX ix_websocket_events_timestamp_id ON ONLY public.websocket_events USING btree ("timestamp", id);



CREATE INDEX ix_websocket_events_user_id_timestamp_id ON ONLY public.websocket_events USING btree (user_id, "timestamp", id);



CREATE TRIGGER websocket_event_inserted_trigger AFTER INSERT ON public.websocket_events FOR EACH ROW EXECUTE FUNCTION public.notify_websocket_event_inserted();



