mod db;
mod log;
mod steam_id;

use cached::proc_macro::cached;
use core::time;
use dotenv::dotenv;
use itertools::Itertools;
use log::PlayerStats;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    time::Instant,
};

use crate::log::LogSerialized;

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

#[cached(size = 50)]
async fn fetch_team_id_for_player(player_id: String) -> Option<i64> {
    match get_team_id_from_player_lut(&player_id) {
        Some(team_id) => {
            println!("Found team_id from LUT");
            Some(team_id)
        }
        _ => {
            println!("Could not find team_id from LUT, sending req to RGL API");
            let res: HashMap<String, Value> =
                match reqwest::get(format!("https://api.rgl.gg/v0/profile/{}", player_id)).await {
                    Ok(response) => match response.json().await {
                        Ok(map) => map,
                        Err(_) => return None,
                    },
                    Err(_) => return None,
                };

            res["currentTeams"]["sixes"]["id"].as_i64()
        }
    }
}

fn get_team_id_from_player_lut(player_id: &str) -> Option<i64> {
    // TODO: cache this entire file in memory instead of reopening
    let file = match File::open("/home/cat/src/insighttf/player_teamid_lut.json") {
        Ok(x) => x,
        Err(_) => return None,
    };
    let reader: BufReader<File> = BufReader::new(file);
    let map: HashMap<String, i64> = match serde_json::from_reader(reader) {
        Ok(x) => x,
        Err(_) => return None,
    };
    match map.get(player_id) {
        Some(team_id) => Some(*team_id),
        _ => None,
    }
}

async fn determine_team_ids_for_match(
    player_map: &HashMap<String, PlayerStats>,
) -> Result<(i32, i32), Box<dyn std::error::Error>> {
    println!("determine_team_ids_for_match called");
    // Separate teams

    let mut last_red_id = 0;
    let mut last_blu_id = 0;
    let mut visited = 0;
    for (id, player) in player_map.iter() {
        let id64 = steam_id::from_steamid3(id.clone()).unwrap().to_string();
        let team_id = match fetch_team_id_for_player(id64).await {
            Some(id) => id,
            None => {
                println!(
                    "Could not find team id for player {} (are they not rostered?)",
                    id
                );
                continue;
            }
        };

        if last_red_id == team_id || last_blu_id == team_id {
            visited += 1;
        }

        match player.team.as_str() {
            "Red" => {
                last_red_id = team_id;
            }
            "Blue" => {
                last_blu_id = team_id;
            }
            _ => {}
        };

        if visited >= 4 {
            break;
        }
    }

    println!("Found Red: {} Blue: {}", last_red_id, last_blu_id);

    Ok((last_red_id as i32, last_blu_id as i32))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // this should have the players based on who is rostered via rgl api
    // but since there are more rostered players than starters, use curated list

    let path = std::env::var("PLAYER_LIST_PATH").expect("PLAYER_LIST_PATH not set");
    let file = File::open(path)?;

    let reader = BufReader::new(file);

    let team_map: HashMap<String, Team> = serde_json::from_reader(reader)?;
    println!("{:#?}", team_map.keys());

    let pool = db::connect().await?;

    for (team_name, team_data) in team_map.iter() {
        db::insert_team(&pool, &team_name, &team_data.id).await?;
    }

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

    let mut logs_cache: HashSet<i32> = HashSet::new();

    let start_of_import = Instant::now();

    let mut i = 1;
    let mut duplicates = 0;
    for combination in player_ids.into_iter().combinations(4) {
        println!("{}: Trying {:?}", i, combination);
        i += 1;

        let url = format!("http://logs.tf/api/v1/log?player={}", combination.join(","));
        println!("{}", url);

        let resp_json: LogsResult = reqwest::get(url).await?.json::<LogsResult>().await?;

        // Add logs to log cache, which we will be adding to our DB
        resp_json.logs.iter().for_each(|x| {
            // Mon May 15 2023 03:59:00 GMT+0000 (Team Registration Deadline)
            // offset does not appear to be working...
            if x.date > 1684123140 {
                if !logs_cache.insert(x.id) {
                    duplicates += 1;
                }
            }
        });

        // Sleep so we don't get rate limited
        std::thread::sleep(time::Duration::from_millis(500));
    }

    for log_id in logs_cache.iter() {
        println!("Making request to logs.tf");

        let start_time = Instant::now();

        let log: LogSerialized = reqwest::get(format!("http://logs.tf/json/{}", log_id))
            .await?
            .json::<LogSerialized>()
            .await?;
        println!("Request recieved from logs.tf");

        let (red_team_id, blu_team_id) = determine_team_ids_for_match(&log.players).await?;

        db::insert_log(&pool, &log_id, &(red_team_id, blu_team_id), &log).await?;

        let duration = start_time.elapsed();
        println!("Added log {} to db", log_id);
        println!("Individual log took {} seconds", duration.as_secs_f32());
    }

    println!("Likely Scrim Log Count: {}", logs_cache.len());
    println!("Duplicate logs: {}", duplicates);
    println!(
        "Total import of log time for froyotech: {} seconds",
        start_of_import.elapsed().as_secs_f32()
    );
    Ok(())
}
