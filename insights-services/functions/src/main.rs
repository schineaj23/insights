use std::collections::HashMap;

use insights::analyzer::analyzer::{AnalyzerResult, BombAttemptAnalyzer};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use steamid_ng::SteamID;
use tf_demo_parser::{demo::Buffer, DemoParser, Stream};
use tokio::time::Instant;
use tracing::info;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples

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

fn analyze_demo(demo_bytes: Vec<u8>) -> Result<AnalyzerResult, Error> {
    let demo_stream = Stream::new(Buffer::from(demo_bytes));
    let (_, (attempts, users)) =
        DemoParser::new_with_analyser(demo_stream, BombAttemptAnalyzer::new()).parse()?;
    Ok((attempts, users))
}

fn package_summary(results: AnalyzerResult) -> Vec<PlayerSummary> {
    let mut bomb_map: HashMap<u16, (i32, i32)> = HashMap::new();

    for attempt in results.0 {
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
        let user = results.1.get(&uid.into()).unwrap();
        let id = SteamID::from_steam3(&user.steam_id).unwrap().account_id() as i64;
        players.push(PlayerSummary {
            name: user.name.clone(),
            steamid: id,
            attempts: cnt,
            damage_per_attempt: dmg as f32 / cnt as f32,
        })
    }

    players
}

async fn bomb_handler(event: Request) -> Result<Response<Body>, Error> {
    let params = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("id"));
    let demo_id = match params {
        Some(id) => id,
        None => {
            let resp = Response::builder()
                .header("content-type", "application/json")
                .body(json!({"error": "Invalid arguments"}).to_string().into())?;
            return Ok(resp);
        }
    };

    info!("Demo {demo_id}: Querying demos.tf for download URL");

    let download_url = format!("https://api.demos.tf/demos/{demo_id}");
    let demos_resp = reqwest::get(download_url)
        .await?
        .json::<DemosApiResponse>()
        .await?;

    info!("Demo {demo_id}: Found URL, starting download.");

    if demos_resp.url.is_empty() {
        let resp = Response::builder()
            .header("content-type", "application/json")
            .body(json!({"error": "URL does not exist"}).to_string().into())?;
        return Ok(resp);
    }

    let resp = reqwest::get(demos_resp.url).await?;

    info!("Demo {demo_id}: Finished download. Starting analysis");

    let before = Instant::now();
    let demo_bytes: Vec<u8> = resp.bytes().await.unwrap().into();
    let results = analyze_demo(demo_bytes)?;

    info!(
        "Demo {demo_id}: Finished Analysis in {:.2?}",
        before.elapsed()
    );

    let before = Instant::now();
    let players = package_summary(results);

    info!(
        "Demo {demo_id}: Packaging finished in {:.2?}",
        before.elapsed()
    );

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
