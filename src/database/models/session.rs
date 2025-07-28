use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub group_id: i64,
    pub title: String,
    pub message_id: Option<i64>,
    pub status: String,
    pub deadline: Option<String>,
    pub created_by: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SessionOption {
    pub id: String,
    pub session_id: String,
    pub datetime: String,
    pub duration: i64, // minutes
    pub confirmed: bool,
}

impl Session {
    pub async fn create(
        pool: &sqlx::SqlitePool,
        group_id: i64,
        title: String,
        created_by: i64,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        sqlx::query!(
            r#"
            INSERT INTO sessions (id, group_id, title, status, created_by, created_at)
            VALUES (?, ?, ?, 'active', ?, ?)
            "#,
            id,
            group_id,
            title,
            created_by,
            now
        )
        .execute(pool)
        .await?;
        
        Self::find_by_id(pool, &id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn find_by_id(
        pool: &sqlx::SqlitePool,
        session_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Session>(
            "SELECT id, group_id, title, message_id, status, deadline, created_by, created_at FROM sessions WHERE id = ?"
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await
    }
}

impl SessionOption {
    pub async fn create(
        pool: &sqlx::SqlitePool,
        session_id: String,
        datetime: DateTime<Utc>,
        duration: i64,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let datetime_str = datetime.to_rfc3339();
        
        sqlx::query!(
            r#"
            INSERT INTO session_options (id, session_id, datetime, duration, confirmed)
            VALUES (?, ?, ?, ?, false)
            "#,
            id,
            session_id,
            datetime_str,
            duration
        )
        .execute(pool)
        .await?;
        
        // Return the created session option
        Ok(SessionOption {
            id,
            session_id,
            datetime: datetime_str,
            duration,
            confirmed: false,
        })
    }

    pub async fn find_by_session(
        pool: &sqlx::SqlitePool,
        session_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, SessionOption>(
            "SELECT id, session_id, datetime, duration, confirmed FROM session_options WHERE session_id = ? ORDER BY datetime"
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
    }

    /// Batch fetch session options for multiple sessions to avoid N+1 queries
    pub async fn find_by_sessions(
        pool: &sqlx::SqlitePool,
        session_ids: &[String],
    ) -> Result<Vec<Self>, sqlx::Error> {
        if session_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = session_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT id, session_id, datetime, duration, confirmed FROM session_options WHERE session_id IN ({}) ORDER BY session_id, datetime",
            placeholders
        );

        let mut query_builder = sqlx::query_as::<_, SessionOption>(&query);
        for session_id in session_ids {
            query_builder = query_builder.bind(session_id);
        }

        query_builder.fetch_all(pool).await
    }
}
