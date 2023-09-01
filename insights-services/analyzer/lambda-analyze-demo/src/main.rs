mod cache;

use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use serde::Deserialize;
use serde_json::json;
use tokio::time::Instant;
use tracing::{info, warn};

use crate::cache::Item;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples

#[derive(Deserialize)]
struct DemosApiResponse {
    url: String,
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

    // Look up in cache and return value if exists
    let client = cache::get_client().await?;
    if let Ok(item) = cache::check_in_cache(&client, demo_id).await {
        info!("Demo {demo_id}: Cache hit!");
        let resp = Response::builder()
            .header("content-type", "application/json")
            .body(item.body.into())?;
        return Ok(resp);
    }

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
    let results = analyzer::analyze(demo_bytes).or(Err("Failed parse demo"))?;

    info!(
        "Demo {demo_id}: Finished Analysis in {:.2?}",
        before.elapsed()
    );

    let before = Instant::now();
    let players = analyzer::package_summary(results);

    info!(
        "Demo {demo_id}: Packaging finished in {:.2?}",
        before.elapsed()
    );

    let body = json!({ "players": players }).to_string();

    // Add this response to cache
    cache::write_to_cache(
        &client,
        Item {
            id: demo_id.to_string(),
            body: body.clone(),
        },
    )
    .await
    .map_err(|err| {
        warn!("Demo {demo_id}: Could not write to cache! Error: {err:?}");
    })
    .ok();

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(body.into())
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
