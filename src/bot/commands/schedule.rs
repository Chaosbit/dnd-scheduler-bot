use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::database::{connection::DatabaseManager, models::*};
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
    
    for _option_str in &option_strings {
        // TODO: Parse actual dates, for now use dummy dates
        let datetime = Utc::now() + chrono::Duration::days(1);
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
    
    let message_text = format!(
        "🎲 **{}**\n\nSelect your availability for each option:\n{}",
        title,
        option_strings.iter().enumerate()
            .map(|(i, opt)| format!("{}. {}", i + 1, opt))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    bot.send_message(msg.chat.id, message_text)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    
    Ok(())
}
