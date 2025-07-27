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
use crate::services::reminder::ReminderService;
use std::sync::Arc;

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
    let db_arc = Arc::new(db_manager);
    
    // Initialize bot
    let bot = Bot::new(&config.telegram_bot_token);
    let handler = BotHandler::new(db_arc.as_ref().clone());
    
    // Initialize and start reminder service
    let mut reminder_service = match ReminderService::new(bot.clone(), db_arc.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to create reminder service: {}", e);
            return Err(anyhow::anyhow!("Failed to create reminder service: {}", e));
        }
    };
    
    if let Err(e) = reminder_service.start().await {
        tracing::error!("Failed to start reminder service: {}", e);
    } else {
        info!("Reminder service started successfully");
    }
    
    // Start bot with handler
    Dispatcher::builder(bot, handler.schema())
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    
    // Stop reminder service on shutdown
    if let Err(e) = reminder_service.stop().await {
        tracing::warn!("Error stopping reminder service: {}", e);
    }
    
    info!("Bot stopped");
    Ok(())
}
