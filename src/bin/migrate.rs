use anyhow::Result;
use dnd_scheduler_bot::database::connection::DatabaseManager;
use dnd_scheduler_bot::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    
    println!("Running database migrations...");
    let db_manager = DatabaseManager::new(&config.database_url).await?;
    db_manager.run_migrations().await?;
    println!("Migrations completed successfully!");
    
    Ok(())
}
