mod config;
mod discord;
mod from_github;
mod models;

use axum::{routing::post, Router};
use from_github::post_from_github::process_github_event;
use discord::message_builder::build_message;
use axum::body::Bytes;
use axum::response::IntoResponse;
use axum::http::StatusCode;

#[tokio::main]
async fn main() {
    //サーバー化
    let app = Router::new()
        .route("/webhook", post(github_webhook));

    //TODO: 本番環境では適切なポートとアドレスを使用してください
    let app_config =  config::load_app_config();
    let addr = format!(
        "{}:{}", 
        app_config.server.host, 
        app_config.server.port
    );
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn github_webhook(body: Bytes) -> impl IntoResponse {
    let json = match String::from_utf8(body.to_vec()) {
        Ok(v) => v,
        Err(e) => return StatusCode::BAD_REQUEST,
    };
    let event = match process_github_event(&json) {
        Ok(e) => e,
        Err(_) => return StatusCode::OK,
    };
    let config = config::load_config();
    let payload = build_message(
        &event.repo,
        &event.title,
        &event.author,
        &event.pr_url
    );

    if let Err(e) = discord::todiscord::send_to_discord(
        &config.discord_webhook_url,
        &payload,
    )
    .await
    {
        eprintln!("Failed to send message to Discord: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}