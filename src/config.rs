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
        let token = env::var("TELEGRAM_BOT_TOKEN")
            .map_err(|_| anyhow!("TELEGRAM_BOT_TOKEN must be set"))?;
        
        if token.trim().is_empty() {
            return Err(anyhow!("TELEGRAM_BOT_TOKEN must be set"));
        }
        
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:./data/scheduler.db".to_string());
        let database_url = if database_url.trim().is_empty() {
            "sqlite:./data/scheduler.db".to_string()
        } else {
            database_url
        };
        
        let port_str = env::var("HTTP_PORT")
            .unwrap_or_else(|_| "3000".to_string());
        let http_port = port_str.trim()
            .parse()
            .map_err(|_| anyhow!("Invalid HTTP_PORT"))?;
        
        Ok(Config {
            telegram_bot_token: token,
            database_url,
            http_port,
        })
    }
}
