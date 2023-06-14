CREATE TABLE IF NOT EXISTS team(
    team_id INTEGER PRIMARY KEY,
    team_name VARCHAR(50) NOT NULL
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