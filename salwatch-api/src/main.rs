mod config;
mod discord;

use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    //ペイロードのダミーデータ※あとで消す
    //===============================
    struct Payload {
        repo: String,
        title: String,
        author: String,
        pr_url: String,
    }
    let dummy_data = Payload {//ダミーデータ
        repo: "my-repo".to_string(),
        title: "My PR Title".to_string(),
        author: "My Author".to_string(),
        pr_url: "https://x.com".to_string(),
    };
    //===============================
    let config = config::load_config();
    //TODO::テスト用になってるから実装後に実物に置き換え
    let payload = discord::message_builder::build_message(
        &dummy_data.repo,
        &dummy_data.title,
        &dummy_data.author,
        &dummy_data.pr_url
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
