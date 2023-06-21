use std::{collections::HashMap, error::Error, fs::File, io::BufReader};

use dotenv::dotenv;
use insights::collect;
use regex::Regex;

#[derive(Debug)]
struct LogRow {
    log_id: i32,
    unix_timestamp: i32,
    map: String,
}

#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
struct DemoRow {
    id: i32,
    name: String,
    map: String,
    created_at: chrono::NaiveDateTime,
}

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let path = std::env::var("TEAM_LIST_PATH")?;
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let team_map: HashMap<String, collect::Team> = serde_json::from_reader(reader)?;

    let log_pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    let demo_pool = sqlx::PgPool::connect(&std::env::var("DEMO_DATABASE_URL")?).await?;

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

    let all_logs = sqlx::query_as!(
        LogRow,
        "select log_id, unix_timestamp, map from log order by log_id asc"
    )
    .fetch_all(&log_pool)
    .await?;

    println!("Logs: {}", all_logs.len());
    let filter = Regex::new(r"_([a-z]+[^_])_?")?;
    for log in all_logs {
        let lc = log.map.to_lowercase();
        let c = match filter.captures(lc.as_str()) {
            Some(m) => m,
            None => {
                println!("Failed on {}", log.map);
                continue;
            }
        };
        let pure_map = c.get(1).unwrap().as_str();
        let filter_string = format!("%{}%", pure_map);

        // select id, name, map, created_at, server from demos where (demos.created_at between to_timestamp(1684204291 + (4*3600) - 100) and to_timestamp(1684204291 + (4*3600) + 100)) and map like '%sunshine%';

        let row = match sqlx::query_as::<_, DemoRow>(
            "SELECT id, name, map, created_at FROM demos where
            (demos.created_at between to_timestamp($1::integer + (4*3600) - 100) AND to_timestamp($1 + (4*3600) + 100))
            AND map LIKE $2::text",
        )
        .bind(&log.unix_timestamp)
        .bind(&filter_string)
        .fetch_one(&demo_pool)
        .await
        {
            Ok(row) => row,
            Err(err) => {
                println!(
                    "no demo for {}, time: {}, error: {}",
                    log.log_id,
                    log.unix_timestamp,
                    err.to_string()
                );
                continue;
            }
        };

        println!("Matched Demo: {:?} for Log {}", row, log.log_id);
    }

    // for (name, team) in team_map.iter() {
    //     println!("{} map pool", name);
    //     for map in maps {
    //         let search = format!("%{}%", map);
    //         let played = sqlx::query!(
    //             "select count(*) from log where (red_team_id = $1 or blu_team_id = $1) and map like $2",
    //             team.id,
    //             search
    //         )
    //         .fetch_one(&pool)
    //         .await?.count.unwrap();

    //         let wins = sqlx::query!(
    //             "select count(*) from log where (red_team_id = $1 and red_team_score > blu_team_score or blu_team_id = $1 and blu_team_score > red_team_score) and map like $2",
    //             team.id,
    //             search
    //         )
    //         .fetch_one(&pool)
    //         .await?
    //         .count
    //         .unwrap();

    //         println!(
    //             "{:15} Cnt: {}\tWin: {}\tPct: {:.2}",
    //             map,
    //             played,
    //             wins,
    //             (wins as f64) / (played as f64)
    //         );
    //     }
    //     println!("\n");
    // }

    Ok(())
}
