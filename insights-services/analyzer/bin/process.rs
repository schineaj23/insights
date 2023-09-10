use std::sync::Arc;

use futures::StreamExt;
use insights::db::{self, LegacyConnectedDemo};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use steamid_ng::SteamID;
use tf_demo_parser::{
    demo::{parser::gamestateanalyser::UserId, Buffer},
    DemoParser, Stream,
};

use analyzer::analyzer::{AnalyzerResult, BombAttemptAnalyzer};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = dotenv::var("LOG_DATABASE_URL").expect("LOG_DATABASE_URL not set");
    let pool: Arc<Pool<Postgres>> = PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await?
        .into();

    let connected_demos: Vec<LegacyConnectedDemo> = db::get_connected_demos(&pool)
        .await?
        .into_iter()
        .filter(|x| x.id.is_some())
        .collect();

    let mut handles = Vec::new();

    for demo in connected_demos.into_iter() {
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            let (attempts, users) = match process_demo(&demo).await {
                Ok((x, y)) => (x, y),
                Err(_) => return,
            };
            for (i, attempt) in attempts.iter().enumerate() {
                let id3 = &users.get(&UserId::from(attempt.user)).unwrap().steam_id;
                let id = SteamID::from_steam3(id3).unwrap().account_id() as i64;

                match analyzer::insert_bomb_attempt(&pool, attempt, id, demo.log_id).await {
                    Err(e) => {
                        eprintln!(
                            "[Demo: {}, Attempt: {}] insert_bomb_attempt: {:?}",
                            demo.id.as_ref().unwrap(),
                            i,
                            e
                        );
                    }
                    _ => {}
                }
            }
            println!("[Demo: {}] Finished", demo.id.as_ref().unwrap());
        }));
    }

    let jobs = futures::stream::iter(handles.into_iter())
        .buffer_unordered(4)
        .map(|x| x)
        .collect::<Vec<_>>();
    jobs.await;

    Ok(())
}

async fn process_demo(
    demo: &LegacyConnectedDemo,
) -> Result<AnalyzerResult, Box<dyn std::error::Error>> {
    let id = demo.id.as_ref().unwrap();

    println!("[Demo: {}] Starting download", id);
    let response = reqwest::get(demo.url.as_ref().unwrap()).await?;
    let bytes: Vec<u8> = response.bytes().await.unwrap().into();
    println!("[Demo: {}] Finished download", id);

    println!("[Demo: {}] Starting analyze", id);
    let stream = Stream::new(Buffer::from(bytes));
    let (_, (attempts, users)) =
        DemoParser::new_with_analyser(stream, BombAttemptAnalyzer::new()).parse()?;
    println!("[Demo: {}] Finished analysis", id);

    Ok((attempts, users))
}
