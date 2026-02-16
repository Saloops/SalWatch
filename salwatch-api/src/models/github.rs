//GitHubのデータに合わせてるだけだから変更しないように
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GitHubWebhook {
    pub repository: Repository,
    pub pull_request: PullRequest,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub title: String,
    pub user: User,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
}