import requests
import json
import time

# Gets all invite players, not sure how to filter season by divison yet.
# Putting teamIds in here manually
teams = {11489, 11304, 11319, 11413, 11389, 11293, 11460, 11490}
team_players = {}

with open("player_teamid_lut.json", "w", encoding="utf-8") as f:
    for team in teams:
        print(f"Querying team {team}")
        r = requests.get(f"https://api.rgl.gg/v0/teams/{team}")
        j = r.json()
        print(r)

        for player in j["players"]:
            team_players[player["steamId"]] = j["teamId"]

        print(f"Added team {j['name']} to list")
        time.sleep(0.2)
    json.dump(team_players, f)
