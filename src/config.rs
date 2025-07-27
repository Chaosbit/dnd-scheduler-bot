use anyhow::{anyhow, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub telegram_bot_token: String,
    pub database_url: String,
    pub http_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN")
                .map_err(|_| anyhow!("TELEGRAM_BOT_TOKEN must be set"))?,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./data/scheduler.db".to_string()),
            http_port: env::var("HTTP_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| anyhow!("Invalid HTTP_PORT"))?,
        })
    }
}
