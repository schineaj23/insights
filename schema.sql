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