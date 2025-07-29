use teloxide::prelude::*;
use crate::utils::feedback::CommandFeedback;

pub async fn handle_general_message(
    bot: Bot,
    msg: Message,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let user_id = msg.from().map(|u| u.id.0).unwrap_or(0);
    let username = msg.from().and_then(|u| u.username.as_ref()).map_or("unknown", |v| v);
    let feedback = CommandFeedback::new(bot.clone(), msg.chat.id);
    
    if let Some(text) = msg.text() {
        tracing::debug!(
            "General message from user {} ({}) in chat {}: '{}'",
            username, user_id, chat_id, text
        );
        
        // Check if it looks like a malformed command
        if text.starts_with('/') {
            tracing::info!(
                "Unknown command '{}' from user {} ({}) in chat {}",
                text, username, user_id, chat_id
            );
            let error_msg = format!("Unknown command: {}", text.split_whitespace().next().unwrap_or(text));
            let suggestion = "Use /help to see all available commands, or check your command syntax.";
            feedback.validation_error(&error_msg, suggestion).await?;
        } else if text.to_lowercase().contains("schedule") || text.to_lowercase().contains("session") {
            // Helpful hint for users trying to schedule
            let suggestion = "Looking to schedule a session? Try:\n• `/schedule \"Session Title\" \"Friday 19:00, Saturday 14:30\"`\n• Use /help for more examples";
            feedback.info(suggestion).await?;
        } else if text.to_lowercase().contains("help") {
            // Direct them to help
            feedback.info("Use /help to see all available commands and examples!").await?;
        }
        // For other messages, we don't respond to avoid spam
    }
    
    Ok(())
}