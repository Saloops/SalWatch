use axum::{routing::post, Router};
use axum::http::HeaderMap;

#[derive(Debug)]
struct ChangeLog {
    repo: String,
    message: String,
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let mut logs: Vec<ChangeLog> = Vec::new();
    logs.push(ChangeLog {
        repo: "salwatch".to_string(),
        message: "Initial commit".to_string(),
    });
    println!("Change logs: {:?}", logs);

    let app = Router::new().route(
        "/webhook",
         post(|headers: HeaderMap, body: String| async move {
            println!("=== Headers ===");
            println!("{:#?}", headers);
            println!("=== Body ===");
            println!("{}", body);
            "OK"
         }),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
