mod collect;
mod db;
mod log;
mod rgl;
mod steam_id;

use cached::proc_macro::cached;
use dotenv::dotenv;
use log::PlayerStats;
use reqwest::Response;
use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::BufReader};
use tokio::time::Instant;

use crate::{collect::Collector, log::LogSerialized};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Team {
    id: i32,
    players: HashMap<String, String>,
}

#[cached(size = 200)]
async fn fetch_team_id_for_player(player_id: String) -> Option<i32> {
    match get_team_id_from_player_lut(&player_id) {
        Some(team_id) => {
            println!("Found team_id from LUT");
            Some(team_id)
        }
        None => {
            println!(
                "Could not find team_id from LUT, asking RGL API for player {}",
                player_id
            );
            let res: Response =
                reqwest::get(format!("https://api.rgl.gg/v0/profile/{}", player_id))
                    .await
                    .ok()?;
            let player = res.json::<rgl::Player>().await.ok()?;
            let team = player.current_teams.get("sixes")?;

            match team {
                Some(team) => Some(team.id),
                _ => None,
            }
        }
    }
}

fn get_team_id_from_player_lut(player_id: &str) -> Option<i32> {
    // TODO: cache this entire file in memory instead of reopening
    let file = File::open("/home/cat/src/insighttf/player_teamid_lut.json").ok()?;
    let reader: BufReader<File> = BufReader::new(file);
    let id_map: HashMap<String, i32> = serde_json::from_reader(reader).ok()?;

    id_map.get(player_id).map(|x| *x)
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
        let id64 = steam_id::from_steamid3(id).unwrap().to_string();
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

    Ok((last_red_id, last_blu_id))
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

    // Inserting teams
    // for (team_name, team_data) in team_map.iter() {
    //     db::insert_team(&pool, &team_name, &team_data.id).await?;
    // }

    // TODO: make the collecting and the adding on separate services so they don't block each other

    // Mon May 15 2023 03:59:00 GMT+0000 (Team Registration Deadline)
    // logs.tf offset does not appear to be working...
    let mut collector = Collector::new(1684123140);
    println!(
        "Collected {} logs from file",
        collector.import_from_file("log_cache.txt").await?
    );
    // let collected = collector.collect_all_team_logs(team_map).await?;

    let logs_cache = collector.get_logs();

    let log_collection_start = Instant::now();

    let log_collection_time = log_collection_start.elapsed();
    let log_insert_start = Instant::now();

    let pool = db::connect().await?;

    for log_id in logs_cache.iter() {
        println!("Making request to logs.tf");

        let start_time = Instant::now();

        let log: LogSerialized = reqwest::get(format!("http://logs.tf/json/{}", log_id))
            .await?
            .json::<LogSerialized>()
            .await?;
        println!(
            "Request recieved from logs.tf in {} seconds",
            start_time.elapsed().as_secs_f32()
        );

        let (red_team_id, blu_team_id) = determine_team_ids_for_match(&log.players).await?;

        db::insert_log(&pool, &log_id, &(red_team_id, blu_team_id), &log).await?;
        println!("Added log {} to db", log_id);

        for (player_id, stats) in log.players.iter() {
            let start_time = Instant::now();

            let player_id = steam_id::from_steamid3(player_id).unwrap();

            // If it is a ringer, just ignore we don't care. It would probably throw an error in DB anyways
            let team_id = match fetch_team_id_for_player(player_id.to_string()).await {
                Some(id) => id,
                None => {
                    println!("Skipping ringer {}", player_id);
                    continue;
                }
            };

            db::insert_player(&pool, &player_id, &team_id).await?;
            db::insert_player_stats(&pool, log_id, &player_id, stats).await?;

            println!(
                "Took {} seconds to insert player stats",
                start_time.elapsed().as_secs_f32()
            );
        }
    }

    println!("Likely scrim log count: {}", logs_cache.len());
    println!(
        "Log collection time: {} seconds",
        log_collection_time.as_secs_f32()
    );
    println!(
        "Log insert time: {} seconds",
        log_insert_start.elapsed().as_secs_f32()
    );
    println!(
        "Total time elapsed: {} seconds",
        log_collection_start.elapsed().as_secs_f32()
    );

    Ok(())
}
