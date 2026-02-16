use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn, error};

use crate::{
    config::Config,
    models::{webhook_event::*, ChangeLog},
    webhook_verifier::WebhookVerifier,
};

/// アプリケーションの共有状態
/// 
/// Axumでは、State extractorを使って複数のハンドラー間で
/// データを共有できます。Arc（Atomic Reference Count）で
/// スレッドセーフに共有します。
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub webhook_verifier: Arc<WebhookVerifier>,
    // 後で追加:
    // pub db: Arc<Database>,
    // pub discord_client: Arc<DiscordClient>,
}

/// GitHub Webhookを受信するハンドラー
/// 
/// # フロー
/// 1. 署名を検証（セキュリティ）
/// 2. イベントタイプを判定
/// 3. ペイロードをパース
/// 4. イベントに応じた処理を実行
/// 5. Discordに通知（後で実装）
/// 
/// # エンドポイント
/// POST /webhook
/// 
/// # 使い方
/// GitHub リポジトリの Settings > Webhooks で以下を設定:
/// - Payload URL: https://your-domain.com/webhook
/// - Content type: application/json
/// - Secret: あなたのシークレット
/// - Events: Push, Pull requests, Issues など
pub async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    info!("📥 Received webhook request");
    
    // ==========================================
    // ステップ1: 署名検証
    // ==========================================
    // 重要: これがないとセキュリティホール！
    if let Err(e) = state.webhook_verifier.verify(&headers, &body) {
        warn!("⚠️ Webhook signature verification failed: {}", e);
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid signature",
                "message": e
            }))
        ).into_response();
    }
    
    info!("✅ Signature verified");
    
    // ==========================================
    // ステップ2: イベントタイプを取得
    // ==========================================
    // GitHub は X-GitHub-Event ヘッダーでイベントタイプを送信
    let event_type = match headers.get("X-GitHub-Event") {
        Some(value) => match value.to_str() {
            Ok(v) => v,
            Err(_) => {
                error!("❌ Invalid event type header");
                return (StatusCode::BAD_REQUEST, "Invalid event type").into_response();
            }
        },
        None => {
            error!("❌ Missing X-GitHub-Event header");
            return (StatusCode::BAD_REQUEST, "Missing event type").into_response();
        }
    };
    
    info!("📋 Event type: {}", event_type);
    
    // ==========================================
    // ステップ3: ペイロードをJSONとしてパース
    // ==========================================
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(json) => json,
        Err(e) => {
            error!("❌ Failed to parse JSON: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid JSON",
                    "message": e.to_string()
                }))
            ).into_response();
        }
    };
    
    // ==========================================
    // ステップ4: イベントタイプごとに処理を分岐
    // ==========================================
    let result = match event_type {
        "push" => handle_push_event(&payload).await,
        "pull_request" => handle_pull_request_event(&payload).await,
        "issues" => handle_issue_event(&payload).await,
        "ping" => {
            // GitHub が Webhook 設定時に送信するテストイベント
            info!("🏓 Received ping event");
            Ok("pong".to_string())
        }
        _ => {
            warn!("⚠️ Unhandled event type: {}", event_type);
            Ok(format!("Event type '{}' is not handled yet", event_type))
        }
    };
    
    // ==========================================
    // ステップ5: レスポンスを返す
    // ==========================================
    match result {
        Ok(message) => {
            info!("✅ Event processed successfully: {}", message);
            (StatusCode::OK, Json(serde_json::json!({
                "status": "success",
                "message": message
            }))).into_response()
        }
        Err(e) => {
            error!("❌ Error processing event: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "status": "error",
                "message": e
            }))).into_response()
        }
    }
}

/// Pushイベントを処理
/// 
/// # 処理内容
/// 1. ペイロードをPushEvent構造体にデシリアライズ
/// 2. コミット情報を抽出
/// 3. ChangeLogに記録
/// 4. Discordに通知（後で実装）
async fn handle_push_event(payload: &Value) -> Result<String, String> {
    info!("🔄 Processing push event");
    
    // JSONをPushEvent構造体にパース
    let push_event: PushEvent = serde_json::from_value(payload.clone())
        .map_err(|e| format!("Failed to parse push event: {}", e))?;
    
    // ブランチ名を抽出（"refs/heads/main" → "main"）
    let branch = push_event.ref_name
        .strip_prefix("refs/heads/")
        .unwrap_or(&push_event.ref_name);
    
    info!("📌 Push to branch: {}", branch);
    info!("👤 Pusher: {}", push_event.pusher.login);
    info!("📝 Commits: {}", push_event.commits.len());
    
    // コミット情報を表示
    for commit in &push_event.commits {
        info!("  - {} ({})", 
            commit.message.lines().next().unwrap_or(""),
            &commit.id[..7] // 最初の7文字だけ表示
        );
    }
    
    // ChangeLogを作成
    let commit_shas: Vec<String> = push_event.commits
        .iter()
        .map(|c| c.id.clone())
        .collect();
    
    let changelog = ChangeLog::from_push(
        &push_event.repository.full_name,
        &push_event.pusher.login,
        branch,
        &commit_shas,
    );
    
    info!("📊 ChangeLog created: {:?}", changelog);
    
    // TODO: データベースに保存
    // TODO: Discordに通知
    
    Ok(format!(
        "Processed push: {} commits to {} by {}",
        push_event.commits.len(),
        branch,
        push_event.pusher.login
    ))
}

/// Pull Requestイベントを処理
async fn handle_pull_request_event(payload: &Value) -> Result<String, String> {
    info!("🔀 Processing pull request event");
    
    let pr_event: PullRequestEvent = serde_json::from_value(payload.clone())
        .map_err(|e| format!("Failed to parse PR event: {}", e))?;
    
    info!("🎯 Action: {}", pr_event.action);
    info!("📌 PR #{}: {}", pr_event.number, pr_event.pull_request.title);
    info!("👤 Author: {}", pr_event.pull_request.user.login);
    info!("🔗 {}", pr_event.pull_request.html_url);
    
    // アクションに応じた処理
    match pr_event.action.as_str() {
        "opened" => {
            info!("✨ New PR opened");
            // TODO: Discordに「新しいPRが作成されました」と通知
        }
        "closed" => {
            if pr_event.pull_request.merged {
                info!("✅ PR merged");
                // TODO: Discordに「PRがマージされました」と通知
            } else {
                info!("❌ PR closed without merge");
            }
        }
        "synchronize" => {
            info!("🔄 PR updated with new commits");
            // TODO: Discordに「PRが更新されました」と通知
        }
        _ => {
            info!("ℹ️ Other PR action: {}", pr_event.action);
        }
    }
    
    // ChangeLogを作成
    let changelog = ChangeLog::from_pr(
        &pr_event.repository.full_name,
        &pr_event.pull_request.user.login,
        &pr_event.action,
        pr_event.number,
        &pr_event.pull_request.title,
    );
    
    info!("📊 ChangeLog created: {:?}", changelog);
    
    // TODO: データベースに保存
    // TODO: Discordに通知
    
    Ok(format!(
        "Processed PR #{}: {} ({})",
        pr_event.number,
        pr_event.pull_request.title,
        pr_event.action
    ))
}

/// Issueイベントを処理
async fn handle_issue_event(payload: &Value) -> Result<String, String> {
    info!("🐛 Processing issue event");
    
    let issue_event: IssueEvent = serde_json::from_value(payload.clone())
        .map_err(|e| format!("Failed to parse issue event: {}", e))?;
    
    info!("🎯 Action: {}", issue_event.action);
    info!("📌 Issue #{}: {}", issue_event.issue.number, issue_event.issue.title);
    info!("👤 Author: {}", issue_event.issue.user.login);
    info!("🔗 {}", issue_event.issue.html_url);
    
    match issue_event.action.as_str() {
        "opened" => {
            info!("✨ New issue opened");
            // TODO: Discordに通知
        }
        "closed" => {
            info!("✅ Issue closed");
            // TODO: Discordに通知
        }
        "reopened" => {
            info!("🔄 Issue reopened");
            // TODO: Discordに通知
        }
        _ => {
            info!("ℹ️ Other issue action: {}", issue_event.action);
        }
    }
    
    // TODO: データベースに保存
    // TODO: Discordに通知
    
    Ok(format!(
        "Processed issue #{}: {} ({})",
        issue_event.issue.number,
        issue_event.issue.title,
        issue_event.action
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_push_event_parsing() {
        let json = r#"{
            "ref": "refs/heads/main",
            "before": "abc123",
            "after": "def456",
            "repository": {
                "id": 1,
                "name": "test-repo",
                "full_name": "user/test-repo",
                "html_url": "https://github.com/user/test-repo",
                "description": null,
                "default_branch": "main"
            },
            "pusher": {
                "id": 1,
                "login": "testuser",
                "avatar_url": "https://example.com/avatar.png",
                "html_url": "https://github.com/testuser"
            },
            "commits": [],
            "compare": "https://github.com/user/test-repo/compare/abc123...def456"
        }"#;
        
        let value: Value = serde_json::from_str(json).unwrap();
        let result = handle_push_event(&value).await;
        assert!(result.is_ok());
    }
}