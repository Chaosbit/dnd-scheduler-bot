use anyhow::Result;
use chrono::Utc;
use dnd_scheduler_bot::database::{connection::DatabaseManager, models::*};
use tempfile::{tempdir, TempDir};

async fn setup_test_db() -> Result<(DatabaseManager, TempDir)> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite:{}", db_path.display());
    
    let db_manager = DatabaseManager::new(&database_url).await?;
    db_manager.run_migrations().await?;
    
    Ok((db_manager, temp_dir))
}

#[tokio::test]
async fn test_group_creation_and_retrieval() -> Result<()> {
    let (db, _temp_dir) = setup_test_db().await?;
    let chat_id = 12345i64;
    
    // Test group creation
    let group = Group::create(&db.pool, chat_id).await?;
    assert_eq!(group.telegram_chat_id, chat_id);
    assert_eq!(group.timezone, "UTC");
    assert_eq!(group.default_duration, 240);
    assert_eq!(group.reminder_hours, 24);
    
    // Test group retrieval
    let found_group = Group::find_by_chat_id(&db.pool, chat_id).await?;
    assert!(found_group.is_some());
    let found_group = found_group.unwrap();
    assert_eq!(found_group.telegram_chat_id, chat_id);
    assert_eq!(found_group.id, group.id);
    
    Ok(())
}

#[tokio::test]
async fn test_group_not_found() -> Result<()> {
    let (db, _temp_dir) = setup_test_db().await?;
    let non_existent_chat_id = 99999i64;
    
    let result = Group::find_by_chat_id(&db.pool, non_existent_chat_id).await?;
    assert!(result.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_session_creation_and_retrieval() -> Result<()> {
    let (db, _temp_dir) = setup_test_db().await?;
    let chat_id = 12345i64;
    let user_id = 67890i64;
    
    // Create a group first
    let group = Group::create(&db.pool, chat_id).await?;
    
    // Create a session
    let title = "Test Adventure".to_string();
    let session = Session::create(&db.pool, group.id, title.clone(), user_id).await?;
    
    assert_eq!(session.group_id, group.id);
    assert_eq!(session.title, title);
    assert_eq!(session.created_by, user_id);
    assert_eq!(session.status, "active");
    assert!(!session.id.is_empty()); // UUID should be generated
    
    // Test session retrieval
    let found_session = Session::find_by_id(&db.pool, &session.id).await?;
    assert!(found_session.is_some());
    let found_session = found_session.unwrap();
    assert_eq!(found_session.id, session.id);
    assert_eq!(found_session.title, title);
    
    Ok(())
}

#[tokio::test]
async fn test_session_not_found() -> Result<()> {
    let (db, _temp_dir) = setup_test_db().await?;
    let non_existent_id = "non-existent-uuid";
    
    let result = Session::find_by_id(&db.pool, non_existent_id).await?;
    assert!(result.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_session_option_creation() -> Result<()> {
    let (db, _temp_dir) = setup_test_db().await?;
    let chat_id = 12345i64;
    let user_id = 67890i64;
    
    // Create group and session
    let group = Group::create(&db.pool, chat_id).await?;
    let session = Session::create(&db.pool, group.id, "Test".to_string(), user_id).await?;
    
    // Create session option
    let datetime = Utc::now() + chrono::Duration::days(1);
    let duration = 240i64;
    let option = SessionOption::create(&db.pool, session.id.clone(), datetime, duration).await?;
    
    assert_eq!(option.session_id, session.id);
    assert_eq!(option.duration, duration);
    assert!(!option.confirmed);
    assert!(!option.id.is_empty()); // UUID should be generated
    
    // Verify the datetime was stored correctly
    let stored_datetime = chrono::DateTime::parse_from_rfc3339(&option.datetime)?;
    let _expected_datetime = datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let _actual_datetime = stored_datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    
    // Allow for small time differences (within 1 second)
    let stored_timestamp = stored_datetime.timestamp();
    let expected_timestamp = datetime.timestamp();
    assert!((stored_timestamp - expected_timestamp).abs() <= 1);
    
    Ok(())
}

#[tokio::test]
async fn test_response_upsert() -> Result<()> {
    let (db, _temp_dir) = setup_test_db().await?;
    let chat_id = 12345i64;
    let user_id = 67890i64;
    
    // Create group, session, and option
    let group = Group::create(&db.pool, chat_id).await?;
    let session = Session::create(&db.pool, group.id, "Test".to_string(), user_id).await?;
    let datetime = Utc::now() + chrono::Duration::days(1);
    let option = SessionOption::create(&db.pool, session.id.clone(), datetime, 240).await?;
    
    // Create initial response
    let username = Some("testuser".to_string());
    let response_text = "yes".to_string();
    let response = Response::upsert(
        &db.pool,
        session.id.clone(),
        option.id.clone(),
        user_id,
        username.clone(),
        response_text.clone(),
    ).await?;
    
    assert_eq!(response.session_id, session.id);
    assert_eq!(response.option_id, option.id);
    assert_eq!(response.user_id, user_id);
    assert_eq!(response.username, username);
    assert_eq!(response.response, response_text);
    
    // Update the response (upsert should replace)
    let new_response_text = "no".to_string();
    let updated_response = Response::upsert(
        &db.pool,
        session.id.clone(),
        option.id.clone(),
        user_id,
        username.clone(),
        new_response_text.clone(),
    ).await?;
    
    assert_eq!(updated_response.response, new_response_text);
    assert_ne!(updated_response.id, response.id); // Should be a new record
    
    // Verify only one response exists for this user/option
    let responses = Response::find_by_session(&db.pool, &session.id).await?;
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].response, new_response_text);
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_users_responses() -> Result<()> {
    let (db, _temp_dir) = setup_test_db().await?;
    let chat_id = 12345i64;
    
    // Create group, session, and option
    let group = Group::create(&db.pool, chat_id).await?;
    let session = Session::create(&db.pool, group.id, "Test".to_string(), 1).await?;
    let datetime = Utc::now() + chrono::Duration::days(1);
    let option = SessionOption::create(&db.pool, session.id.clone(), datetime, 240).await?;
    
    // Add responses from multiple users
    let users = vec![
        (1i64, "user1", "yes"),
        (2i64, "user2", "no"),
        (3i64, "user3", "maybe"),
    ];
    
    for (user_id, username, response_text) in users {
        Response::upsert(
            &db.pool,
            session.id.clone(),
            option.id.clone(),
            user_id,
            Some(username.to_string()),
            response_text.to_string(),
        ).await?;
    }
    
    // Verify all responses exist
    let responses = Response::find_by_session(&db.pool, &session.id).await?;
    assert_eq!(responses.len(), 3);
    
    // Check each response type exists
    let yes_count = responses.iter().filter(|r| r.response == "yes").count();
    let no_count = responses.iter().filter(|r| r.response == "no").count();
    let maybe_count = responses.iter().filter(|r| r.response == "maybe").count();
    
    assert_eq!(yes_count, 1);
    assert_eq!(no_count, 1);
    assert_eq!(maybe_count, 1);
    
    Ok(())
}

#[tokio::test]
async fn test_database_constraints() -> Result<()> {
    let (db, _temp_dir) = setup_test_db().await?;
    let chat_id = 12345i64;
    
    // Test unique constraint on telegram_chat_id
    let _group1 = Group::create(&db.pool, chat_id).await?;
    
    // This should fail due to unique constraint
    let result = Group::create(&db.pool, chat_id).await;
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_foreign_key_relationships() -> Result<()> {
    let (db, _temp_dir) = setup_test_db().await?;
    let chat_id = 12345i64;
    let user_id = 67890i64;
    
    // Create group and session
    let group = Group::create(&db.pool, chat_id).await?;
    let session = Session::create(&db.pool, group.id, "Test".to_string(), user_id).await?;
    
    // Create session option
    let datetime = Utc::now() + chrono::Duration::days(1);
    let option = SessionOption::create(&db.pool, session.id.clone(), datetime, 240).await?;
    
    // Create response
    let response = Response::upsert(
        &db.pool,
        session.id.clone(),
        option.id.clone(),
        user_id,
        Some("testuser".to_string()),
        "yes".to_string(),
    ).await?;
    
    // Verify the relationships
    assert_eq!(session.group_id, group.id);
    assert_eq!(option.session_id, session.id);
    assert_eq!(response.session_id, session.id);
    assert_eq!(response.option_id, option.id);
    
    Ok(())
}