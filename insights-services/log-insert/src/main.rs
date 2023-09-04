mod cache;

use aws_lambda_events::event::sqs::SqsEvent;
use cache::LocalCache;
use dotenv::dotenv;
use importer::Importer;
use insights::log;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("LOG_DATABASE_URL").expect("LOG_DATABASE_URL not set");
    let pool = sqlx::PgPool::connect(&url).await?;

    let cache = LocalCache::new();
    let mut importer = Importer::new(&pool, cache, true);

    let records = event.payload.records;
    for record in records.iter() {
        println!("Processing record: {:?}", record);
        let body = record.body.as_ref().unwrap();
        let log_id = body.parse::<i32>()?;
        let log = log::fetch_log(&log_id).await?;
        importer.import_log(log_id, &log).await?;
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

    dotenv().ok();
    run(service_fn(function_handler)).await
}
