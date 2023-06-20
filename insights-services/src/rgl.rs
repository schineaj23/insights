use std::collections::HashMap;

use serde::Deserialize;

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
}
