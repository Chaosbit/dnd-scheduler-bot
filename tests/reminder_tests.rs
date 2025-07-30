#![allow(clippy::unwrap_used)]

use dnd_scheduler_bot::database::models::{Reminder, Session, Group, SessionOption};
use dnd_scheduler_bot::database::connection::DatabaseManager;
use tempfile::{tempdir, TempDir};
use chrono::{Utc, Duration};

async fn setup_test_db() -> (DatabaseManager, TempDir) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db_url = format!("sqlite:{}", db_path.to_string_lossy());
    
    let db = DatabaseManager::new(&db_url).await.unwrap();
    db.run_migrations().await.unwrap();
    (db, dir)
}

#[tokio::test]
async fn test_reminder_creation() {
    let (db, _temp_dir) = setup_test_db().await;
    
    // Create group and session first for foreign key constraint
    let group = Group::create(&db.pool, 12345).await.unwrap();
    let session = Session::create(&db.pool, group.id, "Test Session".to_string(), 67890).await.unwrap();
    
    let days_before = 7i64;
    
    let reminder = Reminder::create(&db.pool, session.id.clone(), days_before)
        .await
        .unwrap();
    
    assert_eq!(reminder.session_id, session.id);
    assert_eq!(reminder.days_before, days_before);
    assert!(!reminder.id.is_empty());
    assert!(!reminder.sent_at.is_empty());
}

#[tokio::test] 
async fn test_reminder_exists() {
    let (db, _temp_dir) = setup_test_db().await;
    
    // Create group and session first for foreign key constraint
    let group = Group::create(&db.pool, 12346).await.unwrap();
    let session = Session::create(&db.pool, group.id, "Test Session 2".to_string(), 67891).await.unwrap();
    
    let days_before = 14i64;
    
    // Should not exist initially
    let exists = Reminder::exists(&db.pool, &session.id, days_before)
        .await
        .unwrap();
    assert!(!exists);
    
    // Create reminder
    Reminder::create(&db.pool, session.id.clone(), days_before)
        .await
        .unwrap();
    
    // Should exist now
    let exists = Reminder::exists(&db.pool, &session.id, days_before)
        .await
        .unwrap();
    assert!(exists);
    
    // Different days_before should not exist
    let exists_different = Reminder::exists(&db.pool, &session.id, 3)
        .await
        .unwrap();
    assert!(!exists_different);
}

#[tokio::test]
async fn test_reminder_find_by_session() {
    let (db, _temp_dir) = setup_test_db().await;
    
    // Create group and session first for foreign key constraint
    let group = Group::create(&db.pool, 12347).await.unwrap();
    let session = Session::create(&db.pool, group.id, "Test Session 3".to_string(), 67892).await.unwrap();
    
    // Create multiple reminders for same session
    Reminder::create(&db.pool, session.id.clone(), 14).await.unwrap();
    Reminder::create(&db.pool, session.id.clone(), 7).await.unwrap();
    Reminder::create(&db.pool, session.id.clone(), 3).await.unwrap();
    
    let reminders = Reminder::find_by_session(&db.pool, &session.id)
        .await
        .unwrap();
    
    assert_eq!(reminders.len(), 3);
    
    // Should be ordered by days_before DESC
    assert_eq!(reminders[0].days_before, 14);
    assert_eq!(reminders[1].days_before, 7);
    assert_eq!(reminders[2].days_before, 3);
    
    // All should have same session_id
    for reminder in &reminders {
        assert_eq!(reminder.session_id, session.id);
    }
}

#[tokio::test]
async fn test_reminder_unique_constraint() {
    let (db, _temp_dir) = setup_test_db().await;
    
    // Create group and session first for foreign key constraint
    let group = Group::create(&db.pool, 12348).await.unwrap();
    let session = Session::create(&db.pool, group.id, "Test Session 4".to_string(), 67893).await.unwrap();
    
    let days_before = 7i64;
    
    // Create first reminder
    let first = Reminder::create(&db.pool, session.id.clone(), days_before)
        .await
        .unwrap();
    
    // Attempting to create duplicate should fail
    let result = Reminder::create(&db.pool, session.id.clone(), days_before).await;
    assert!(result.is_err());
    
    // But different days_before should work
    let different = Reminder::create(&db.pool, session.id.clone(), 3)
        .await
        .unwrap();
    
    assert_eq!(first.session_id, different.session_id);
    assert_ne!(first.days_before, different.days_before);
}

#[tokio::test]
async fn test_reminder_with_real_session() {
    let (db, _temp_dir) = setup_test_db().await;
    
    // Create group first
    let group = Group::create(&db.pool, 12345).await.unwrap();
    
    // Create session
    let session = Session::create(&db.pool, group.id, "Test Session".to_string(), 67890)
        .await
        .unwrap();
    
    // Create session option
    let future_date = Utc::now() + Duration::days(10);
    let _option = SessionOption::create(&db.pool, session.id.clone(), future_date, 240)
        .await
        .unwrap();
    
    // Create reminder for this session
    let reminder = Reminder::create(&db.pool, session.id.clone(), 3)
        .await
        .unwrap();
    
    assert_eq!(reminder.session_id, session.id);
    
    // Verify reminder can be found
    let found_reminders = Reminder::find_by_session(&db.pool, &session.id)
        .await
        .unwrap();
    
    assert_eq!(found_reminders.len(), 1);
    assert_eq!(found_reminders[0].id, reminder.id);
}

#[tokio::test]
async fn test_reminder_cleanup_on_session_delete() {
    let (db, _temp_dir) = setup_test_db().await;
    
    // Create group and session
    let group = Group::create(&db.pool, 54321).await.unwrap();
    let session = Session::create(&db.pool, group.id, "Temp Session".to_string(), 98765)
        .await
        .unwrap();
    
    // Create reminder
    let _reminder = Reminder::create(&db.pool, session.id.clone(), 7)
        .await
        .unwrap();
    
    // Verify reminder exists
    let exists = Reminder::exists(&db.pool, &session.id, 7).await.unwrap();
    assert!(exists);
    
    // Delete session (this should cascade delete the reminder due to foreign key)
    sqlx::query!("DELETE FROM sessions WHERE id = ?", session.id)
        .execute(&db.pool)
        .await
        .unwrap();
    
    // Reminder should no longer exist
    let exists_after = Reminder::exists(&db.pool, &session.id, 7).await.unwrap();
    assert!(!exists_after);
}