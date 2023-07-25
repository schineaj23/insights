--
-- PostgreSQL database dump
--

-- Dumped from database version 15.3
-- Dumped by pg_dump version 15.3

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
-- Name: public; Type: SCHEMA; Schema: -; Owner: avnadmin
--

CREATE SCHEMA public;


ALTER SCHEMA public OWNER TO avnadmin;

--
-- Name: SCHEMA public; Type: COMMENT; Schema: -; Owner: avnadmin
--

COMMENT ON SCHEMA public IS 'standard public schema';


--
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

SET default_tablespace = '';

SET default_table_access_method = heap;

--
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
-- Name: bomb_attempt_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: avnadmin
--

ALTER SEQUENCE public.bomb_attempt_id_seq OWNED BY public.bomb_attempt.id;


--
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
-- Name: player; Type: TABLE; Schema: public; Owner: avnadmin
--

CREATE TABLE public.player (
    id integer NOT NULL,
    steamid64 bigint NOT NULL,
    team_id integer
);


ALTER TABLE public.player OWNER TO avnadmin;

--
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
-- Name: player_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: avnadmin
--

ALTER SEQUENCE public.player_id_seq OWNED BY public.player.id;


--
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
-- Name: player_stats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: avnadmin
--

ALTER SEQUENCE public.player_stats_id_seq OWNED BY public.player_stats.id;


--
-- Name: team; Type: TABLE; Schema: public; Owner: avnadmin
--

CREATE TABLE public.team (
    team_id integer NOT NULL,
    team_name character varying(50) NOT NULL
);


ALTER TABLE public.team OWNER TO avnadmin;

--
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
-- Name: team_stats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: avnadmin
--

ALTER SEQUENCE public.team_stats_id_seq OWNED BY public.team_stats.id;


--
-- Name: bomb_attempt id; Type: DEFAULT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.bomb_attempt ALTER COLUMN id SET DEFAULT nextval('public.bomb_attempt_id_seq'::regclass);


--
-- Name: player id; Type: DEFAULT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player ALTER COLUMN id SET DEFAULT nextval('public.player_id_seq'::regclass);


--
-- Name: player_stats id; Type: DEFAULT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player_stats ALTER COLUMN id SET DEFAULT nextval('public.player_stats_id_seq'::regclass);


--
-- Name: team_stats id; Type: DEFAULT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.team_stats ALTER COLUMN id SET DEFAULT nextval('public.team_stats_id_seq'::regclass);


--
-- Name: bomb_attempt bomb_attempt_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.bomb_attempt
    ADD CONSTRAINT bomb_attempt_pkey PRIMARY KEY (id);


--
-- Name: log log_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.log
    ADD CONSTRAINT log_pkey PRIMARY KEY (log_id);


--
-- Name: player player_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player
    ADD CONSTRAINT player_pkey PRIMARY KEY (id);


--
-- Name: player_stats player_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player_stats
    ADD CONSTRAINT player_stats_pkey PRIMARY KEY (id);


--
-- Name: player player_steamid64_key; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player
    ADD CONSTRAINT player_steamid64_key UNIQUE (steamid64);


--
-- Name: team team_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.team
    ADD CONSTRAINT team_pkey PRIMARY KEY (team_id);


--
-- Name: team_stats team_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.team_stats
    ADD CONSTRAINT team_stats_pkey PRIMARY KEY (id);


--
-- Name: team team_team_name_key; Type: CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.team
    ADD CONSTRAINT team_team_name_key UNIQUE (team_name);


--
-- Name: bomb_attempt bomb_attempt_log_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.bomb_attempt
    ADD CONSTRAINT bomb_attempt_log_id_fkey FOREIGN KEY (log_id) REFERENCES public.log(log_id);


--
-- Name: bomb_attempt bomb_attempt_player_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.bomb_attempt
    ADD CONSTRAINT bomb_attempt_player_id_fkey FOREIGN KEY (player_id) REFERENCES public.player(steamid64);


--
-- Name: player_stats player_stats_log_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player_stats
    ADD CONSTRAINT player_stats_log_id_fkey FOREIGN KEY (log_id) REFERENCES public.log(log_id);


--
-- Name: player_stats player_stats_player_steamid64_fkey; Type: FK CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.player_stats
    ADD CONSTRAINT player_stats_player_steamid64_fkey FOREIGN KEY (player_steamid64) REFERENCES public.player(steamid64);


--
-- Name: team_stats team_stats_log_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: avnadmin
--

ALTER TABLE ONLY public.team_stats
    ADD CONSTRAINT team_stats_log_id_fkey FOREIGN KEY (log_id) REFERENCES public.log(log_id);


--
-- PostgreSQL database dump complete
--

