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
    let pr = match payload.pull_request {
        Some(pr) => pr,
        None => return Err("No pull request data found".to_string()),
    };
    if payload.action != "opened" {
        return Err("Pull request action is not 'opened'".to_string());
    }
    //================================
    //Discordに送るためのペイロードの作成
    //================================
    let event = ChangeEvent {
        repo: payload.repository.full_name,
        title: pr.title,
        author: pr.user.login,
        pr_url: pr.html_url,
    };
    //================================
    Ok(event)
}