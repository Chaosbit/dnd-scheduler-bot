use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::database::{connection::DatabaseManager, models::*};
use crate::utils::{datetime::{parse_datetime, format_datetime}, markdown::escape_markdown};
use chrono::Utc;

pub async fn handle_schedule(
    bot: Bot,
    msg: Message,
    title: String,
    options: String,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let user_id = msg.from().map(|u| u.id.0 as i64).unwrap_or(0);
    
    // Get or create group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => group,
        Ok(None) => Group::create(&db.pool, chat_id).await.map_err(|e| {
            teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string()))
        })?,
        Err(e) => return Err(teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string())).into()),
    };
    
    // Create session
    let session = Session::create(&db.pool, group.id, title.clone(), user_id).await.map_err(|e| {
        teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string()))
    })?;
    
    // Parse options (for now, just split by commas - TODO: proper date parsing)
    let option_strings: Vec<&str> = options.split(',').map(|s| s.trim()).collect();
    let mut session_options = Vec::new();
    
    for option_str in &option_strings {
        // Parse the datetime from the option string
        let datetime = match parse_datetime(option_str) {
            Ok(dt) => dt,
            Err(_) => {
                // If parsing fails, send error message and return
                bot.send_message(msg.chat.id, 
                    format!("❌ Could not parse date/time: '{}'\n\nPlease use formats like:\n• Friday 19:00\n• Monday 14.30\n• Tuesday 20:00", option_str)
                ).await?;
                return Ok(());
            }
        };
        
        let option = SessionOption::create(&db.pool, session.id.clone(), datetime, 240).await.map_err(|e| {
            teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string()))
        })?;
        session_options.push(option);
    }
    
    // Create inline keyboard
    let mut keyboard_rows = Vec::new();
    for (_i, option) in session_options.iter().enumerate() {
        let row = vec![
            InlineKeyboardButton::callback(
                "✅ Yes",
                format!("{}:{}:yes", session.id, option.id),
            ),
            InlineKeyboardButton::callback(
                "❌ No", 
                format!("{}:{}:no", session.id, option.id),
            ),
            InlineKeyboardButton::callback(
                "❓ Maybe",
                format!("{}:{}:maybe", session.id, option.id),
            ),
        ];
        keyboard_rows.push(row);
    }
    
    let keyboard = InlineKeyboardMarkup::new(keyboard_rows);
    
    let mut message_text = format!("🎲 **{}**\n\nSelect your availability for each option:\n\n", title);
    
    for (i, option) in session_options.iter().enumerate() {
        let datetime_str = chrono::DateTime::parse_from_rfc3339(&option.datetime)
            .map(|dt| format_datetime(&dt.with_timezone(&Utc)))
            .unwrap_or_else(|_| option.datetime.clone());
        
        message_text.push_str(&format!("**{}\\. {}**\n", i + 1, escape_markdown(&datetime_str)));
        message_text.push_str("✅ 0 • ❌ 0 • ❓ 0\n\n");
    }
    
    let sent_message = bot.send_message(msg.chat.id, message_text)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    
    // Store the message ID in the session for future updates
    if let Err(e) = update_session_message_id(&db.pool, &session.id, sent_message.id.0).await {
        tracing::warn!("Failed to store message ID: {}", e);
    }
    
    Ok(())
}

async fn update_session_message_id(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    message_id: i32,
) -> Result<(), sqlx::Error> {
    let message_id_i64 = message_id as i64;
    sqlx::query!(
        "UPDATE sessions SET message_id = ? WHERE id = ?",
        message_id_i64,
        session_id
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

// Helper function to escape markdown characters
