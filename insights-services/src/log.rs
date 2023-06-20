use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

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

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct LogInfo {
    pub map: String,
    pub date: i32,
}
