use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use aws_sdk_dynamodb as dynamodb;
use dynamodb::{error::BoxError, types::AttributeValue, Client};
use tracing::info;

const TABLE_NAME: &'static str = "analyze-demo-cache";

#[derive(Debug)]
struct ItemNotFound;

impl Display for ItemNotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item not found!")
    }
}

impl Error for ItemNotFound {}

pub struct Item {
    pub id: String,
    pub body: String,
}

pub async fn get_client() -> Result<Client, BoxError> {
    let config = aws_config::load_from_env().await;
    let client = dynamodb::Client::new(&config);

    Ok(client)
}

pub async fn check_in_cache(client: &Client, id: &str) -> Result<Item, BoxError> {
    let id_av = AttributeValue::S(id.to_string());

    let request = client.get_item().table_name(TABLE_NAME).key("id", id_av);

    let resp = request.send().await?;
    let items = resp.item().ok_or(ItemNotFound)?;
    let id = items.get("id").unwrap().as_s().unwrap();
    let body = items.get("body").unwrap().as_s().unwrap();

    Ok(Item {
        id: id.clone(),
        body: body.clone(),
    })
}

pub async fn write_to_cache(client: &Client, item: Item) -> Result<(), BoxError> {
    let id_av = AttributeValue::S(item.id);
    let body_av = AttributeValue::S(item.body);

    let request = client
        .put_item()
        .table_name(TABLE_NAME)
        .item("id", id_av)
        .item("body", body_av);

    info!("Executing request [{request:?}], adding item to table.");

    let _resp = request.send().await?;

    Ok(())
}
