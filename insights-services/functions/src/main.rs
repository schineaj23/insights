use std::collections::HashMap;

use insights::analyzer::analyzer::BombAttemptAnalyzer;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tf_demo_parser::{demo::Buffer, DemoParser, Stream};
use tracing::info;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
#[allow(dead_code)]
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request
    let who = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("name"))
        .unwrap_or("world");
    let message = format!("Hello {who}, this is an AWS Lambda HTTP request");

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())
        .map_err(Box::new)?;

    Ok(resp)
}

#[derive(Deserialize)]
struct DemosApiResponse {
    url: String,
}

#[derive(Serialize)]
struct PlayerSummary {
    name: String,
    steamid: i64,
    attempts: i32,
    damage_per_attempt: f32,
}

async fn bomb_handler(event: Request) -> Result<Response<Body>, Error> {
    let demo_id = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("id"))
        .expect("no id found");

    info!("Demo {demo_id}: Querying demos.tf for download URL");

    let download_url = format!("https://api.demos.tf/demos/{demo_id}");
    let demos_resp = reqwest::get(download_url)
        .await?
        .json::<DemosApiResponse>()
        .await?;

    info!("Demo {demo_id}: Found URL, starting download.");

    let resp = reqwest::get(demos_resp.url).await?;

    info!("Demo {demo_id}: Finished download. Starting analysis");

    let demo_bytes: Vec<u8> = resp.bytes().await.unwrap().into();
    let demo_stream = Stream::new(Buffer::from(demo_bytes));
    let (_, (attempts, users)) =
        DemoParser::new_with_analyser(demo_stream, BombAttemptAnalyzer::new()).parse()?;

    info!("Demo {demo_id}: Finished analysis. Packaging");

    let mut bomb_map: HashMap<u16, (i32, i32)> = HashMap::new();

    for attempt in attempts {
        bomb_map
            .entry(attempt.user)
            .and_modify(|u| {
                u.0 += 1;
                u.1 += attempt.damage as i32;
            })
            .or_insert((1, attempt.damage as i32));
    }

    let mut players: Vec<PlayerSummary> = Vec::new();

    for (uid, (cnt, dmg)) in bomb_map {
        let user = users.get(&uid.into()).unwrap();
        players.push(PlayerSummary {
            name: user.name.clone(),
            steamid: insights::steam_id::from_steamid3(&user.steam_id).unwrap(),
            attempts: cnt,
            damage_per_attempt: dmg as f32 / cnt as f32,
        })
    }

    info!("Demo {demo_id}: Finished packaging.");

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(json!({ "players": players }).to_string().into())
        .map_err(Box::new)?;

    Ok(resp)
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

    run(service_fn(bomb_handler)).await
}
