use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::database::{connection::DatabaseManager, models::*};
use crate::utils::{
    datetime::{parse_datetime, format_datetime}, 
    markdown::escape_markdown,
    validation::{validate_session_title, validate_time_options, validate_telegram_chat_id},
    feedback::{CommandFeedback, ProgressTracker}
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
    
    // Initialize feedback system
    let feedback = CommandFeedback::new(bot.clone(), msg.chat.id);
    let mut progress = ProgressTracker::new(feedback, 4);
    
    // Start progress tracking
    progress.start("Creating new D&D session...").await?;
    
    // Validate inputs
    if let Err(e) = validate_telegram_chat_id(chat_id) {
        let error_msg = format!("Invalid chat configuration: {}", e);
        let suggestion = "This command can only be used in properly configured chat groups.";
        CommandFeedback::new(bot.clone(), msg.chat.id).validation_error(&error_msg, suggestion).await?;
        progress.error("Failed to create session due to chat validation error").await?;
        return Ok(());
    }
    
    if let Err(e) = validate_session_title(&title) {
        let error_msg = format!("Invalid session title: {}", e);
        let suggestion = "Use a title between 3-100 characters. Example: 'Weekly D&D Session'";
        CommandFeedback::new(bot.clone(), msg.chat.id).validation_error(&error_msg, suggestion).await?;
        progress.error("Failed to create session due to invalid title").await?;
        return Ok(());
    }
    
    let validated_options = match validate_time_options(&options) {
        Ok(opts) => {
            progress.next_step("Time options validated successfully").await?;
            opts
        },
        Err(e) => {
            let error_msg = format!("Invalid time options: {}", e);
            let suggestion = "Use formats like 'Friday 19:00, Saturday 14:30'. You can specify multiple times separated by commas.";
            CommandFeedback::new(bot.clone(), msg.chat.id).validation_error(&error_msg, suggestion).await?;
            progress.error("Failed to create session due to invalid time options").await?;
            return Ok(());
        }
    };
    
    // Get or create group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => {
            progress.next_step("Group found, creating session...").await?;
            group
        },
        Ok(None) => {
            progress.next_step("Setting up new group and creating session...").await?;
            Group::create(&db.pool, chat_id).await.map_err(|e| {
                tracing::error!("Failed to create group: {}", e);
                teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string()))
            })?
        },
        Err(e) => {
            progress.error("Failed to access group information").await?;
            return Err(teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string())));
        }
    };
    
    // Create session
    let session = Session::create(&db.pool, group.id, title.clone(), user_id).await.map_err(|e| {
        tracing::error!("Failed to create session: {}", e);
        teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string()))
    })?;
    
    // Parse and create session options
    let mut session_options = Vec::new();
    let total_options = validated_options.len();
    
    for (i, option_str) in validated_options.iter().enumerate() {
        // Parse the datetime from the option string
        let datetime = match parse_datetime(option_str) {
            Ok(dt) => dt,
            Err(_e) => {
                let error_msg = format!("Could not parse date/time: '{}'", option_str);
                let suggestion = "Please use formats like 'Friday 19:00', 'Monday 14:30', or 'Tuesday 20:00'";
                CommandFeedback::new(bot.clone(), msg.chat.id).validation_error(&error_msg, suggestion).await?;
                progress.error(&format!("Failed to parse time option {}/{}", i + 1, total_options)).await?;
                return Ok(());
            }
        };
        
        let option = SessionOption::create(&db.pool, session.id.clone(), datetime, 240).await.map_err(|e| {
            tracing::error!("Failed to create session option: {}", e);
            teloxide::RequestError::Api(teloxide::ApiError::Unknown(e.to_string()))
        })?;
        session_options.push(option);
    }
    
    progress.next_step(&format!("Created session with {} time options", session_options.len())).await?;
    
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
        CommandFeedback::new(bot.clone(), msg.chat.id).warning("Session created but message tracking may not work perfectly").await?;
    }
    
    // Complete progress and send detailed success feedback
    let success_message = format!(
        "Session '{}' created successfully!\n\n📊 Session Details:\n• {} time options available\n• Session ID: {}\n• Voting is now open!\n\n💡 Use /list to see all active sessions",
        title,
        session_options.len(),
        &session.id[..8] // Show first 8 chars of ID
    );
    
    progress.complete(&success_message).await?;
    
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
