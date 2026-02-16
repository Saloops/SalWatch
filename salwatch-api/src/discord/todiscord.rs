use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::discord::message_builder;

pub async fn send_to_discord<T: Serialize>(
    webhook_url: &str,
    payload: &T,
) -> Result<(), reqwest::Error> {
    let client = Client::new();

    client.post(webhook_url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?; // HTTPエラーを検出

    Ok(())
}