## Current Full Procedure for Importing Seasons

1) Go to RGL page for season that you want and find season ID from url
2) Run generator (`scrape_players_import.py`) with arguments `-s (season id)`
3) Get season start/end dates from corresponding RGL articles and convert to unix timestamps
4) Pass the files outputted from generator into the importer
5) `importer -t {path to team/player json} -l {path to player lut json} -s {season start unixtime} -e {season end unixtime} -f (fetch new logs) -d (dump to file just in case)`
6) Make sure everything looks correct (that the number of logs seem accurate, the logs don't look like pugs)
7) Create the teams in the DB with the teamporter, using the season number not season ID (will fix) `teamporter -s {season number} -t {path to team/player json}`
    - FIXME: Add season_id column
8) Insert the logs into the DB with `importer -t {path to team/player json} -l {path to player lut json} -s {season start unixtime} -e {season end unixtime} -r {path to dumped_logs.csv} -i`
9) yippie :D hopefully nothing broke!