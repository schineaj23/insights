use core::time;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;
use tokio::time::Instant;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Team {
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

pub struct Collector {
    log_id_cache: HashSet<i32>,
    pub offset: i32,
}

#[allow(dead_code)]
impl Collector {
    pub fn new(offset: i32) -> Self {
        Collector {
            log_id_cache: HashSet::new(),
            offset: offset,
        }
    }

    pub async fn import_from_file(&mut self, path: &str) -> Result<i32, Box<dyn Error>> {
        let collected = tokio::fs::read_to_string(path)
            .await?
            .split("\n")
            .into_iter()
            .map(|x| self.log_id_cache.insert(x.parse::<i32>().unwrap()))
            .count();

        Ok(collected as i32)
    }

    pub async fn dump_cache_to_file(&self, path: &str) -> tokio::io::Result<()> {
        tokio::fs::write(path, self.log_id_cache.iter().join("\n")).await
    }

    pub fn get_logs(&self) -> &HashSet<i32> {
        &self.log_id_cache
    }

    pub async fn collect_log_ids_for_players(
        &mut self,
        player_ids: &Vec<String>,
    ) -> Result<i32, Box<dyn Error>> {
        let search_result = reqwest::get(format!(
            "http://logs.tf/api/v1/log?player={}",
            player_ids.join(",")
        ))
        .await?
        .json::<LogsResult>()
        .await?;

        let mut collected = 0;
        search_result.logs.iter().for_each(|log| {
            if log.date >= self.offset {
                self.log_id_cache.insert(log.id);
                collected += 1;
            }
        });

        Ok(collected)
    }

    pub async fn collect_permuation_of_players(
        &mut self,
        player_ids: Vec<String>,
        k: usize,
    ) -> Result<i32, Box<dyn Error>> {
        let mut collected = 0;
        let mut i = 1;

        for combination in player_ids.into_iter().combinations(k) {
            collected += self.collect_log_ids_for_players(&combination).await?;
            println!("Combination {}: collected {} unique logs", i, collected);
            i += 1;

            std::thread::sleep(time::Duration::from_millis(250));
        }

        Ok(collected)
    }

    pub async fn collect_all_team_logs(
        &mut self,
        team_map: HashMap<String, Team>,
    ) -> Result<i32, Box<dyn Error>> {
        for (team_name, team) in team_map.iter() {
            println!("Starting import of {}", team_name);
            let import_instant = Instant::now();

            let player_ids: Vec<String> = team.players.values().cloned().collect();

            self.collect_permuation_of_players(player_ids, 4).await?;

            println!(
                "Team {} collection time: {} seconds\n",
                team_name,
                import_instant.elapsed().as_secs_f32()
            );
        }
        Ok(self.log_id_cache.len() as i32)
    }
}
