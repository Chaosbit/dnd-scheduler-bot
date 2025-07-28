use anyhow::{Result, anyhow};
use dnd_scheduler_bot::database::connection::DatabaseManager;
use dnd_scheduler_bot::config::Config;
use std::env;
use std::io;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize basic logging for the migration
    env_logger::init();
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("migrate");
    
    match command {
        "migrate" | "up" => run_migrations().await,
        "check" => check_database().await,
        "reset" => reset_database().await,
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {command}");
            print_help();
            std::process::exit(1);
        }
    }
}

async fn run_migrations() -> Result<()> {
    println!("🔧 D&D Scheduler Bot - Database Migration Tool");
    println!("================================================");
    
    // Load environment configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    
    println!("📊 Database URL: {}", mask_url(&config.database_url));
    
    // Ensure data directory exists for SQLite
    if config.database_url.starts_with("sqlite:") {
        let db_path = config.database_url.strip_prefix("sqlite:").unwrap_or(&config.database_url);
        if let Some(parent) = Path::new(db_path).parent() {
            if !parent.exists() {
                println!("📁 Creating directory: {}", parent.display());
                std::fs::create_dir_all(parent)?;
            }
        }
    }
    
    println!("🚀 Running database migrations...");
    
    let db_manager = DatabaseManager::new(&config.database_url).await
        .map_err(|e| anyhow!("Failed to connect to database: {}", e))?;
    
    match db_manager.run_migrations().await {
        Ok(_) => {
            println!("✅ Migrations completed successfully!");
            println!("\n🎯 Your D&D Scheduler Bot database is ready!");
        }
        Err(e) => {
            eprintln!("❌ Migration failed: {e}");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn check_database() -> Result<()> {
    println!("🔍 Checking database connection and schema...");
    
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    
    println!("📊 Database URL: {}", mask_url(&config.database_url));
    
    let db_manager = DatabaseManager::new(&config.database_url).await
        .map_err(|e| anyhow!("Failed to connect to database: {}", e))?;
    
    // Try to query the database to check if tables exist
    match check_tables(&db_manager).await {
        Ok(tables) => {
            println!("✅ Database connection successful!");
            println!("📋 Found tables:");
            for table in tables {
                println!("  • {table}");
            }
        }
        Err(e) => {
            println!("⚠️  Database check failed: {e}");
            println!("💡 Try running 'migrate up' to create the schema");
        }
    }
    
    Ok(())
}

async fn reset_database() -> Result<()> {
    println!("⚠️  WARNING: This will delete ALL data in the database!");
    println!("🤔 Are you sure you want to continue? (yes/no)");
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "yes" {
        println!("❌ Reset cancelled.");
        return Ok(());
    }
    
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    
    // For SQLite, we can just delete the file
    if config.database_url.starts_with("sqlite:") {
        let db_path = config.database_url.strip_prefix("sqlite:").unwrap_or(&config.database_url);
        if Path::new(db_path).exists() {
            std::fs::remove_file(db_path)?;
            println!("🗑️  Deleted database file: {db_path}");
        }
    } else {
        return Err(anyhow!("Reset is only supported for SQLite databases"));
    }
    
    // Run migrations to recreate the schema
    println!("🔄 Recreating database schema...");
    run_migrations().await?;
    
    println!("✅ Database reset completed!");
    
    Ok(())
}

async fn check_tables(db_manager: &DatabaseManager) -> Result<Vec<String>> {
    let rows = sqlx::query!("SELECT name FROM sqlite_master WHERE type='table'")
        .fetch_all(&db_manager.pool)
        .await?;
    
    Ok(rows.into_iter().filter_map(|row| row.name).collect())
}

fn mask_url(url: &str) -> String {
    // Simple URL masking for security (don't show full paths in production)
    if url.starts_with("sqlite:") {
        let path = url.strip_prefix("sqlite:").unwrap_or(url);
        if let Some(filename) = Path::new(path).file_name() {
            format!("sqlite:.../{}", filename.to_string_lossy())
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

fn print_help() {
    println!("🎲 D&D Scheduler Bot - Database Migration Tool");
    println!();
    println!("USAGE:");
    println!("    migrate [COMMAND]");
    println!();
    println!("COMMANDS:");
    println!("    migrate, up    Run database migrations (default)");
    println!("    check          Check database connection and schema");
    println!("    reset          Reset database (SQLite only) - DESTRUCTIVE!");
    println!("    help           Show this help message");
    println!();
    println!("ENVIRONMENT:");
    println!("    DATABASE_URL   Database connection string (default: sqlite:./data/scheduler.db)");
    println!();
    println!("EXAMPLES:");
    println!("    migrate                    # Run migrations");
    println!("    migrate check              # Check database status");
    println!("    migrate reset              # Reset database (careful!)");
    println!();
}
