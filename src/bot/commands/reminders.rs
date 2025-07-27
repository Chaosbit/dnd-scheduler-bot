use teloxide::prelude::*;
use crate::database::connection::DatabaseManager;
use crate::services::reminder::ReminderService;
use std::sync::Arc;

pub async fn handle_test_reminders(
    bot: Bot,
    msg: Message,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    // For security, only allow certain users to test reminders
    // In a real implementation, you might check if user is group admin
    
    bot.send_message(msg.chat.id, "🔄 **Testing Reminder System**\n\nChecking for sessions that need reminders...")
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    
    // Create a temporary reminder service to test
    let db_arc = Arc::new(db.clone());
    let reminder_service = match ReminderService::new(bot.clone(), db_arc).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to create reminder service for testing: {}", e);
            bot.send_message(msg.chat.id, "❌ **Error Creating Reminder Service**\n\nCould not initialize reminder service for testing\\.")
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        }
    };
    
    match reminder_service.check_reminders_now().await {
        Ok(_) => {
            bot.send_message(msg.chat.id, "✅ **Reminder Check Complete**\n\nReminder system tested successfully\\. Any due reminders have been sent\\.")
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
        Err(e) => {
            tracing::error!("Manual reminder check failed: {}", e);
            bot.send_message(msg.chat.id, "❌ **Reminder Check Failed**\n\nThere was an error checking for reminders\\. Check the logs for details\\.")
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
    }
    
    Ok(())
}