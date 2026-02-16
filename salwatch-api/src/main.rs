mod config;
mod discord;

use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let config = config::load_config();

    // 起動時に Discord にテスト送信
    discord::todiscord::send_to_discord(
        &config.discord_webhook_url,
        "SalWatch API started",
    )
    .await
    .expect("Failed to send Discord message");

    let app = Router::new()
        .route("/", get(|| async { "SalWatch API running" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
