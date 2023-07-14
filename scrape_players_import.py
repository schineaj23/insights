from selenium import webdriver
from selenium.webdriver.common.by import By
import re
import time
import json

# Hopefully the comments were helpful for future me when this breaks!

# Firefox because this works on my install and Chrome doesn't work on WSL
options = webdriver.FirefoxOptions()
options.add_argument("-headless")
driver = webdriver.Firefox(options=options)

# r=40 will always be Sixes
standings_site = driver.get("https://rgl.gg/Public/LeagueTable.aspx?r=40")

selected_div = 0

# Get the list of all the division tables (0 = Invite, etc.. descending order)
divs = driver.find_elements(By.CSS_SELECTOR, ".table-striped tbody")
team_table = divs[selected_div].find_elements(By.CSS_SELECTOR, ".deco-none-excepthover")

teams = {}
flattened_players = {}

print(f"Starting import of division {selected_div}")

# Create list of teams and their IDs
for i, team_link in enumerate(team_table):
    if i == len(team_table) - 1:
        break
    name, link = str(team_link.text), team_link.get_attribute("href")
    id = re.search("\?t=([0-9]+)&", link).group(1)
    print(f"{name}: {id}")
    teams[name] = {"id": int(id), "players": {}}

for team_name, team in teams.items():
    driver.get(f"https://rgl.gg/Public/Team.aspx?t={team['id']}&r=40")

    # The player table is the first table in the layout
    player_table = driver.find_elements(By.CSS_SELECTOR, "tbody")[0]
    children = player_table.find_elements(By.CSS_SELECTOR, "tr")

    for i, row in enumerate(children):
        if i == 0 or i == len(children) - 1:
            continue

        # An "X" or "i" icon appears if the player is not paid up. (Skip these players)
        try:
            row.find_element(By.CSS_SELECTOR, ".glyphicon")
            print(f"{team_name}: Skipping non-starter")
            continue
        except:
            pass

        # The columns for each table row is laid out as follows:
        # | (starter/paid/banned/etc.) icon | name (link to profile) | date joined |
        cols = row.find_elements(By.CSS_SELECTOR, "td")

        # Grab the player name from text and SteamID64 from the anchor's href
        col = cols[1]
        name = col.text

        # If there is a verified check, the first link is an about page.
        # We want the second link in this case.
        index = len(col.find_elements(By.TAG_NAME, "a")) - 1
        link = col.find_elements(By.TAG_NAME, "a")[index].get_attribute("href")
        steam_id = str(re.search("\?p=([0-9]+)&", link).group(1))

        # Add to our fancy nested dict
        team["players"][name] = steam_id
        # ...and to the sloppy flattened SteamID to TeamID dict (for the importer)
        flattened_players[steam_id] = team["id"]

        print(f"{team_name}: +({name}:{steam_id})")
    time.sleep(1)

driver.quit()

with open("players_only_starters.json", "w", encoding="utf-8") as f:
    json.dump(teams, f)

with open("player_teamid_lut.json", "w", encoding="utf-8") as f:
    json.dump(flattened_players, f)

print("Completed generating team-player lists")
