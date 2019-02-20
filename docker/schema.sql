--
-- PostgreSQL database dump
--

-- Dumped from database version 10.6 (Ubuntu 10.6-0ubuntu0.18.10.1)
-- Dumped by pg_dump version 10.6 (Ubuntu 10.6-0ubuntu0.18.10.1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: plpgsql; Type: EXTENSION; Schema: -; Owner: 
--

CREATE EXTENSION IF NOT EXISTS plpgsql WITH SCHEMA pg_catalog;


--
-- Name: EXTENSION plpgsql; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION plpgsql IS 'PL/pgSQL procedural language';


--
-- Name: diesel_manage_updated_at(regclass); Type: FUNCTION; Schema: public; Owner: middleware
--

CREATE FUNCTION public.diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;


ALTER FUNCTION public.diesel_manage_updated_at(_tbl regclass) OWNER TO middleware;

--
-- Name: diesel_set_updated_at(); Type: FUNCTION; Schema: public; Owner: middleware
--

CREATE FUNCTION public.diesel_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.diesel_set_updated_at() OWNER TO middleware;

SET default_tablespace = '';

SET default_with_oids = false;

--
-- Name: __diesel_schema_migrations; Type: TABLE; Schema: public; Owner: middleware
--

CREATE TABLE public.__diesel_schema_migrations (
    version character varying(50) NOT NULL,
    run_on timestamp without time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.__diesel_schema_migrations OWNER TO middleware;

--
-- Name: key_blocks; Type: TABLE; Schema: public; Owner: middleware
--

CREATE TABLE public.key_blocks (
    id integer NOT NULL,
    hash character varying(55),
    height bigint,
    miner character varying(55),
    beneficiary character varying(55),
    nonce numeric(20,0),
    pow text,
    prev_hash character varying(55),
    prev_key_hash character varying(55),
    state_hash character varying(55),
    target bigint,
    time_ bigint,
    version integer
);


ALTER TABLE public.key_blocks OWNER TO middleware;

--
-- Name: key_blocks_id_seq; Type: SEQUENCE; Schema: public; Owner: middleware
--

CREATE SEQUENCE public.key_blocks_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.key_blocks_id_seq OWNER TO middleware;

--
-- Name: key_blocks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: middleware
--

ALTER SEQUENCE public.key_blocks_id_seq OWNED BY public.key_blocks.id;


--
-- Name: micro_blocks; Type: TABLE; Schema: public; Owner: middleware
--

CREATE TABLE public.micro_blocks (
    id integer NOT NULL,
    key_block_id integer NOT NULL,
    hash character varying(55) NOT NULL,
    pof_hash character varying(55) NOT NULL,
    prev_hash character varying(55) NOT NULL,
    prev_key_hash character varying(55) NOT NULL,
    signature character varying(255) NOT NULL,
    time_ bigint,
    state_hash character varying(255) NOT NULL,
    txs_hash character varying(255) NOT NULL,
    version integer NOT NULL
);


ALTER TABLE public.micro_blocks OWNER TO middleware;

--
-- Name: micro_blocks_id_seq; Type: SEQUENCE; Schema: public; Owner: middleware
--

CREATE SEQUENCE public.micro_blocks_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.micro_blocks_id_seq OWNER TO middleware;

--
-- Name: micro_blocks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: middleware
--

ALTER SEQUENCE public.micro_blocks_id_seq OWNED BY public.micro_blocks.id;


--
-- Name: transactions; Type: TABLE; Schema: public; Owner: middleware
--

CREATE TABLE public.transactions (
    id integer NOT NULL,
    micro_block_id integer,
    block_height integer NOT NULL,
    block_hash character varying(55) NOT NULL,
    hash character varying(55) NOT NULL,
    signatures text NOT NULL,
    tx_type character varying(64) NOT NULL,
    tx jsonb NOT NULL,
    fee bigint NOT NULL,
    size integer NOT NULL,
    valid boolean DEFAULT true NOT NULL
);


ALTER TABLE public.transactions OWNER TO middleware;

--
-- Name: transactions_id_seq; Type: SEQUENCE; Schema: public; Owner: middleware
--

CREATE SEQUENCE public.transactions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.transactions_id_seq OWNER TO middleware;

--
-- Name: transactions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: middleware
--

ALTER SEQUENCE public.transactions_id_seq OWNED BY public.transactions.id;


--
-- Name: key_blocks id; Type: DEFAULT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.key_blocks ALTER COLUMN id SET DEFAULT nextval('public.key_blocks_id_seq'::regclass);


--
-- Name: micro_blocks id; Type: DEFAULT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.micro_blocks ALTER COLUMN id SET DEFAULT nextval('public.micro_blocks_id_seq'::regclass);


--
-- Name: transactions id; Type: DEFAULT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.transactions ALTER COLUMN id SET DEFAULT nextval('public.transactions_id_seq'::regclass);


--
-- Name: __diesel_schema_migrations __diesel_schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.__diesel_schema_migrations
    ADD CONSTRAINT __diesel_schema_migrations_pkey PRIMARY KEY (version);


--
-- Name: key_blocks key_blocks_hash_key; Type: CONSTRAINT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.key_blocks
    ADD CONSTRAINT key_blocks_hash_key UNIQUE (hash);


--
-- Name: key_blocks key_blocks_height_key; Type: CONSTRAINT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.key_blocks
    ADD CONSTRAINT key_blocks_height_key UNIQUE (height);


--
-- Name: key_blocks key_blocks_pkey; Type: CONSTRAINT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.key_blocks
    ADD CONSTRAINT key_blocks_pkey PRIMARY KEY (id);


--
-- Name: micro_blocks micro_blocks_pkey; Type: CONSTRAINT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.micro_blocks
    ADD CONSTRAINT micro_blocks_pkey PRIMARY KEY (id);


--
-- Name: transactions transactions_hash_key; Type: CONSTRAINT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_hash_key UNIQUE (hash);


--
-- Name: transactions transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_pkey PRIMARY KEY (id);


--
-- Name: key_blocks_beneficiary; Type: INDEX; Schema: public; Owner: middleware
--

CREATE INDEX key_blocks_beneficiary ON public.key_blocks USING btree (beneficiary);


--
-- Name: key_blocks_height; Type: INDEX; Schema: public; Owner: middleware
--

CREATE INDEX key_blocks_height ON public.key_blocks USING btree (height);


--
-- Name: micro_blocks_hash; Type: INDEX; Schema: public; Owner: middleware
--

CREATE INDEX micro_blocks_hash ON public.micro_blocks USING btree (hash);


--
-- Name: micro_blocks_key_block_id; Type: INDEX; Schema: public; Owner: middleware
--

CREATE INDEX micro_blocks_key_block_id ON public.micro_blocks USING btree (key_block_id);


--
-- Name: transactions_block_hash_index; Type: INDEX; Schema: public; Owner: middleware
--

CREATE INDEX transactions_block_hash_index ON public.transactions USING btree (block_hash);


--
-- Name: transactions_micro_block_id; Type: INDEX; Schema: public; Owner: middleware
--

CREATE INDEX transactions_micro_block_id ON public.transactions USING btree (micro_block_id);


--
-- Name: transactions_tx_type_index; Type: INDEX; Schema: public; Owner: middleware
--

CREATE INDEX transactions_tx_type_index ON public.transactions USING btree (tx_type);


--
-- Name: micro_blocks micro_blocks_key_block_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.micro_blocks
    ADD CONSTRAINT micro_blocks_key_block_id_fkey FOREIGN KEY (key_block_id) REFERENCES public.key_blocks(id) ON DELETE CASCADE;


--
-- Name: transactions transactions_micro_block_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: middleware
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_micro_block_id_fkey FOREIGN KEY (micro_block_id) REFERENCES public.micro_blocks(id) ON DELETE CASCADE;



create materialized view if not exists public.agg_generations as
    ( select kb.*, COALESCE(jsonb_agg( mb.*) FILTER (WHERE mb.id IS NOT NULL), '[]') 
    as micro_blocks from public.key_blocks kb left join 
    ( select m.*, COALESCE(jsonb_agg( tx.*) FILTER (WHERE tx.id IS NOT NULL), '[]') 
    as transactions from public.micro_blocks m left join public.transactions tx on m.id = tx.micro_block_id group by m.id ) 
    mb on kb.id = mb.key_block_id where kb.height >=0 group by kb.id ) with no data;

create index agg_generations_height_idx ON public.agg_generations (height);
refresh materialized view public.agg_generations;



create materialized view if not exists public.account_spendtx_balance as (
select tx->>'recipient_id' as account, SUM(NULLIF(tx->>'amount', '')::decimal) AS amount,
count(hash) as txcount
from transactions t where valid= true and tx_type = 'SpendTx' and tx->>'recipient_id' like 'ak_%'
group by account order by amount desc);

refresh materialized view public.account_spendtx_balance;
--
-- PostgreSQL database dump complete
--

