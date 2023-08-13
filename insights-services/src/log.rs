use std::{collections::HashMap, error::Error};

use serde::Deserialize;
use serde_json::Value;

const API_URL: &'static str = "http://logs.tf/api/v1/log";

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PlayerStats {
    pub team: String,
    pub class_stats: Vec<ClassStats>,
    pub kills: i16,
    pub deaths: i16,
    pub assists: i16,
    pub dmg: i32,
    pub dmg_real: i32,
    pub dt: i32,
    pub dt_real: i32,
    pub hr: i32,
    pub ubers: i16,
    pub drops: i16,
    pub headshots: i16,
    pub headshots_hit: i16,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ClassStats {
    pub r#type: String,
    pub kills: i16,
    pub assists: i16,
    pub deaths: i16,
    pub dmg: i32,
    pub weapon: Value,
    pub total_time: i32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct TeamStats {
    pub score: i16,
    pub kills: i16,
    pub deaths: i16,
    pub dmg: i32,
    pub charges: i16,
    pub drops: i8,
    pub firstcaps: i8,
    pub caps: i8,
}

// Log result by logs.tf/json/:id
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct LogSerialized {
    pub version: i8,
    pub teams: HashMap<String, TeamStats>,
    pub length: i16,
    pub players: HashMap<String, PlayerStats>,
    pub names: Value,
    pub rounds: Value,
    pub info: LogInfo,
}

// Calling this a "log view" since it is the small metadata returned from the log search api
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct LogView {
    pub date: i32,
    pub id: i32,
    pub map: String,
    pub players: i32,
    pub title: String,
    pub views: i32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub logs: Vec<LogView>,
    pub parameters: Value,
    pub results: i32,
    pub success: bool,
    pub total: i32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct LogInfo {
    pub map: String,
    pub date: i32,
}

pub async fn search_by_players(players: &str) -> Result<SearchResult, Box<dyn Error>> {
    let res = reqwest::get(format!("{API_URL}?player={players}")).await?;
    let search = res.json::<SearchResult>().await?;
    Ok(search)
}

pub async fn fetch_log(log_id: &i32) -> Result<LogSerialized, Box<dyn Error>> {
    let res = reqwest::get(format!("{API_URL}/{log_id}")).await?;
    let log = res.json::<LogSerialized>().await?;
    Ok(log)
}
