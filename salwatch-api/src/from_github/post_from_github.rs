//githubからpostされた内容を処理するモジュール
use crate::models::{
    github::GitHubWebhook,
    salwatch::ChangeEvent,
};

pub fn process_github_event(json: &str) -> Result<ChangeEvent, String> {
    //jsonの解析
    //================================
    let payload: GitHubWebhook =
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    //================================
    //必要な情報の抽出
    //================================

    //================================
    //Discordに送るためのペイロードの作成
    //================================
    let event = ChangeEvent {
        repo: payload.repository.full_name,
        title: payload.pull_request.title,
        author: payload.pull_request.user.login,
        pr_url: payload.pull_request.html_url,
    };
    //================================
    Ok(event)
}