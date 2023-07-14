import requests
import json
import time

# Gets all invite players, not sure how to filter season by divison yet.
# Putting teamIds in here manually
teams = {11489, 11304, 11319, 11413, 11389, 11293, 11460, 11490}
team_players = {}

with open("players.json", "w", encoding="utf-8") as f:
    for team in teams:
        print(f"Querying team {team}")
        r = requests.get(f"https://api.rgl.gg/v0/teams/{team}")
        j = r.json()
        print(r)

        n = {}
        for player in j["players"]:
            if player["leftAt"] == None:
                n[player["name"]] = player["steamId"]
        team_players[j["name"]] = n

        print(f"Added team {j['name']} to list")
        time.sleep(1)
    json.dump(team_players, f)
