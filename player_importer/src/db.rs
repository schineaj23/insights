use std::error::Error;

use crate::log::LogSerialized;

pub async fn connect() -> Result<sqlx::PgPool, sqlx::Error> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    sqlx::PgPool::connect(&url).await
}

pub async fn insert_team(
    pool: &sqlx::PgPool,
    team_name: &str,
    team_id: &i32,
) -> Result<(), Box<dyn Error>> {
    let q =
        "insert into team (team_id, team_name) VALUES ($1, $2) on conflict (team_id) do nothing";

    sqlx::query(q)
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
    let q = "INSERT INTO log (log_id, unix_timestamp, map, red_team_id, blu_team_id, red_team_score, blu_team_score) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (log_id) DO NOTHING";

    let red_score = log_data.teams.get("Red").unwrap().score as i32;
    let blu_score = log_data.teams.get("Blue").unwrap().score as i32;

    sqlx::query(q)
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
