use core::time;
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::BufReader};

#[derive(Debug, Deserialize)]
struct Team {
    #[serde(flatten)]
    players: HashMap<String, String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LogsResult {
    logs: Vec<Log>,
    parameters: Value,
    results: i32,
    success: bool,
    total: i32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct Log {
    date: i64,
    id: i32,
    map: String,
    players: i32,
    title: String,
    views: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // this should have the players based on who is rostered via rgl api
    // but since there are more rostered players than starters, use curated list

    let file = match File::open("/home/cat/src/insighttf/players_only_starters.json") {
        Ok(file) => file,
        Err(error) => panic!("Error opening file {:?}", error),
    };

    let reader = BufReader::new(file);

    let team_list: HashMap<String, Team> = serde_json::from_reader(reader)?;
    println!("{:#?}", team_list.keys());

    // for entry in team_list {
    //     let (team_name, team) = (entry.0, entry.1);
    //     println!("Team Name: {}", team_name);
    // }

    let player_ids: Vec<String> = team_list
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

    let mut logs: HashMap<i32, Log> = HashMap::new();

    let mut i = 1;
    for combination in player_ids.into_iter().combinations(4) {
        println!("{:?} COMBINATION ITERATION {}", combination, i);
        i += 1;

        let url = format!("http://logs.tf/api/v1/log?player={}", combination.join(","));
        println!("{}", url);

        let resp = reqwest::get(url).await?;
        let resp_json: LogsResult = resp.json::<LogsResult>().await?;
        // println!("{:#?}", resp_json);

        // Add logs to log cache, which we will be adding to our DB
        resp_json.logs.iter().for_each(|x| {
            logs.insert(x.id, x.clone());
        });

        // Sleep so we don't get rate limited
        std::thread::sleep(time::Duration::from_millis(500));
    }

    println!("Unique Log Count: {}", logs.len());

    let logs_filtered: HashMap<&i32, &Log> = logs
        .iter()
        .filter(|(_, log)| {
            // Mon May 15 2023 03:59:00 GMT+0000 (Team Registration Deadline)
            log.date > 1684123140
        })
        .collect();

    // println!("{:?}", logs_filtered);
    println!("{:?}", logs_filtered.len());

    Ok(())
}
