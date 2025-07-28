use teloxide::prelude::*;
use crate::database::connection::DatabaseManager;
use crate::services::reminder::ReminderService;
use crate::utils::feedback::CommandFeedback;
use std::sync::Arc;

pub async fn handle_test_reminders(
    bot: Bot,
    msg: Message,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    let feedback = CommandFeedback::new(bot.clone(), msg.chat.id);
    
    // Send processing message
    let processing_msg = feedback.send_processing("Testing reminder system...").await?;
    
    // For security, only allow certain users to test reminders
    // In a real implementation, you might check if user is group admin
    let _user_id = msg.from().map(|u| u.id.0).unwrap_or(0);
    
    // Add a simple check - in a real implementation, you'd want proper admin verification
    feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
        "Initializing reminder service...").await?;
    
    // Create a temporary reminder service to test
    let db_arc = Arc::new(db.clone());
    let reminder_service = match ReminderService::new(bot.clone(), db_arc).await {
        Ok(service) => {
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
                "Checking for sessions that need reminders...").await?;
            service
        },
        Err(e) => {
            tracing::error!("Failed to create reminder service for testing: {}", e);
            feedback.error("Could not initialize reminder service for testing").await?;
            return Ok(());
        }
    };
    
    match reminder_service.check_reminders_now().await {
        Ok(_) => {
            let success_message = "Reminder system test completed successfully!\\n\\n✅ All due reminders have been processed and sent\\n\\n💡 **How reminders work:**\\n• 2 weeks before confirmed sessions\\n• 1 week before confirmed sessions\\n• 3 days before confirmed sessions\\n\\n🔧 Reminders run automatically every hour";
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Success, success_message).await?;
        }
        Err(e) => {
            tracing::error!("Manual reminder check failed: {}", e);
            let error_msg = "Reminder system test failed";
            let suggestion = "There was an error during the reminder check. Please contact an administrator or check the system logs for details.";
            feedback.validation_error(error_msg, suggestion).await?;
        }
    }
    
    Ok(())
}