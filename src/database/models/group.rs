use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Group {
    pub id: i64,
    pub telegram_chat_id: i64,
    pub timezone: String,
    pub default_duration: i64, // minutes
    pub reminder_hours: i64,
    pub created_at: String,
}

impl Group {
    pub async fn find_by_chat_id(
        pool: &sqlx::SqlitePool,
        chat_id: i64,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            "SELECT id, telegram_chat_id, timezone, default_duration, reminder_hours, created_at FROM groups WHERE telegram_chat_id = ?"
        )
        .bind(chat_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &sqlx::SqlitePool,
        chat_id: i64,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        sqlx::query!(
            r#"
            INSERT INTO groups (telegram_chat_id, timezone, default_duration, reminder_hours, created_at)
            VALUES (?, 'UTC', 240, 24, ?)
            "#,
            chat_id,
            now
        )
        .execute(pool)
        .await?;
        
        // Fetch the created group
        Self::find_by_chat_id(pool, chat_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }
}
