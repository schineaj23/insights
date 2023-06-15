use std::error::Error;

use crate::log::{LogSerialized, PlayerStats};

// TODO: create error enums

pub async fn connect() -> Result<sqlx::PgPool, sqlx::Error> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    sqlx::PgPool::connect(&url).await
}

pub async fn insert_team(
    pool: &sqlx::PgPool,
    team_name: &str,
    team_id: &i32,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "insert into team (team_id, team_name) VALUES ($1, $2) on conflict (team_id) do nothing",
        &team_id,
        &team_name
    )
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

    sqlx::query!(
        "insert into log (log_id, unix_timestamp, map, red_team_id, blu_team_id, red_team_score, blu_team_score) values ($1, $2, $3, $4, $5, $6, $7) on conflict (log_id) do nothing",
        &log_id,
        &log_data.info.date,
        &log_data.info.map,
        &teams.0,
        &teams.1,
        &red_score,
        &blu_score)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn insert_player(
    pool: &sqlx::PgPool,
    steam_id: &i64,
    team_id: &i32,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "insert into player (steamid64, team_id) values ($1, $2) on conflict do nothing",
        steam_id,
        team_id
    )
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
    sqlx::query!(
        "insert into player_stats (log_id, player_steamid64, kills, deaths, dmg, dmg_real, dt, dt_real, hr, ubers, drops, headshots, headshots_hit) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        log_id,
        player_id,
        stats.kills as i32,
        stats.deaths as i32,
        stats.dmg,
        stats.dmg_real,
        stats.dt,
        stats.dt_real,
        stats.hr,
        stats.ubers as i32,
        stats.drops as i32,
        stats.headshots as i32,
        stats.headshots_hit as i32)
        .execute(pool)
        .await?;

    Ok(())
}
