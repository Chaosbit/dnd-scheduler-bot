use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use crate::database::connection::DatabaseManager;
use crate::database::models::*;
use crate::utils::{datetime::format_datetime, markdown::escape_markdown};
use chrono::Utc;
use std::collections::HashMap;

pub async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    db: DatabaseManager,
) -> ResponseResult<()> {
    if let Some(data) = q.data.clone() {
        // Handle settings callbacks
        if data.starts_with("settings:") {
            return handle_settings_callback(bot, q, data, &db).await;
        }
        
        // Parse callback data: "session_id:option_id:response"
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() == 3 {
            let session_id = parts[0];
            let option_id = parts[1];
            let response = parts[2];
            
            let user = &q.from;
            let user_id = user.id.0 as i64;
            let username = user.username.clone();
            
            // Validate response type
            match crate::utils::validation::validate_response_type(response) {
                Ok(_) => {},
                Err(e) => {
                    bot.answer_callback_query(q.id)
                        .text(format!("Invalid response: {}", e))
                        .await?;
                    return Ok(());
                }
            }
            
            // Update response in database
            let _response_record = match Response::upsert(
                &db.pool,
                session_id.to_string(),
                option_id.to_string(),
                user_id,
                username,
                response.to_string(),
            ).await {
                Ok(r) => r,
                Err(e) => {
                    bot.answer_callback_query(q.id)
                        .text("Failed to save response")
                        .await?;
                    tracing::error!("Failed to save response: {}", e);
                    return Ok(());
                }
            };
            
            // Get session and update the message
            match update_session_message(&bot, &db, session_id, &q).await {
                Ok(_) => {
                    let response_emoji = match response {
                        "yes" => "✅",
                        "no" => "❌", 
                        "maybe" => "❓",
                        _ => "👍"
                    };
                    bot.answer_callback_query(q.id)
                        .text(format!("{} Marked as {}", response_emoji, response))
                        .await?;
                },
                Err(e) => {
                    bot.answer_callback_query(q.id)
                        .text("Response saved but couldn't update message")
                        .await?;
                    tracing::error!("Failed to update message: {}", e);
                }
            }
        } else {
            bot.answer_callback_query(q.id)
                .text("Invalid callback data format")
                .await?;
        }
    }
    
    Ok(())
}

async fn update_session_message(
    bot: &Bot,
    db: &DatabaseManager,
    session_id: &str,
    q: &CallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get session details
    let session = Session::find_by_id(&db.pool, session_id)
        .await?
        .ok_or("Session not found")?;
    
    // Get session options
    let session_options = SessionOption::find_by_session(&db.pool, session_id).await?;
    
    // Get all responses for this session
    let responses = Response::find_by_session(&db.pool, session_id).await?;
    
    // Group responses by option_id
    let mut responses_by_option: HashMap<String, Vec<&Response>> = HashMap::new();
    for response in &responses {
        responses_by_option.entry(response.option_id.clone())
            .or_insert_with(Vec::new)
            .push(response);
    }
    
    // Build the updated message text
    let mut message_text = format!("🎲 **{}**\n\nSelect your availability for each option:\n\n", session.title);
    
    // Create inline keyboard
    let mut keyboard_rows = Vec::new();
    
    for (i, option) in session_options.iter().enumerate() {
        // Parse datetime and format it
        let datetime_str = chrono::DateTime::parse_from_rfc3339(&option.datetime)
            .map(|dt| format_datetime(&dt.with_timezone(&Utc)))
            .unwrap_or_else(|_| option.datetime.clone());
        
        message_text.push_str(&format!("**{}\\. {}**\n", i + 1, escape_markdown(&datetime_str)));
        
        // Count responses for this option
        let empty_vec = Vec::new();
        let option_responses = responses_by_option.get(&option.id).unwrap_or(&empty_vec);
        let yes_count = option_responses.iter().filter(|r| r.response == "yes").count();
        let no_count = option_responses.iter().filter(|r| r.response == "no").count();
        let maybe_count = option_responses.iter().filter(|r| r.response == "maybe").count();
        
        message_text.push_str(&format!(
            "✅ {} • ❌ {} • ❓ {}\n\n",
            yes_count, no_count, maybe_count
        ));
        
        // Add the inline keyboard row for this option
        let row = vec![
            InlineKeyboardButton::callback(
                format!("✅ {}", yes_count),
                format!("{}:{}:yes", session.id, option.id),
            ),
            InlineKeyboardButton::callback(
                format!("❌ {}", no_count), 
                format!("{}:{}:no", session.id, option.id),
            ),
            InlineKeyboardButton::callback(
                format!("❓ {}", maybe_count),
                format!("{}:{}:maybe", session.id, option.id),
            ),
        ];
        keyboard_rows.push(row);
    }
    
    let keyboard = InlineKeyboardMarkup::new(keyboard_rows);
    
    // Update the message if we have message info from the callback
    if let Some(message) = q.message.as_ref() {
        bot.edit_message_text(message.chat.id, message.id, message_text)
            .reply_markup(keyboard)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
    }
    
    Ok(())
}

// Helper function to escape markdown characters
pub 
async fn handle_settings_callback(
    bot: Bot,
    q: CallbackQuery,
    data: String,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    let setting = data.strip_prefix("settings:").unwrap_or(&data);
    
    match setting {
        "timezone" => {
            bot.answer_callback_query(q.id)
                .text("🕐 Timezone settings will be available in a future update!")
                .await?;
        }
        "duration" => {
            bot.answer_callback_query(q.id)
                .text("⏱️ Default duration settings will be available in a future update!")
                .await?;
        }
        "autoconfirm" => {
            bot.answer_callback_query(q.id)
                .text("🤖 Auto-confirm settings will be available in a future update!")
                .await?;
        }
        "stats" => {
            bot.answer_callback_query(q.id)
                .text("📊 Opening detailed statistics...")
                .await?;
            
            if let Some(message) = q.message {
                crate::bot::commands::stats::handle_stats(bot, message.clone(), db).await?;
            }
        }
        "close" => {
            bot.answer_callback_query(q.id)
                .text("Settings closed")
                .await?;
            
            if let Some(message) = q.message {
                bot.delete_message(message.chat.id, message.id).await?;
            }
        }
        _ => {
            bot.answer_callback_query(q.id)
                .text("Unknown setting")
                .await?;
        }
    }
    
    Ok(())
}
