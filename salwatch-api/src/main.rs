mod config;
mod discord;
mod from_github;
mod models;

use std::fs;
use axum::{routing::get, Router};
use from_github::post_from_github::process_github_event;
use discord::message_builder::build_message;

#[tokio::main]
async fn main() {
    //ダミーで処理してるのはここ
    //================================
    let json = fs::read_to_string("sample/github_pr.json").unwrap();
    //================================
    let event = process_github_event(&json).unwrap();
    let config = config::load_config();
    //TODO::テスト用になってるから実装後に実物に置き換え
    let payload = discord::message_builder::build_message(
        &event.repo,
        &event.title,
        &event.author,
        &event.pr_url
    );


    // 起動時に Discord にテスト送信
    discord::todiscord::send_to_discord(
        &config.discord_webhook_url,
        &payload,
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
