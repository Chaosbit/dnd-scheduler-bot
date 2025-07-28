use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::database::{connection::DatabaseManager, models::*};
use crate::utils::{
    datetime::{parse_datetime, format_datetime}, 
    markdown::escape_markdown,
    validation::{validate_session_title, validate_time_options, validate_telegram_chat_id}
};
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
    
    // Validate inputs
    if let Err(e) = validate_telegram_chat_id(chat_id) {
        bot.send_message(msg.chat.id, format!("❌ Invalid chat: {e}")).await?;
        return Ok(());
    }
    
    if let Err(e) = validate_session_title(&title) {
        bot.send_message(msg.chat.id, format!("❌ Invalid session title: {e}")).await?;
        return Ok(());
    }
    
    let validated_options = match validate_time_options(&options) {
        Ok(opts) => opts,
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Invalid time options: {e}")).await?;
            return Ok(());
        }
    };
    
    // Get or create group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => group,
        Ok(None) => Group::create(&db.pool, chat_id).await.map_err(|e| {
            teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string()))
        })?,
        Err(e) => return Err(teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string()))),
    };
    
    // Create session
    let session = Session::create(&db.pool, group.id, title.clone(), user_id).await.map_err(|e| {
        teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string()))
    })?;
    
    // Use the validated options instead of re-parsing
    let mut session_options = Vec::new();
    
    for option_str in &validated_options {
        // Parse the datetime from the option string
        let datetime = match parse_datetime(option_str) {
            Ok(dt) => dt,
            Err(_) => {
                // If parsing fails, send error message and return
                bot.send_message(msg.chat.id, 
                    format!("❌ Could not parse date/time: '{option_str}'\n\nPlease use formats like:\n• Friday 19:00\n• Monday 14.30\n• Tuesday 20:00")
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
    for option in session_options.iter() {
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
    
    let mut message_text = format!("🎲 **{title}**\n\nSelect your availability for each option:\n\n");
    
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
    
    // Send confirmation message to the session creator
    bot.send_message(msg.chat.id, format!("✅ Session '{}' created successfully!", title))
        .reply_to_message_id(msg.id)
        .await?;
    
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
