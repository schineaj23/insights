use serde::{Deserialize, Serialize};
use std::error::Error;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct DemoSerialized {
    pub id: i32,
    pub url: String,
    pub name: String,
    pub server: String,
    pub duration: i32,
    pub nick: String,
    pub map: String,
    pub red: String,
    pub blue: String,
    #[serde(rename(deserialize = "redScore"))]
    pub red_score: i32,
    #[serde(rename(deserialize = "blueScore"))]
    pub blue_score: i32,
    #[serde(rename(deserialize = "playerCount"))]
    pub player_count: i32,
    pub uploader: i32,
    pub hash: String,
    pub backend: String,
    pub path: String,
}

// Searches for demo based on map and players for 6v6.
// Effectively replaces db::get_connected_demo if log is known
pub async fn search_demo(map: &str, players: &str) -> Result<Vec<DemoSerialized>, Box<dyn Error>> {
    let url = format!(
        "https://api.demos.tf/demos?map={}&type=6v6&players[]={}",
        map, players
    );

    let req = reqwest::get(url).await?;
    let demos = req.json::<Vec<DemoSerialized>>().await?;
    Ok(demos)
}
