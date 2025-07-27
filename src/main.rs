use anyhow::Result;
use teloxide::prelude::*;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod bot;
mod config;
mod database;
mod services;
mod utils;

use crate::bot::handlers::BotHandler;
use crate::config::Config;
use crate::database::connection::DatabaseManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dnd_scheduler_bot=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    
    info!("Starting D&D Scheduler Bot v{}", env!("CARGO_PKG_VERSION"));

    // Initialize database
    let db_manager = DatabaseManager::new(&config.database_url).await?;
    db_manager.run_migrations().await?;
    
    // Initialize bot
    let bot = Bot::new(&config.telegram_bot_token);
    let handler = BotHandler::new(db_manager);
    
    // Start bot with handler
    Dispatcher::builder(bot, handler.schema())
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    
    info!("Bot stopped");
    Ok(())
}
