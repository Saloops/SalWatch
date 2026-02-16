//discordに送るPayloadを作るモジュール
use serde::Serialize;
#[derive(Serialize, Debug)]
pub struct DiscordPayload {
    content: String,
}
//実際のSalWatchのイベントを元にしたメッセージを作る関数を追加する予定
pub fn build_message(
    repo: &str,
    title: &str,
    author: &str,
    pr_url: &str,
) -> DiscordPayload {
    DiscordPayload {
        content: format!(
            "🐾 PRが来てます\n\
             リポジトリ: {}\n\
             タイトル: {}\n\
             作成者: {}\n\
             URL: {}",
            repo, title, author, pr_url
        ),
    }
}