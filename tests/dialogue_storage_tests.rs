use dnd_scheduler_bot::bot::handlers::BotHandler;
use dnd_scheduler_bot::database::connection::DatabaseManager;
use teloxide::dispatching::dialogue::InMemStorage;
use tempfile::TempDir;

#[tokio::test]
async fn test_dialogue_storage_setup() {
    // Create test database
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite://{}", db_path.display());
    
    let db = DatabaseManager::new(&db_url)
        .await
        .expect("Failed to create test database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db.pool)
        .await
        .expect("Failed to run migrations");
    
    // Create bot handler
    let handler = BotHandler::new(db);
    
    // Create dialogue storage
    let _storage: std::sync::Arc<InMemStorage<()>> = InMemStorage::new().into();
    
    // This should not panic - create the schema
    let _schema = handler.schema();
    
    // Test passes if we reach here without panicking
    assert!(true);
}