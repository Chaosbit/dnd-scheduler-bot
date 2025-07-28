use dnd_scheduler_bot::{
    database::{
        connection::DatabaseManager,
        models::{Group, Session, SessionOption, Response},
    },
};
use tempfile::TempDir;
use chrono::{Utc, Duration};

/// Helper function to create a test database
async fn create_test_db() -> (DatabaseManager, TempDir) {
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
    
    (db, temp_dir)
}

// Helper functions removed to focus on database integration testing

#[tokio::test]
async fn test_schedule_command_database_operations() {
    let (db, _temp_dir) = create_test_db().await;
    let chat_id = -1001234567890_i64;
    let user_id = 123456789_u64;
    
    // Create a test group in the database
    let group = Group::create(&db.pool, chat_id)
        .await
        .expect("Failed to create test group");
    
    // Test the database operations that would be performed by schedule command
    let session = Session::create(
        &db.pool,
        group.id,
        "Test Session".to_string(),
        user_id as i64,
    ).await.expect("Failed to create session");
    
    // Create session options
    let datetime1 = Utc::now() + Duration::days(1);
    let datetime2 = Utc::now() + Duration::days(7);
    
    let _option1 = SessionOption::create(
        &db.pool,
        session.id.clone(),
        datetime1,
        180, // 3 hours
    ).await.expect("Failed to create option 1");
    
    let _option2 = SessionOption::create(
        &db.pool,
        session.id.clone(),
        datetime2,
        180,
    ).await.expect("Failed to create option 2");
    
    // Verify session was created
    let found_session = Session::find_by_id(&db.pool, &session.id)
        .await
        .expect("Failed to find session")
        .expect("Session not found");
    
    assert_eq!(found_session.title, "Test Session");
    assert_eq!(found_session.status, "active");
}

#[tokio::test]
async fn test_list_command_database_operations() {
    let (db, _temp_dir) = create_test_db().await;
    let chat_id = -1001234567890_i64;
    let user_id = 123456789_u64;
    
    // Create test group and session
    let group = Group::create(&db.pool, chat_id)
        .await
        .expect("Failed to create test group");
    
    let session = Session::create(
        &db.pool,
        group.id,
        "Test Session".to_string(),
        user_id as i64,
    ).await.expect("Failed to create session");
    
    // Create session options
    let datetime = Utc::now() + Duration::days(1);
    let option = SessionOption::create(
        &db.pool,
        session.id.clone(),
        datetime,
        180,
    ).await.expect("Failed to create session option");
    
    // Create some responses
    let _response1 = Response::upsert(
        &db.pool,
        session.id.clone(),
        option.id.clone(),
        user_id as i64,
        Some("testuser".to_string()),
        "yes".to_string(),
    ).await.expect("Failed to create response");
    
    let _response2 = Response::upsert(
        &db.pool,
        session.id.clone(),
        option.id.clone(),
        (user_id + 1) as i64,
        Some("testuser2".to_string()),
        "maybe".to_string(),
    ).await.expect("Failed to create response 2");
    
    // Test the database queries used by list command
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT id, group_id, title, message_id, status, deadline, created_by, created_at 
         FROM sessions 
         WHERE group_id = ? AND status IN ('active', 'confirmed') 
         ORDER BY created_at DESC"
    )
    .bind(group.id)
    .fetch_all(&db.pool)
    .await
    .expect("Failed to fetch sessions");
    
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].title, "Test Session");
    
    // Test batch queries used by optimized list command
    let session_ids: Vec<String> = sessions.iter().map(|s| s.id.clone()).collect();
    
    let options = SessionOption::find_by_sessions(&db.pool, &session_ids)
        .await
        .expect("Failed to batch fetch options");
    
    let responses = Response::find_by_sessions(&db.pool, &session_ids)
        .await
        .expect("Failed to batch fetch responses");
    
    assert_eq!(options.len(), 1);
    assert_eq!(responses.len(), 2);
    assert!(responses.iter().any(|r| r.response == "yes"));
    assert!(responses.iter().any(|r| r.response == "maybe"));
}

#[tokio::test]
async fn test_callback_handler_integration() {
    let (db, _temp_dir) = create_test_db().await;
    let chat_id = -1001234567890_i64;
    let user_id = 123456789_u64;
    
    // Create test data
    let group = Group::create(&db.pool, chat_id)
        .await
        .expect("Failed to create test group");
    
    let session = Session::create(
        &db.pool,
        group.id,
        "Test Session".to_string(),
        user_id as i64,
    ).await.expect("Failed to create session");
    
    let datetime = Utc::now() + Duration::days(1);
    let option = SessionOption::create(
        &db.pool,
        session.id.clone(),
        datetime,
        180,
    ).await.expect("Failed to create session option");
    
    // Test callback data parsing and response creation
    let callback_data = format!("{}:{}:yes", session.id, option.id);
    
    // Simulate the database operations that callback handler would perform
    let parts: Vec<&str> = callback_data.split(':').collect();
    assert_eq!(parts.len(), 3);
    
    let session_id = parts[0];
    let option_id = parts[1];
    let response = parts[2];
    
    // Test response upsert
    let response_record = Response::upsert(
        &db.pool,
        session_id.to_string(),
        option_id.to_string(),
        user_id as i64,
        Some("testuser".to_string()),
        response.to_string(),
    ).await.expect("Failed to create response");
    
    assert_eq!(response_record.response, "yes");
    assert_eq!(response_record.session_id, session.id);
    assert_eq!(response_record.option_id, option.id);
    
    // Test that subsequent upsert replaces the response
    let updated_response = Response::upsert(
        &db.pool,
        session_id.to_string(),
        option_id.to_string(),
        user_id as i64,
        Some("testuser".to_string()),
        "no".to_string(),
    ).await.expect("Failed to update response");
    
    assert_eq!(updated_response.response, "no");
    
    // Verify only one response exists for this user/option combination
    let all_responses = Response::find_by_session(&db.pool, session_id)
        .await
        .expect("Failed to find responses");
    
    assert_eq!(all_responses.len(), 1);
    assert_eq!(all_responses[0].response, "no");
}

#[tokio::test]
async fn test_session_management_integration() {
    let (db, _temp_dir) = create_test_db().await;
    let chat_id = -1001234567890_i64;
    let user_id = 123456789_u64;
    
    // Create test data
    let group = Group::create(&db.pool, chat_id)
        .await
        .expect("Failed to create test group");
    
    let session = Session::create(
        &db.pool,
        group.id,
        "Test Session".to_string(),
        user_id as i64,
    ).await.expect("Failed to create session");
    
    // Test session confirmation
    sqlx::query!(
        "UPDATE sessions SET status = 'confirmed' WHERE id = ?",
        session.id
    )
    .execute(&db.pool)
    .await
    .expect("Failed to confirm session");
    
    let confirmed_session = Session::find_by_id(&db.pool, &session.id)
        .await
        .expect("Failed to find session")
        .expect("Session not found");
    
    assert_eq!(confirmed_session.status, "confirmed");
    
    // Test session cancellation
    sqlx::query!(
        "UPDATE sessions SET status = 'cancelled' WHERE id = ?",
        session.id
    )
    .execute(&db.pool)
    .await
    .expect("Failed to cancel session");
    
    let cancelled_session = Session::find_by_id(&db.pool, &session.id)
        .await
        .expect("Failed to find session")
        .expect("Session not found");
    
    assert_eq!(cancelled_session.status, "cancelled");
    
    // Test deadline setting
    let deadline = Utc::now() + Duration::days(2);
    let deadline_str = deadline.to_rfc3339();
    
    sqlx::query!(
        "UPDATE sessions SET deadline = ? WHERE id = ?",
        deadline_str,
        session.id
    )
    .execute(&db.pool)
    .await
    .expect("Failed to set deadline");
    
    let session_with_deadline = Session::find_by_id(&db.pool, &session.id)
        .await
        .expect("Failed to find session")
        .expect("Session not found");
    
    assert!(session_with_deadline.deadline.is_some());
    assert_eq!(session_with_deadline.deadline.unwrap(), deadline_str);
}

#[tokio::test]
async fn test_batch_query_performance() {
    let (db, _temp_dir) = create_test_db().await;
    let chat_id = -1001234567890_i64;
    let user_id = 123456789_u64;
    
    // Create test group
    let group = Group::create(&db.pool, chat_id)
        .await
        .expect("Failed to create test group");
    
    // Create multiple sessions for batch testing
    let mut session_ids = Vec::new();
    for i in 0..5 {
        let session = Session::create(
            &db.pool,
            group.id,
            format!("Test Session {}", i + 1),
            user_id as i64,
        ).await.expect("Failed to create session");
        
        // Create options for each session
        let datetime = Utc::now() + Duration::days(i + 1);
        let _option = SessionOption::create(
            &db.pool,
            session.id.clone(),
            datetime,
            180,
        ).await.expect("Failed to create session option");
        
        session_ids.push(session.id);
    }
    
    // Test batch fetching of session options
    let start_time = std::time::Instant::now();
    let all_options = SessionOption::find_by_sessions(&db.pool, &session_ids)
        .await
        .expect("Failed to batch fetch session options");
    let batch_duration = start_time.elapsed();
    
    assert_eq!(all_options.len(), 5);
    
    // Test individual fetching for comparison (this would be the N+1 problem)
    let start_time = std::time::Instant::now();
    let mut individual_options = Vec::new();
    for session_id in &session_ids {
        let options = SessionOption::find_by_session(&db.pool, session_id)
            .await
            .expect("Failed to fetch individual session options");
        individual_options.extend(options);
    }
    let individual_duration = start_time.elapsed();
    
    assert_eq!(individual_options.len(), 5);
    
    // Batch queries should be faster or at least comparable
    // In a real database with more data, batch queries would be significantly faster
    println!("Batch query time: {:?}", batch_duration);
    println!("Individual query time: {:?}", individual_duration);
    
    // Verify data consistency between both approaches
    assert_eq!(all_options.len(), individual_options.len());
}

#[tokio::test]
async fn test_database_indexes_usage() {
    let (db, _temp_dir) = create_test_db().await;
    let chat_id = -1001234567890_i64;
    let user_id = 123456789_u64;
    
    // Create test data to exercise the indexes
    let group = Group::create(&db.pool, chat_id)
        .await
        .expect("Failed to create test group");
    
    let session = Session::create(
        &db.pool,
        group.id,
        "Test Session".to_string(),
        user_id as i64,
    ).await.expect("Failed to create session");
    
    let datetime = Utc::now() + Duration::days(1);
    let option = SessionOption::create(
        &db.pool,
        session.id.clone(),
        datetime,
        180,
    ).await.expect("Failed to create session option");
    
    // Create multiple responses to test indexes
    for i in 0..10 {
        let _response = Response::upsert(
            &db.pool,
            session.id.clone(),
            option.id.clone(),
            (user_id + i) as i64,
            Some(format!("user{}", i)),
            if i % 3 == 0 { "yes" } else if i % 3 == 1 { "no" } else { "maybe" }.to_string(),
        ).await.expect("Failed to create response");
    }
    
    // Test queries that should benefit from indexes
    
    // 1. Find sessions by group_id (uses idx_sessions_group_id)
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT id, group_id, title, message_id, status, deadline, created_by, created_at 
         FROM sessions 
         WHERE group_id = ?"
    )
    .bind(group.id)
    .fetch_all(&db.pool)
    .await
    .expect("Failed to query sessions by group_id");
    
    assert_eq!(sessions.len(), 1);
    
    // 2. Find session options by session_id (uses idx_session_options_session_id)
    let options = sqlx::query_as::<_, SessionOption>(
        "SELECT id, session_id, datetime, duration, confirmed 
         FROM session_options 
         WHERE session_id = ?"
    )
    .bind(&session.id)
    .fetch_all(&db.pool)
    .await
    .expect("Failed to query session options by session_id");
    
    assert_eq!(options.len(), 1);
    
    // 3. Find responses by session_id (uses idx_responses_session_id)
    let responses = sqlx::query_as::<_, Response>(
        "SELECT id, session_id, option_id, user_id, username, response, created_at 
         FROM responses 
         WHERE session_id = ?"
    )
    .bind(&session.id)
    .fetch_all(&db.pool)
    .await
    .expect("Failed to query responses by session_id");
    
    assert_eq!(responses.len(), 10);
    
    // 4. Find responses by user_id (uses idx_responses_user_id)
    let user_responses = sqlx::query_as::<_, Response>(
        "SELECT id, session_id, option_id, user_id, username, response, created_at 
         FROM responses 
         WHERE user_id = ?"
    )
    .bind(user_id as i64)
    .fetch_all(&db.pool)
    .await
    .expect("Failed to query responses by user_id");
    
    assert_eq!(user_responses.len(), 1);
    
    // 5. Composite query for sessions by group_id and status (uses idx_sessions_group_status)
    let active_sessions = sqlx::query_as::<_, Session>(
        "SELECT id, group_id, title, message_id, status, deadline, created_by, created_at 
         FROM sessions 
         WHERE group_id = ? AND status = ?"
    )
    .bind(group.id)
    .bind("active")
    .fetch_all(&db.pool)
    .await
    .expect("Failed to query sessions by group_id and status");
    
    assert_eq!(active_sessions.len(), 1);
}