use std::error::Error;

use insights::rgl;
use sqlx::{FromRow, Postgres};

#[allow(dead_code)]
#[derive(FromRow, Debug)]
struct Player {
    id: i32,
    steamid64: i64,
    team_id: Option<i32>,
    name: Option<String>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = dotenv::var("LOG_DATABASE_URL").expect("LOG_DATABASE_URL must be set");
    let log_pool = sqlx::PgPool::connect(&url).await?;

    let players = sqlx::query_as::<Postgres, Player>("select * from player")
        .fetch_all(&log_pool)
        .await?;

    for player in players.iter() {
        if player.name.is_some() {
            println!("Skipping {}", player.name.as_ref().unwrap());
            continue;
        }

        println!("Querying RGL for player {}", player.steamid64);
        let player_search = rgl::search_player(&player.steamid64.to_string()).await?;
        sqlx::query::<Postgres>("update player set name = $1 where steamid64 = $2")
            .bind(&player_search.name)
            .bind(&player.steamid64)
            .execute(&log_pool)
            .await?;
        println!(
            "Updated name {} for id {}",
            player_search.name, player.steamid64
        );
    }
    Ok(())
}
