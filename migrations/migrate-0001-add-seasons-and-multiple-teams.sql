alter table team add column season integer;
update team set season = 12;
alter table player add column teams integer[];
update player set teams = ARRAY[team_id];
alter table player drop column team_id;