use std::{collections::HashMap, error::Error};

use serde::Deserialize;

const API_URL: &'static str = "https://api.rgl.gg/v0";

#[derive(Debug, Deserialize)]
pub struct Team {
    pub id: i32,
    pub tag: String,
    pub name: String,
    pub status: String,
    #[serde(rename(deserialize = "seasonId"))]
    pub season_id: i32,
    #[serde(rename(deserialize = "divisionId"))]
    pub divison_id: i32,
    #[serde(rename(deserialize = "divisionName"))]
    pub division_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    #[serde(rename(deserialize = "currentTeams"))]
    pub current_teams: HashMap<String, Option<crate::rgl::Team>>,
    pub name: String,
}

pub async fn search_player(player_id: &str) -> Result<Player, Box<dyn Error>> {
    let resp = reqwest::get(format!("{API_URL}/profile/{}", player_id)).await?;
    let parsed = resp.json::<Player>().await?;
    Ok(parsed)
}
