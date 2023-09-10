use aws_lambda_events::event::sqs::SqsEvent;
use insights::{demos::MatchedDemoMessage, log};
use itertools::Itertools;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use steamid_ng::SteamID;
use tracing::{info, warn};

async fn load_credentials() -> aws_sdk_sqs::Client {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sqs::Client::new(&config);
    client
}

async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Box<dyn std::error::Error>> {
    let client = load_credentials().await;

    let records = event.payload.records;
    for (i, record) in records.iter().enumerate() {
        info!("Processing record {}: {:?}", i, record);
        let body = record.body.as_ref().unwrap();

        let log_id = body.parse::<i32>()?;

        let log = log::fetch_log(&log_id).await?;

        let players_arg = log
            .players
            .keys()
            .map(|x| {
                let id = SteamID::from_steam3(&x).unwrap();
                let id64 = u64::from(id).to_string();
                id64
            })
            .join("&players[]=");

        let url = format!(
            "https://api.demos.tf/demos?map={}&type=6v6&players[]={}",
            log.info.map, players_arg
        );

        info!("Record {}: Log: https://logs.tf/{}", i, log_id);
        info!("Record {}: Demo: {}", i, url);

        let found = insights::demos::search_demo(&log.info.map, &players_arg).await?;
        if found.len() == 0 {
            warn!("Record {}: No demos found for ID: {}", i, log_id);
            return Ok(());
        }

        let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL must be set!");

        for demo_data in found.into_iter() {
            let message_body = MatchedDemoMessage {
                log_id: log_id,
                demo: demo_data,
            };

            let body = serde_json::to_string(&message_body)?;

            let message = client
                .send_message()
                .queue_url(&queue_url)
                .message_body(&body)
                .send()
                .await?;
            info!("Record {}: Added to queue. Id: {:?}", i, message.message_id);
        }
    }

    Ok(())
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
