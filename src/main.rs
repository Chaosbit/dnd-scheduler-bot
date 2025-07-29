//! # DND Scheduler Bot Main Entry Point
//! 
//! This is the main entry point for the DND Scheduler Bot application.
//! It initializes logging, loads configuration, sets up the database,
//! starts the reminder service, and runs the Telegram bot.

use anyhow::Result;
use teloxide::prelude::*;
use teloxide::dispatching::dialogue::InMemStorage;
use tracing::info;
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
use crate::services::health::HealthService;
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
    info!("Configuration loaded - Database: {}, HTTP Port: {}", 
        config.database_url, config.http_port);

    // Initialize database
    info!("Initializing database connection...");
    let db_manager = DatabaseManager::new(&config.database_url).await?;
    info!("Running database migrations...");
    db_manager.run_migrations().await?;
    let db_arc = Arc::new(db_manager);
    info!("Database initialized successfully");
    
    // Initialize bot
    info!("Initializing Telegram bot...");
    let bot = Bot::new(&config.telegram_bot_token);
    let handler = BotHandler::new(db_arc.as_ref().clone());
    info!("Telegram bot initialized successfully");
    
    // Initialize and start reminder service
    info!("Initializing reminder service...");
    let mut reminder_service = match ReminderService::new(bot.clone(), db_arc.clone()).await {
        Ok(service) => {
            info!("Reminder service initialized successfully");
            service
        },
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
    
    // Initialize health service
    let health_service = HealthService::new(db_arc.clone());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.http_port))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to port {}: {}", config.http_port, e))?;
    
    info!("Health check server starting on port {}", config.http_port);
    
    // Run both the bot and health server concurrently
    let bot_task = tokio::spawn(async move {
        let storage: std::sync::Arc<InMemStorage<()>> = InMemStorage::new().into();
        Dispatcher::builder(bot, handler.schema())
            .dependencies(dptree::deps![storage])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });
    
    let health_task = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, health_service.router).await {
            tracing::error!("Health server error: {}", e);
        }
    });
    
    // Wait for either task to complete (which would indicate shutdown)
    tokio::select! {
        result1 = bot_task => {
            if let Err(e) = result1 {
                tracing::error!("Bot task error: {}", e);
            }
        }
        result2 = health_task => {
            if let Err(e) = result2 {
                tracing::error!("Health task error: {}", e);
            }
        }
    }
    
    // Stop reminder service on shutdown
    if let Err(e) = reminder_service.stop().await {
        tracing::warn!("Error stopping reminder service: {}", e);
    }
    
    info!("Application stopped");
    Ok(())
}
