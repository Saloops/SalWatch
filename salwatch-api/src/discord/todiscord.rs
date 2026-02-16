use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct DiscordMessage {
    content: String,
}

pub async fn send_to_discord(webhook_url: &str, message: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let payload = DiscordMessage {
        content: message.to_string(),
    };

    client.post(webhook_url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?; // HTTPエラーを検出

    Ok(())
}