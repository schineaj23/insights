CREATE TABLE IF NOT EXISTS team(
    team_id INTEGER PRIMARY KEY,
    team_name VARCHAR(50) NOT NULL UNIQUE
);
CREATE TABLE IF NOT EXISTS log (
    log_id INTEGER PRIMARY KEY,
    unix_timestamp INTEGER NOT NULL,
    map VARCHAR(50) NOT NULL,
    red_team_id INTEGER NOT NULL,
    blu_team_id INTEGER NOT NULL,
    red_team_score INTEGER NOT NULL,
    blu_team_score INTEGER NOT NULL,
    FOREIGN KEY (red_team_id) REFERENCES team (team_id),
    FOREIGN KEY (blu_team_id) REFERENCES team (team_id)
);
CREATE TABLE IF NOT EXISTS player(
    id SERIAL PRIMARY KEY,
    steamid64 BIGINT NOT NULL UNIQUE,
    team_id INT REFERENCES team (team_id)
);
CREATE TABLE IF NOT EXISTS player_stats(
    id SERIAL PRIMARY KEY,
    log_id INTEGER NOT NULL,
    player_steamid64 BIGINT NOT NULL REFERENCES player(steamid64),
    kills INTEGER,
    deaths INTEGER,
    dmg INTEGER,
    dmg_real INTEGER,
    dt INTEGER,
    dt_real INTEGER,
    hr INTEGER,
    ubers INTEGER,
    drops INTEGER,
    headshots INTEGER,
    headshots_hit INTEGER,
    FOREIGN KEY (log_id) REFERENCES log (log_id)
);
CREATE TABLE IF NOT EXISTS team_stats(
    id SERIAL PRIMARY KEY,
    log_id INTEGER NOT NULL,
    team_id INTEGER NOT NUll,
    score INTEGER,
    kills INTEGER,
    deaths INTEGER,
    dmg INTEGER,
    charges INTEGER,
    drops INTEGER,
    first_caps INTEGER,
    caps INTEGER,
    FOREIGN KEY (log_id) REFERENCES log (log_id),
    FOREIGN KEY (team_id) REFERENCES team (team_id)
);
CREATE EXTENSION IF NOT EXISTS postgres_fdw;
CREATE SERVER IF NOT EXISTS insights_demos_server FOREIGN DATA WRAPPER postgres_fdw OPTIONS (
    host '127.0.0.1',
    dbname 'insights-dev-demos',
    port '5432'
);
\
set db_username 'usrname' \
set db_password 'passwrd' CREATE USER MAPPING FOR cat SERVER insights_demos_server OPTIONS (user :'db_username', password :'db_password');
IMPORT FOREIGN SCHEMA public
LIMIT TO demos
FROM SERVER insights_demos_server INTO public;
drop materialized view if exists connected_demos;
SET TIME ZONE 'UTC';
create materialized view connected_demos as
select demos.name,
    demos.id,
    demos.url,
    demos.created_at,
    log.log_id,
    log.red_team_id,
    log.blu_team_id,
    log.red_team_score,
    log.blu_team_score,
    log.map
from demos
    right join log on (
        demos.created_at between to_timestamp(log.unix_timestamp - 30)::timestamp without time zone and to_timestamp(log.unix_timestamp + 100)::timestamp without time zone
        and demos.map = log.map
        and abs(
            (demos."scoreBlue" - log.blu_team_score) + (log.red_team_score - demos."scoreRed")
        ) < 2
        and exists(
            select player_steamid64
            from player_stats
            where player_stats.log_id = log.log_id
                and cast(player_steamid64 as varchar) in (
                    select steamid
                    from users
                    where id in (
                            select user_id
                            from players
                            where demo_id = demos.id
                        )
                )
        )
    )
where log.map is not null
order by(log.log_id);