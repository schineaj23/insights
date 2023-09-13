use aws_lambda_events::event::sqs::SqsEvent;
use insights::demos::{DemoSerialized, MatchedDemoMessage};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use snafu::Snafu;
use steamid_ng::SteamID;
use tf_demo_parser::demo::parser::gamestateanalyser::UserId;
use tracing::{debug, info, warn};

#[derive(Debug, Snafu)]
enum AttemptInsertError {
    #[snafu(display("Unable to download demo {}, has been deleted", id))]
    DemoDeleted { id: i32 },

    #[snafu(display("Could not analyze demo, parsing failed"))]
    ParseFailed,
}

async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    let url = std::env::var("LOG_DATABASE_URL")?;
    let pool = sqlx::PgPool::connect(&url).await?;

    let records = event.payload.records;
    for record in records.iter() {
        let record_id = record.message_id.as_ref().unwrap();

        let message_body = record.body.as_ref().unwrap();
        let parsed = unescaper::unescape(&message_body)?;
        let body = serde_json::from_str::<MatchedDemoMessage>(&parsed)?;

        let demo = &body.demo;

        info!(
            "Record {}: Demo {}: Starting demo download",
            record_id, demo.id
        );

        let demo_bytes = download_demo(&demo).await?;

        info!(
            "Record {}: Demo {}: Finished demo download",
            record_id, demo.id
        );

        info!("Record {}: Demo {}: Starting analysis", record_id, demo.id);

        let (attempts, user_info) = match analyzer::analyze(demo_bytes) {
            Ok(it) => it,
            Err(_) => return Err(Box::new(AttemptInsertError::ParseFailed)),
        };

        info!("Record {}: Demo {}: Finished analysis", record_id, demo.id);

        for (i, attempt) in attempts.iter().enumerate() {
            let id3 = &user_info.get(&UserId::from(attempt.user)).unwrap().steam_id;
            let id = u64::from(SteamID::from_steam3(id3).unwrap()) as i64;

            match analyzer::insert_bomb_attempt(&pool, attempt, id, body.log_id).await {
                Err(e) => {
                    warn!(
                        "[Demo: {}, Attempt: {}] insert_bomb_attempt: {:?}",
                        demo.id, i, e
                    );
                }
                Ok(_) => {
                    debug!(
                        "Record {}:[Demo {}, Attempt {}] insert_bomb_attempt: {}",
                        record_id, body.log_id, i, id
                    );
                }
            }
        }

        info!("Record {}: Demo {}: Finished.", record_id, body.demo.id);
    }

    Ok(())
}

async fn download_demo(demo: &DemoSerialized) -> Result<Vec<u8>, Error> {
    if demo.backend == "deleted" {
        return Err(Box::new(AttemptInsertError::DemoDeleted { id: demo.id }));
    }

    let res = reqwest::get(&demo.url).await?;
    let bytes = res.bytes().await?.to_vec();

    Ok(bytes)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
