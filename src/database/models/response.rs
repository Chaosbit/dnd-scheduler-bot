use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub session_id: String,
    pub option_id: String,
    pub user_id: i64,
    pub username: Option<String>,
    pub response: String, // 'yes', 'no', 'maybe'
    pub created_at: String,
}

impl Response {
    pub async fn upsert(
        pool: &sqlx::SqlitePool,
        session_id: String,
        option_id: String,
        user_id: i64,
        username: Option<String>,
        response: String,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        // Delete existing response for this user/option
        sqlx::query!(
            "DELETE FROM responses WHERE session_id = ? AND option_id = ? AND user_id = ?",
            session_id,
            option_id,
            user_id
        )
        .execute(pool)
        .await?;
        
        // Insert new response
        sqlx::query!(
            r#"
            INSERT INTO responses (id, session_id, option_id, user_id, username, response, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            session_id,
            option_id,
            user_id,
            username,
            response,
            now
        )
        .execute(pool)
        .await?;
        
        // Return the created response
        Ok(Response {
            id,
            session_id,
            option_id,
            user_id,
            username,
            response,
            created_at: now.clone(),
        })
    }

    pub async fn find_by_session(
        pool: &sqlx::SqlitePool,
        session_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Response>(
            "SELECT id, session_id, option_id, user_id, username, response, created_at FROM responses WHERE session_id = ?"
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
    }

    /// Batch fetch responses for multiple sessions to avoid N+1 queries
    pub async fn find_by_sessions(
        pool: &sqlx::SqlitePool,
        session_ids: &[String],
    ) -> Result<Vec<Self>, sqlx::Error> {
        if session_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = session_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT id, session_id, option_id, user_id, username, response, created_at FROM responses WHERE session_id IN ({placeholders}) ORDER BY session_id, created_at"
        );

        let mut query_builder = sqlx::query_as::<_, Response>(&query);
        for session_id in session_ids {
            query_builder = query_builder.bind(session_id);
        }

        query_builder.fetch_all(pool).await
    }
}
