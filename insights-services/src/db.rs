use std::error::Error;

use sqlx::{FromRow, Postgres};

use crate::log::{LogSerialized, PlayerStats};

// TODO: create error enums

pub async fn insert_team(
    pool: &sqlx::PgPool,
    team_name: &str,
    team_id: &i32,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        "insert into team (team_id, team_name) VALUES ($1, $2) on conflict (team_id) do nothing",
    )
    .bind(&team_id)
    .bind(&team_name)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_log(
    pool: &sqlx::PgPool,
    log_id: &i32,
    teams: &(i32, i32),
    log_data: &LogSerialized,
) -> Result<(), Box<dyn Error>> {
    let red_score = log_data.teams.get("Red").unwrap().score as i32;
    let blu_score = log_data.teams.get("Blue").unwrap().score as i32;

    sqlx::query(
        "insert into log (log_id, unix_timestamp, map, red_team_id, blu_team_id, red_team_score, blu_team_score) values ($1, $2, $3, $4, $5, $6, $7) on conflict (log_id) do nothing")
        .bind(&log_id)
        .bind(&log_data.info.date)
        .bind(&log_data.info.map)
        .bind(&teams.0)
        .bind(&teams.1)
        .bind(&red_score)
        .bind(&blu_score)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn insert_player(
    pool: &sqlx::PgPool,
    steam_id: &i64,
    team_id: &i32,
) -> Result<(), Box<dyn Error>> {
    sqlx::query("insert into player (steamid64, team_id) values ($1, $2) on conflict do nothing")
        .bind(&steam_id)
        .bind(&team_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn insert_player_stats(
    pool: &sqlx::PgPool,
    log_id: &i32,
    player_id: &i64,
    stats: &PlayerStats,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        "insert into player_stats (log_id, player_steamid64, kills, deaths, dmg, dmg_real, dt, dt_real, hr, ubers, drops, headshots, headshots_hit) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)")
        .bind(&log_id)
        .bind(&player_id)
        .bind(&(stats.kills as i32))
        .bind(&(stats.deaths as i32))
        .bind(&stats.dmg)
        .bind(&stats.dmg_real)
        .bind(&stats.dt)
        .bind(&stats.dt_real)
        .bind(&stats.hr)
        .bind(&(stats.ubers as i32))
        .bind(&(stats.drops as i32))
        .bind(&(stats.headshots as i32))
        .bind(&(stats.headshots_hit as i32))
        .execute(pool)
        .await?;

    Ok(())
}

#[derive(FromRow, Clone)]
pub struct ConnectedDemo {
    pub name: Option<String>,
    pub url: Option<String>,
    pub log_id: i32,
    pub id: Option<i32>,
    pub map: String,
}

pub async fn get_connected_demos(
    pool: &sqlx::PgPool,
) -> Result<Vec<ConnectedDemo>, Box<dyn Error>> {
    let demos = sqlx::query_as::<Postgres, ConnectedDemo>(
        "select name, url, log_id, id, map from connected_demos",
    )
    .fetch_all(pool)
    .await?;
    Ok(demos)
}
