use std::{error::Error, sync::Arc};

use chrono::{DateTime, NaiveDateTime, Utc};
use dotenv::dotenv;
use regex::Regex;
use sqlx::Postgres;

#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
struct LogRow {
    log_id: i32,
    unix_timestamp: i32,
    map: String,
}

#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
struct DemoRow {
    name: String,
    url: String,
    map: String,
    created_at: chrono::NaiveDateTime,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let demo_pool: sqlx::Pool<Postgres> =
        sqlx::PgPool::connect(&std::env::var("DEMO_DATABASE_URL")?).await?;
    let log_pool = sqlx::PgPool::connect(&std::env::var("LOG_DATABASE_URL")?).await?;

    println!("connected to db");

    let maps = [
        "process",
        "snakewater",
        "bagel",
        "sultry",
        "gullywash",
        "product",
        "metalworks",
        "sunshine",
        "granary",
    ];

    let all_logs: Arc<[LogRow]> = sqlx::query_as::<Postgres, LogRow>(
        "select log_id, unix_timestamp, map from log order by log_id asc",
    )
    .fetch_all(&log_pool)
    .await?
    .into();

    println!("Collected Logs");

    let mut demos: Vec<DemoRow> = Vec::new();
    let filter = Regex::new(r"_([a-z]+[^_])_?")?;

    for log in all_logs.iter() {
        let captures = match filter.captures(&log.map) {
            Some(m) => m,
            None => {
                println!("Failed on {}", log.map);
                continue;
            }
        };
        let map_unfiltered = captures.get(1).unwrap().as_str();

        if !maps.contains(&map_unfiltered) {
            println!(
                "Log({}): Map is not in map pool! Found {}",
                log.log_id, map_unfiltered
            );
            continue;
        }

        let map = format!("%{}%", map_unfiltered);

        let lower_bound = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt((log.unix_timestamp as i64) - 100, 0).unwrap(),
            Utc,
        );

        let upper_bound = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt((log.unix_timestamp as i64) + 100, 0).unwrap(),
            Utc,
        );

        let query = sqlx::query_as::<Postgres, DemoRow>("select name, url, map, created_at from demos where demos.created_at between $1 and $2 and demos.map like $3")
        .bind(&lower_bound)
        .bind(&upper_bound)
        .bind(&map)
        .fetch_one(&demo_pool).await;
        match query {
            Ok(demo) => {
                println!("Log({}): {:?}", log.log_id, demo);
                demos.push(demo);
            }
            Err(error) => {
                println!("Log({}): {:?}", log.log_id, error);
            }
        };
    }

    println!("Demos found: {}", demos.len());

    // TODO: create column for demo id and join logs on demo_id

    Ok(())
}
