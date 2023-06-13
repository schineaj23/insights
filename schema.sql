CREATE TABLE IF NOT EXISTS log(
    id INT PRIMARY KEY NOT NULL,
    map VARCHAR(64) NOT NULL,
    date INT NOT NULL,
    team_name_red VARCHAR(32),
    team_name_blu VARCHAR(32)
);