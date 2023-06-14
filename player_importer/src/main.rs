mod steam_id;

use cached::proc_macro::cached;
use core::time;
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::BufReader};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Team {
    id: i32,
    players: HashMap<String, String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LogsResult {
    logs: Vec<LogView>,
    parameters: Value,
    results: i32,
    success: bool,
    total: i32,
}

// Calling this a "log view" since it is the small metadata returned from the log search api
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct LogView {
    date: i32,
    id: i32,
    map: String,
    players: i32,
    title: String,
    views: i32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PlayerStats {
    team: String,
    class_stats: Vec<ClassStats>,
    kills: i16,
    deaths: i16,
    assists: i16,
    dmg: i16,
    dmg_real: i16,
    dt: i16,
    dt_real: i16,
    hr: i16,
    ubers: i16,
    drops: i16,
    headshots: i16,
    headshots_hit: i16,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ClassStats {
    r#type: String,
    kills: i16,
    assists: i16,
    deaths: i16,
    dmg: i32,
    weapon: Value,
    total_time: i32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct TeamData {
    score: i8,
    kills: i8,
    deaths: i8,
    dmg: i16,
    charges: i8,
    drops: i8,
    firstcaps: i8,
    caps: i8,
}

// Log result by logs.tf/json/:id
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LogSerialized {
    version: i8,
    teams: Value,
    length: i16,
    players: HashMap<String, PlayerStats>,
    names: Value,
    rounds: Value,
    info: Value,
}

#[cached(size = 50)]
async fn fetch_team_id_for_player(player_id: String) -> i64 {
    let res: HashMap<String, Value> =
        match reqwest::get(format!("https://api.rgl.gg/v0/profile/{}", player_id)).await {
            Ok(response) => match response.json().await {
                Ok(map) => map,
                Err(_) => return 0,
            },
            Err(_) => return 0,
        };

    res["currentTeams"]["sixes"]["id"].as_i64().unwrap()
}

async fn get_first_duplicate_id(player_ids: Vec<String>) -> Option<i64> {
    let mut last_id: i64 = 0;
    for player in player_ids {
        let id = fetch_team_id_for_player(player).await;
        if id == last_id {
            return Some(id);
        }
        last_id = id;
    }
    None
}

async fn determine_team_ids_for_match(
    player_map: HashMap<String, PlayerStats>,
) -> Result<(i32, i32), Box<dyn std::error::Error>> {
    println!("determine_team_ids_for_match called");
    // Separate teams
    let mut red_players: Vec<String> = Vec::new();
    let mut blu_players: Vec<String> = Vec::new();

    for (id, player) in player_map.iter() {
        let id64 = steam_id::from_steamid3(id.clone()).unwrap().to_string();
        if player.team.eq("Red") {
            red_players.push(id64);
        } else {
            blu_players.push(id64);
        }
    }

    // Coming up with the naive solution until performance is an issue!

    let red_id = get_first_duplicate_id(red_players).await.unwrap();
    let blu_id = get_first_duplicate_id(blu_players).await.unwrap();

    println!("Found Red: {} Blue: {}", red_id, blu_id);

    Ok((red_id as i32, blu_id as i32))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // this should have the players based on who is rostered via rgl api
    // but since there are more rostered players than starters, use curated list

    let file = match File::open("C:\\Users\\drew\\project\\insights\\players_only_starters.json") {
        Ok(file) => file,
        Err(error) => panic!("Error opening file {:?}", error),
    };

    let reader = BufReader::new(file);

    let team_map: HashMap<String, Team> = serde_json::from_reader(reader)?;
    println!("{:#?}", team_map.keys());

    // for entry in team_list {
    //     let (team_name, team) = (entry.0, entry.1);
    //     println!("Team Name: {}", team_name);
    // }

    let player_ids: Vec<String> = team_map
        .get("froyotech")
        .unwrap()
        .players
        .values()
        .cloned()
        .collect();
    println!("{:?}", player_ids);

    // Should also compute permutations for scrims.
    // Scrims defined as 4+ of the team playing on the same team (during scrim times) to account for ringers
    // Add all these logs, use db to filter out duplicates. the amount of api calls -> nCr = n! / r!(n-r)!

    let mut logs_cache: HashMap<i32, LogView> = HashMap::new();

    let mut i = 1;
    for combination in player_ids.into_iter().combinations(6) {
        println!("{}: Trying {:?}", i, combination);
        i += 1;

        let url = format!("http://logs.tf/api/v1/log?player={}", combination.join(","));
        println!("{}", url);

        let resp_json: LogsResult = reqwest::get(url).await?.json::<LogsResult>().await?;
        // println!("{:#?}", resp_json);

        // Add logs to log cache, which we will be adding to our DB
        resp_json.logs.iter().for_each(|x| {
            // Mon May 15 2023 03:59:00 GMT+0000 (Team Registration Deadline)
            // offset does not appear to be working...
            if x.date > 1684123140 {
                logs_cache.insert(x.id, x.clone());
            }
        });

        // Sleep so we don't get rate limited
        std::thread::sleep(time::Duration::from_millis(1000));
    }

    println!("Likely Scrim Log Count: {}", logs_cache.len());

    let random_log = match logs_cache.iter().next() {
        Some(x) => x.0,
        None => panic!("uh oh"),
    };
    let log: LogSerialized = reqwest::get(format!("http://logs.tf/json/{}", random_log))
        .await?
        .json::<LogSerialized>()
        .await?;
    println!("{:#?}", log);

    let (red_team_id, blu_team_id) = determine_team_ids_for_match(log.players).await?;
    // Now construct our sql query and put this into the database

    Ok(())
}
