# HowToUse
### 前提条件
DiscordのWebhook　URLを入手する方法を調べてください。
GitHubリポジトリを用意してください。
外部からアクセス可能なサーバーを用意してください。
### セットアップ
1. Discord Webhook を作成
2. config.toml に webhook_url を設定
3. サーバーを起動
4. GitHub リポジトリに Webhook を追加
   - Payload URL: https://xxx/webhook
   - Content-Type: application/json
   - Event: Pull request
