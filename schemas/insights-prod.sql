--
-- PostgreSQL database dump
--

-- Dumped from database version 15.3
-- Dumped by pg_dump version 15.3

-- Started on 2023-08-20 20:59:05

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- TOC entry 7 (class 2615 OID 16796)
-- Name: aiven_extras; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA aiven_extras;


ALTER SCHEMA aiven_extras OWNER TO postgres;

--
-- TOC entry 8 (class 2615 OID 2200)
-- Name: public; Type: SCHEMA; Schema: -; Owner: avnadmin
--

-- *not* creating schema, since initdb creates it


ALTER SCHEMA public OWNER TO avnadmin;

--
-- TOC entry 2 (class 3079 OID 16797)
-- Name: aiven_extras; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS aiven_extras WITH SCHEMA aiven_extras;


--
-- TOC entry 4461 (class 0 OID 0)
-- Dependencies: 2
-- Name: EXTENSION aiven_extras; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION aiven_extras IS 'aiven_extras';


--
-- TOC entry 3 (class 3079 OID 16936)
-- Name: postgres_fdw; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS postgres_fdw WITH SCHEMA public;


--
-- TOC entry 4462 (class 0 OID 0)
-- Dependencies: 3
-- Name: EXTENSION postgres_fdw; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION postgres_fdw IS 'foreign-data wrapper for remote PostgreSQL servers';


--
-- TOC entry 245 (class 1255 OID 16649)
-- Name: collect_win_pct(); Type: FUNCTION; Schema: public; Owner: avnadmin
--

CREATE FUNCTION public.collect_win_pct() RETURNS TABLE(name character varying, logs integer, wins integer, pct numeric)
    LANGUAGE plpgsql
    AS $$ 
DECLARE 
	_ID integer; 
	_NAME text; 
	_LOGS_TOTAL integer; 
BEGIN
	for _id, _name in select team_id, team_name from team loop
		select count (*) from log into _logs_total where (blu_team_id = _id or red_team_id = _id);
		"name" := _name;
		logs := _logs_total;
		wins := get_win_count(_id);
		pct := trunc(cast(get_win_count(_id) as decimal)/cast(_logs_total as decimal), 2);
		return next;
	end loop;
end;
$$;


ALTER FUNCTION public.collect_win_pct() OWNER TO avnadmin;

--
-- TOC entry 246 (class 1255 OID 16650)
-- Name: get_win_count(integer); Type: FUNCTION; Schema: public; Owner: avnadmin
--

CREATE FUNCTION public.get_win_count(team_id integer) RETURNS integer
    LANGUAGE plpgsql
    AS $$
declare
wins integer;
begin
	select
		count(*)
	into wins
	from
		log
	where
   		(
			(blu_team_id = team_id and red_team_id != 0 and blu_team_score > red_team_score) or
			(red_team_id = team_id and blu_team_id != 0 and red_team_score > blu_team_score)
		);

	return wins;
end;
$$;


ALTER FUNCTION public.get_win_count(team_id integer) OWNER TO avnadmin;

--
-- TOC entry 2122 (class 1417 OID 16945)
-- Name: demos_remote; Type: SERVER; Schema: -; Owner: avnadmin
--

CREATE SERVER demos_remote FOREIGN DATA WRAPPER postgres_fdw OPTIONS (
    dbname 'xx',
    host 'xxx',
    port 'xx',
    sslmode 'require'
);


ALTER SERVER demos_remote OWNER TO avnadmin;

--
-- TOC entry 4480 (class 0 OID 0)
-- Name: USER MAPPING avnadmin SERVER demos_remote; Type: USER MAPPING; Schema: -; Owner: avnadmin
--

CREATE USER MAPPING FOR avnadmin SERVER demos_remote OPTIONS (
    password 'xxx',
    "user" 'xxxx'
);


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- TOC entry 217 (class 1259 OID 16653)
-- Name: bomb_attempt; Type: TABLE; Schema: public; Owner: avnadmin
--

CREATE TABLE public.bomb_attempt (
    id integer NOT NULL,
    player_id bigint NOT NULL,
    log_id integer NOT NULL,
    damage integer NOT NULL,
    damage_taken integer NOT NULL,
    start_tick integer NOT NULL,
    end_tick integer,
    died boolean
);


ALTER TABLE public.bomb_attempt OWNER TO avnadmin;

--
-- TOC entry 218 (class 1259 OID 16656)
-- Name: bomb_attempt_id_seq; Type: SEQUENCE; Schema: public; Owner: avnadmin
--

CREATE SEQUENCE public.bomb_attempt_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.bomb_attempt_id_seq OWNER TO avnadmin;

--
-- TOC entry 4482 (class 0 OID 0)
-- Dependencies: 218
-- Name: bomb_attempt_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: avnadmin
--

ALTER SEQUENCE public.bomb_attempt_id_seq OWNED BY public.bomb_attempt.id;


--
-- TOC entry 230 (class 1259 OID 16947)
-- Name: demos; Type: FOREIGN TABLE; Schema: public; Owner: avnadmin
--

CREATE FOREIGN TABLE public.demos (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    url character varying(255) NOT NULL,
    map character varying(255) NOT NULL,
    red character varying(255) NOT NULL,
    blu character varying(255) NOT NULL,
    uploader integer NOT NULL,
    duration integer NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    backend character varying(255) NOT NULL,
    path character varying(255) NOT NULL,
    "scoreBlue" integer NOT NULL,
    "scoreRed" integer NOT NULL,
    version integer NOT NULL,
    server character varying(255) NOT NULL,
    nick character varying(255) NOT NULL,
    deleted_at timestamp without time zone,
    "playerCount" integer NOT NULL,
    hash character varying(255) NOT NULL,
    blue_team_id integer,
    red_team_id integer
)
SERVER demos_remote
OPTIONS (
    schema_name 'public',
    table_name 'demos'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN id OPTIONS (
    column_name 'id'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN name OPTIONS (
    column_name 'name'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN url OPTIONS (
    column_name 'url'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN map OPTIONS (
    column_name 'map'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN red OPTIONS (
    column_name 'red'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN blu OPTIONS (
    column_name 'blu'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN uploader OPTIONS (
    column_name 'uploader'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN duration OPTIONS (
    column_name 'duration'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN created_at OPTIONS (
    column_name 'created_at'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN updated_at OPTIONS (
    column_name 'updated_at'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN backend OPTIONS (
    column_name 'backend'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN path OPTIONS (
    column_name 'path'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN "scoreBlue" OPTIONS (
    column_name 'scoreBlue'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN "scoreRed" OPTIONS (
    column_name 'scoreRed'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN version OPTIONS (
    column_name 'version'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN server OPTIONS (
    column_name 'server'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN nick OPTIONS (
    column_name 'nick'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN deleted_at OPTIONS (
    column_name 'deleted_at'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN "playerCount" OPTIONS (
    column_name 'playerCount'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN hash OPTIONS (
    column_name 'hash'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN blue_team_id OPTIONS (
    column_name 'blue_team_id'
);
ALTER FOREIGN TABLE public.demos ALTER COLUMN red_team_id OPTIONS (
    column_name 'red_team_id'
);


ALTER FOREIGN TABLE public.demos OWNER TO avnadmin;

--
-- TOC entry 219 (class 1259 OID 16663)
-- Name: log; Type: TABLE; Schema: public; Owner: avnadmin
--

CREATE TABLE public.log (
    log_id integer NOT NULL,
    unix_timestamp integer NOT NULL,
    map character varying(50) NOT NULL,
    red_team_id integer NOT NULL,
    blu_team_id integer NOT NULL,
    red_team_score integer NOT NULL,
    blu_team_score integer NOT NULL
);


ALTER TABLE public.log OWNER TO avnadmin;

--
-- TOC entry 220 (class 1259 OID 16666)
-- Name: player_stats; Type: TABLE; Schema: public; Owner: avnadmin
--

CREATE TABLE public.player_stats (
    id integer NOT NULL,
    log_id integer NOT NULL,
    player_steamid64 bigint NOT NULL,
    kills integer,
    deaths integer,
    dmg integer,
    dmg_real integer,
    dt integer,
    dt_real integer,
    hr integer,
    ubers integer,
    drops integer,
    headshots integer,
    headshots_hit integer
);


ALTER TABLE public.player_stats OWNER TO avnadmin;

--
-- TOC entry 231 (class 1259 OID 16950)
-- Name: players; Type: FOREIGN TABLE; Schema: public; Owner: avnadmin
--

CREATE FOREIGN TABLE public.players (
    id integer NOT NULL,
    demo_id integer NOT NULL,
    demo_user_id integer NOT NULL,
    user_id integer NOT NULL,
    name character varying(255) NOT NULL,
    team character varying(255) NOT NULL,
    class character varying(255) NOT NULL,
    kills integer,
    assists integer,
    deaths integer
)
SERVER demos_remote
OPTIONS (
    schema_name 'public',
    table_name 'players'
);
ALTER FOREIGN TABLE public.players ALTER COLUMN id OPTIONS (
    column_name 'id'
);
ALTER FOREIGN TABLE public.players ALTER COLUMN demo_id OPTIONS (
    column_name 'demo_id'
);
ALTER FOREIGN TABLE public.players ALTER COLUMN demo_user_id OPTIONS (
    column_name 'demo_user_id'
);
ALTER FOREIGN TABLE public.players ALTER COLUMN user_id OPTIONS (
    column_name 'user_id'
);
ALTER FOREIGN TABLE public.players ALTER COLUMN name OPTIONS (
    column_name 'name'
);
ALTER FOREIGN TABLE public.players ALTER COLUMN team OPTIONS (
    column_name 'team'
);
ALTER FOREIGN TABLE public.players ALTER COLUMN class OPTIONS (
    column_name 'class'
);
ALTER FOREIGN TABLE public.players ALTER COLUMN kills OPTIONS (
    column_name 'kills'
);
ALTER FOREIGN TABLE public.players ALTER COLUMN assists OPTIONS (
    column_name 'assists'
);
ALTER FOREIGN TABLE public.players ALTER COLUMN deaths OPTIONS (
    column_name 'deaths'
);


ALTER FOREIGN TABLE public.players OWNER TO avnadmin;

--
-- TOC entry 232 (class 1259 OID 16953)
-- Name: users; Type: FOREIGN TABLE; Schema: public; Owner: avnadmin
--

CREATE FOREIGN TABLE public.users (
    id integer NOT NULL,
    steamid character varying(255) NOT NULL,
    name character varying(255) NOT NULL,
    avatar character varying(255) NOT NULL,
    token character varying(255) NOT NULL,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL
)
SERVER demos_remote
OPTIONS (
    schema_name 'public',
    table_name 'users'
);
ALTER FOREIGN TABLE public.users ALTER COLUMN id OPTIONS (
    column_name 'id'
);
ALTER FOREIGN TABLE public.users ALTER COLUMN steamid OPTIONS (
    column_name 'steamid'
);
ALTER FOREIGN TABLE public.users ALTER COLUMN name OPTIONS (
    column_name 'name'
);
ALTER FOREIGN TABLE public.users ALTER COLUMN avatar OPTIONS (
    column_name 'avatar'
);
ALTER FOREIGN TABLE public.users ALTER COLUMN token OPTIONS (
    column_name 'token'
);
ALTER FOREIGN TABLE public.users ALTER COLUMN created_at OPTIONS (
    column_name 'created_at'
);
ALTER FOREIGN TABLE public.users ALTER COLUMN updated_at OPTIONS (
    column_name 'updated_at'
);


ALTER FOREIGN TABLE public.users OWNER TO avnadmin;

--
-- TOC entry 233 (class 1259 OID 16956)
-- Name: connected_demos; Type: MATERIALIZED VIEW; Schema: public; Owner: avnadmin
--

CREATE MATERIALIZED VIEW public.connected_demos AS
 SELECT demos.name,
    demos.id,
    demos.url,
    demos.created_at,
    log.log_id,
    log.red_team_id,
    log.blu_team_id,
    log.red_team_score,
    log.blu_team_score,
    log.map
   FROM (public.demos
     RIGHT JOIN public.log ON ((((demos.created_at >= ((to_timestamp(((log.unix_timestamp - 30))::double precision))::timestamp without time zone AT TIME ZONE 'utc'::text)) AND (demos.created_at <= ((to_timestamp(((log.unix_timestamp + 100))::double precision))::timestamp without time zone AT TIME ZONE 'utc'::text))) AND ((demos.map)::text = (log.map)::text) AND (abs(((demos."scoreBlue" - log.blu_team_score) + (log.red_team_score - demos."scoreRed"))) < 2) AND (EXISTS ( SELECT player_stats.player_steamid64
           FROM public.player_stats
          WHERE ((player_stats.log_id = log.log_id) AND (((player_stats.player_steamid64)::character varying)::text IN ( SELECT users.steamid
                   FROM public.users
                  WHERE (users.id IN ( SELECT players.user_id
                           FROM public.players
                          WHERE (players.demo_id = demos.id)))))))))))
  WHERE (log.map IS NOT NULL)
  ORDER BY log.log_id
  WITH NO DATA;


ALTER TABLE public.connected_demos OWNER TO avnadmin;

--
-- TOC entry 221 (class 1259 OID 16691)
-- Name: player; Type: TABLE; Schema: public; Owner: avnadmin
--

CREATE TABLE public.player (
    id integer NOT NULL,
    steamid64 bigint NOT NULL,
    team_id integer,
    name character varying(255)
);


ALTER TABLE public.player OWNER TO avnadmin;

--
-- TOC entry 222 (class 1259 OID 16694)
-- Name: player_id_seq; Type: SEQUENCE; Schema: public; Owner: avnadmin
--

CREATE SEQUENCE public.player_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.player_id_seq OWNER TO avnadmin;

--
-- TOC entry 4483 (class 0 OID 0)
-- Dependencies: 222
-- Name: player_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: avnadmin
--

ALTER SEQUENCE public.player_id_seq OWNED BY public.player.id;


--
-- TOC entry 223 (class 1259 OID 16695)
-- Name: player_stats_id_seq; Type: SEQUENCE; Schema: public; Owner: avnadmin
--

CREATE SEQUENCE public.player_stats_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.player_stats_id_seq OWNER TO avnadmin;

--
-- TOC entry 4484 (class 0 OID 0)
-- Dependencies: 223
-- Name: player_stats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: avnadmin
--

ALTER SEQUENCE public.player_stats_id_seq OWNED BY public.player_stats.id;


--
-- TOC entry 224 (class 1259 OID 16699)
-- Name: team; Type: TABLE; Schema: public; Owner: avnadmin
--

CREATE TABLE public.team (
    team_id integer NOT NULL,
    team_name character varying(50) NOT NULL
);


ALTER TABLE public.team OWNER TO avnadmin;

--
-- TOC entry 225 (class 1259 OID 16702)
-- Name: team_stats; Type: TABLE; Schema: public; Owner: avnadmin
--

CREATE TABLE public.team_stats (
    id integer NOT NULL,
    log_id integer NOT NULL,
    team_id integer NOT NULL,
    score integer,
    kills integer,
    deaths integer,
    dmg integer,
    charges integer,
    drops integer,
    first_caps integer,
    caps integer
);


ALTER TABLE public.team_stats OWNER TO avnadmin;

--
-- TOC entry 226 (class 1259 OID 16705)
-- Name: team_stats_id_seq; Type: SEQUENCE; Schema: public; Owner: avnadmin
--

CREATE SEQUENCE public.team_stats_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.team_stats_id_seq OWNER TO avnadmin;

--
-- TOC entry 4485 (class 0 OID 0)
-- Dependencies: 226
-- Name: team_stats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: avnadmin
--

ALTER SEQUENCE public.team_stats_id_seq OWNED BY public.team_stats.id;


--
-- TOC entry 4286 (class 2604 OID 16715)
-- Name: bomb_attempt id; Type: DEFAULT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.bomb_attempt ALTER COLUMN id SET DEFAULT nextval('public.bomb_attempt_id_seq'::regclass);


--
-- TOC entry 4288 (class 2604 OID 16716)
-- Name: player id; Type: DEFAULT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player ALTER COLUMN id SET DEFAULT nextval('public.player_id_seq'::regclass);


--
-- TOC entry 4287 (class 2604 OID 16717)
-- Name: player_stats id; Type: DEFAULT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player_stats ALTER COLUMN id SET DEFAULT nextval('public.player_stats_id_seq'::regclass);


--
-- TOC entry 4289 (class 2604 OID 16718)
-- Name: team_stats id; Type: DEFAULT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.team_stats ALTER COLUMN id SET DEFAULT nextval('public.team_stats_id_seq'::regclass);


--
-- TOC entry 4291 (class 2606 OID 16720)
-- Name: bomb_attempt bomb_attempt_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.bomb_attempt
    ADD CONSTRAINT bomb_attempt_pkey PRIMARY KEY (id);


--
-- TOC entry 4293 (class 2606 OID 16722)
-- Name: log log_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.log
    ADD CONSTRAINT log_pkey PRIMARY KEY (log_id);


--
-- TOC entry 4297 (class 2606 OID 16724)
-- Name: player player_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player
    ADD CONSTRAINT player_pkey PRIMARY KEY (id);


--
-- TOC entry 4295 (class 2606 OID 16726)
-- Name: player_stats player_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player_stats
    ADD CONSTRAINT player_stats_pkey PRIMARY KEY (id);


--
-- TOC entry 4299 (class 2606 OID 16728)
-- Name: player player_steamid64_key; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player
    ADD CONSTRAINT player_steamid64_key UNIQUE (steamid64);


--
-- TOC entry 4301 (class 2606 OID 16730)
-- Name: team team_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.team
    ADD CONSTRAINT team_pkey PRIMARY KEY (team_id);


--
-- TOC entry 4305 (class 2606 OID 16732)
-- Name: team_stats team_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.team_stats
    ADD CONSTRAINT team_stats_pkey PRIMARY KEY (id);


--
-- TOC entry 4303 (class 2606 OID 16734)
-- Name: team team_team_name_key; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.team
    ADD CONSTRAINT team_team_name_key UNIQUE (team_name);


--
-- TOC entry 4306 (class 2606 OID 16735)
-- Name: bomb_attempt bomb_attempt_log_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.bomb_attempt
    ADD CONSTRAINT bomb_attempt_log_id_fkey FOREIGN KEY (log_id) REFERENCES public.log(log_id);


--
-- TOC entry 4307 (class 2606 OID 16740)
-- Name: bomb_attempt bomb_attempt_player_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.bomb_attempt
    ADD CONSTRAINT bomb_attempt_player_id_fkey FOREIGN KEY (player_id) REFERENCES public.player(steamid64);


--
-- TOC entry 4308 (class 2606 OID 16745)
-- Name: player_stats player_stats_log_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player_stats
    ADD CONSTRAINT player_stats_log_id_fkey FOREIGN KEY (log_id) REFERENCES public.log(log_id);


--
-- TOC entry 4309 (class 2606 OID 16750)
-- Name: player_stats player_stats_player_steamid64_fkey; Type: FK CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player_stats
    ADD CONSTRAINT player_stats_player_steamid64_fkey FOREIGN KEY (player_steamid64) REFERENCES public.player(steamid64);


--
-- TOC entry 4310 (class 2606 OID 16755)
-- Name: team_stats team_stats_log_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.team_stats
    ADD CONSTRAINT team_stats_log_id_fkey FOREIGN KEY (log_id) REFERENCES public.log(log_id);


--
-- TOC entry 4460 (class 0 OID 0)
-- Dependencies: 7
-- Name: SCHEMA aiven_extras; Type: ACL; Schema: -; Owner: postgres
--

GRANT USAGE ON SCHEMA aiven_extras TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4463 (class 0 OID 0)
-- Dependencies: 257
-- Name: FUNCTION auto_explain_load(); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.auto_explain_load() TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4464 (class 0 OID 0)
-- Dependencies: 265
-- Name: FUNCTION claim_public_schema_ownership(); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.claim_public_schema_ownership() TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4465 (class 0 OID 0)
-- Dependencies: 248
-- Name: FUNCTION dblink_slot_create_or_drop(arg_connection_string text, arg_slot_name text, arg_action text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.dblink_slot_create_or_drop(arg_connection_string text, arg_slot_name text, arg_action text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4466 (class 0 OID 0)
-- Dependencies: 254
-- Name: FUNCTION pg_create_publication_for_all_tables(arg_publication_name text, arg_publish text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.pg_create_publication_for_all_tables(arg_publication_name text, arg_publish text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4467 (class 0 OID 0)
-- Dependencies: 249
-- Name: FUNCTION pg_create_subscription(arg_subscription_name text, arg_connection_string text, arg_publication_name text, arg_slot_name text, arg_slot_create boolean, arg_copy_data boolean); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.pg_create_subscription(arg_subscription_name text, arg_connection_string text, arg_publication_name text, arg_slot_name text, arg_slot_create boolean, arg_copy_data boolean) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4468 (class 0 OID 0)
-- Dependencies: 253
-- Name: FUNCTION pg_drop_subscription(arg_subscription_name text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.pg_drop_subscription(arg_subscription_name text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4469 (class 0 OID 0)
-- Dependencies: 255
-- Name: FUNCTION pg_list_all_subscriptions(); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.pg_list_all_subscriptions() TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4470 (class 0 OID 0)
-- Dependencies: 266
-- Name: FUNCTION pg_stat_replication_list(); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.pg_stat_replication_list() TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4471 (class 0 OID 0)
-- Dependencies: 256
-- Name: FUNCTION session_replication_role(arg_parameter text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.session_replication_role(arg_parameter text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4472 (class 0 OID 0)
-- Dependencies: 258
-- Name: FUNCTION set_auto_explain_log_analyze(arg_parameter text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.set_auto_explain_log_analyze(arg_parameter text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4473 (class 0 OID 0)
-- Dependencies: 262
-- Name: FUNCTION set_auto_explain_log_buffers(arg_parameter text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.set_auto_explain_log_buffers(arg_parameter text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4474 (class 0 OID 0)
-- Dependencies: 259
-- Name: FUNCTION set_auto_explain_log_format(arg_parameter text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.set_auto_explain_log_format(arg_parameter text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4475 (class 0 OID 0)
-- Dependencies: 260
-- Name: FUNCTION set_auto_explain_log_min_duration(arg_parameter text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.set_auto_explain_log_min_duration(arg_parameter text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4476 (class 0 OID 0)
-- Dependencies: 264
-- Name: FUNCTION set_auto_explain_log_nested_statements(arg_parameter text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.set_auto_explain_log_nested_statements(arg_parameter text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4477 (class 0 OID 0)
-- Dependencies: 261
-- Name: FUNCTION set_auto_explain_log_timing(arg_parameter text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.set_auto_explain_log_timing(arg_parameter text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4478 (class 0 OID 0)
-- Dependencies: 263
-- Name: FUNCTION set_auto_explain_log_verbose(arg_parameter text); Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT ALL ON FUNCTION aiven_extras.set_auto_explain_log_verbose(arg_parameter text) TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4479 (class 0 OID 0)
-- Dependencies: 2121
-- Name: FOREIGN DATA WRAPPER postgres_fdw; Type: ACL; Schema: -; Owner: postgres
--

GRANT ALL ON FOREIGN DATA WRAPPER postgres_fdw TO avnadmin WITH GRANT OPTION;


--
-- TOC entry 4481 (class 0 OID 0)
-- Dependencies: 229
-- Name: TABLE pg_stat_replication; Type: ACL; Schema: aiven_extras; Owner: postgres
--

GRANT SELECT ON TABLE aiven_extras.pg_stat_replication TO avnadmin WITH GRANT OPTION;


-- Completed on 2023-08-20 20:59:06

--
-- PostgreSQL database dump complete
--

