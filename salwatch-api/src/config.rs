//config.tomlから設定を読み込むためのモジュール
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub discord_webhook_url: String,
}   //TODO::configに追加するたびに追加を忘れずに

pub fn load_config() -> Config {
    let context = fs::read_to_string("config.toml")
        .expect("Failed to read config.toml");
    
    toml::from_str(&context)
        .expect("Failed to parse config.toml")
}