use core::time;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use itertools::Itertools;
use serde::Deserialize;
use tokio::time::Instant;

use crate::log::search_by_players;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Team {
    pub id: i32,
    pub players: HashMap<String, String>,
}

pub struct Collector {
    log_id_cache: HashSet<i32>,
    pub start: i32,
    pub end: Option<i32>,
}

impl Collector {
    pub fn new(start: i32, end: Option<i32>) -> Self {
        Collector {
            log_id_cache: HashSet::new(),
            start: start,
            end: end,
        }
    }

    pub async fn import_from_file(&mut self, path: &str) -> Result<i32, Box<dyn Error>> {
        let collected = tokio::fs::read_to_string(path)
            .await?
            .split(",")
            .into_iter()
            .map(|x| self.log_id_cache.insert(x.parse::<i32>().unwrap()))
            .count();

        Ok(collected as i32)
    }

    pub async fn dump_cache_to_file(&self, path: &str) -> tokio::io::Result<()> {
        tokio::fs::write(path, self.log_id_cache.iter().join(",")).await
    }

    pub fn get_logs(&self) -> &HashSet<i32> {
        &self.log_id_cache
    }

    pub async fn collect_log_ids_for_players(
        &mut self,
        player_ids: &Vec<String>,
    ) -> Result<i32, Box<dyn Error>> {
        let search_result = search_by_players(&player_ids.join(",")).await?;

        let mut collected = 0;
        search_result.logs.iter().for_each(|log| {
            // todo:sixes
            if log.players < 12 || log.players > 14 {
                return;
            }

            if self.end.is_some() && log.date > self.end.unwrap() {
                return;
            }

            if log.date >= self.start {
                collected += self.log_id_cache.insert(log.id) as i32;
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

        for (i, combination) in player_ids.into_iter().combinations(k).enumerate() {
            let found = self.collect_log_ids_for_players(&combination).await?;
            println!("Combination {}: collected {} new logs", i, found);
            collected += found;

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

            let collected = self.collect_permuation_of_players(player_ids, 4).await?;

            println!(
                "Team {} collection time: {} seconds for {} logs\n",
                team_name,
                import_instant.elapsed().as_secs_f32(),
                collected
            );
        }
        Ok(self.log_id_cache.len() as i32)
    }
}
