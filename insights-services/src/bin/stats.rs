use std::{collections::HashMap, error::Error, fs::File, io::BufReader};

use dotenv::dotenv;
use insights::{collect, db};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let path = std::env::var("TEAM_LIST_PATH")?;
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let team_map: HashMap<String, collect::Team> = serde_json::from_reader(reader)?;
    println!("map read");

    let pool: sqlx::Pool<sqlx::Postgres> = db::connect().await?;

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

    for (name, team) in team_map.iter() {
        println!("{} map pool", name);
        for map in maps {
            let search = format!("%{}%", map);
            let played = sqlx::query!(
                "select count(*) from log where (red_team_id = $1 or blu_team_id = $1) and map like $2",
                team.id,
                search
            )
            .fetch_one(&pool)
            .await?.count.unwrap();

            let wins = sqlx::query!(
                "select count(*) from log where (red_team_id = $1 and red_team_score > blu_team_score or blu_team_id = $1 and blu_team_score > red_team_score) and map like $2",
                team.id,
                search
            )
            .fetch_one(&pool)
            .await?
            .count
            .unwrap();

            println!(
                "{:15} Cnt: {}\tWin: {}\tPct: {:.2}",
                map,
                played,
                wins,
                (wins as f64) / (played as f64)
            );
        }
        println!("\n");
    }

    Ok(())
}
