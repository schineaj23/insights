import argparse
from typing import Optional
import unittest
from selenium import webdriver
from selenium.webdriver.common.by import By
import re
import time
import json


# Hopefully the comments were helpful for future me when this breaks!
class PlayerGenerator:
    def __init__(self, driver) -> None:
        self.driver = driver

        self.teams = dict()
        self.flattened_players = dict()

    def set_page(self, url: Optional[str]) -> str:
        # r=40 will always be Sixes
        # therefore https://rgl.gg/Public/LeagueTable.aspx?r=40
        if url is None:
            url = "https://rgl.gg/Public/LeagueTable.aspx?r=40"
        self.standings_site = self.driver.get(url)

    def add_player(self, team_name, team, row):
        # An "X" or "i" icon appears if the player is not paid up. (Skip these players)
        try:
            row.find_element(By.CSS_SELECTOR, ".glyphicon")
            print(f"{team_name}: Skipping non-starter")
            return
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
        self.flattened_players[steam_id] = team["id"]

        print(f"{team_name}: +({name}:{steam_id})")

    def add_division(self, division: int) -> None:
        # Get the list of all the division tables (0 = Invite, etc.. descending order)
        divs = self.driver.find_elements(By.CSS_SELECTOR, ".table-striped tbody")
        team_table = divs[division].find_elements(
            By.CSS_SELECTOR, ".deco-none-excepthover"
        )

        print(f"Starting import of division {division}")

        # Create list of teams and their IDs
        for i, team_link in enumerate(team_table):
            if i == len(team_table) - 1:
                break
            name, link = str(team_link.text), team_link.get_attribute("href")
            id = re.search("\?t=([0-9]+)&", link).group(1)
            print(f"{name}: {id}")
            self.teams[name] = {"id": int(id), "players": {}}

        for team_name, team in self.teams.items():
            print(f"Redirecting to {team_name}")
            self.driver.get(f"https://rgl.gg/Public/Team.aspx?t={team['id']}&r=40")

            # The player table is the first table in the layout
            player_table = self.driver.find_elements(By.CSS_SELECTOR, "tbody")[0]
            children = player_table.find_elements(By.CSS_SELECTOR, "tr")

            for i, row in enumerate(children):
                if i == 0 or i == len(children) - 1:
                    continue
                self.add_player(team_name, team, row)
            time.sleep(1)

        print("Completed generating team-player lists")
        self.driver.quit()

    def save(self):
        with open("players_only_starters.json", "w", encoding="utf-8") as f:
            json.dump(self.teams, f)

        with open("player_teamid_lut.json", "w", encoding="utf-8") as f:
            json.dump(self.flattened_players, f)


class TestScraper(unittest.TestCase):
    @classmethod
    def setUpClass(self):
        # Firefox because this works on my install and Chrome doesn't work on WSL
        options = webdriver.FirefoxOptions()
        options.add_argument("-headless")
        driver = webdriver.Firefox(options=options)

        self.gen = PlayerGenerator(driver)
        self.gen.set_page("https://rgl.gg/Public/LeagueTable.aspx?s=139&r=40")
        self.gen.add_division(0)

    def test_teams_invite(self):
        with open("generators/test/players_only_starters.json", "r") as f:
            test_teams = json.load(f)

            self.assertEqual(len(test_teams), len(self.gen.teams))
            self.assertEqual(test_teams, self.gen.teams)

    def test_player_lut(self):
        with open("generators/test/player_teamid_lut.json", "r") as f:
            test_flattened_players = json.load(f)

            self.assertEqual(
                len(test_flattened_players), len(self.gen.flattened_players)
            )
            self.assertEqual(test_flattened_players, self.gen.flattened_players)


def main():
    parser = argparse.ArgumentParser(
        prog="Generator",
        description="Generates player/team information for given season and division for RGL sixes.",
    )
    parser.add_argument(
        "-d",
        "--division",
        help="Division to select where invite/highest div = 0",
        default=0,
        type=int,
    )
    parser.add_argument(
        "-s",
        "--season",
        help="Desired RGL season ID (not the season number)",
        default=-1,
        type=int,
    )
    args = vars(parser.parse_args())

    options = webdriver.FirefoxOptions()
    options.add_argument("--allow-hosts")
    options.add_argument("-headless")
    driver = webdriver.Firefox(options=options)
    gen = PlayerGenerator(driver)

    # Default to current season
    if args["season"] == -1:
        gen.set_page("https://rgl.gg/Public/LeagueTable.aspx?r=40")
    else:
        gen.set_page(f"https://rgl.gg/Public/LeagueTable.aspx?s={args['season']}&r=40")

    gen.add_division(args["division"])
    gen.save()


if __name__ == "__main__":
    main()
