use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub session_id: String,
    pub days_before: i64,
    pub sent_at: String,
}

impl Reminder {
    pub async fn create(
        pool: &sqlx::SqlitePool,
        session_id: String,
        days_before: i64,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let sent_at = Utc::now().to_rfc3339();
        
        sqlx::query(
            "INSERT INTO reminders (id, session_id, days_before, sent_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&session_id)
        .bind(days_before)
        .bind(&sent_at)
        .execute(pool)
        .await?;
        
        Ok(Reminder {
            id,
            session_id,
            days_before,
            sent_at,
        })
    }
    
    pub async fn exists(
        pool: &sqlx::SqlitePool,
        session_id: &str,
        days_before: i64,
    ) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM reminders WHERE session_id = ? AND days_before = ?"
        )
        .bind(session_id)
        .bind(days_before)
        .fetch_one(pool)
        .await?;
        
        Ok(count > 0)
    }
    
    pub async fn find_by_session(
        pool: &sqlx::SqlitePool,
        session_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Reminder>(
            "SELECT id, session_id, days_before, sent_at FROM reminders WHERE session_id = ? ORDER BY days_before DESC"
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
    }
}