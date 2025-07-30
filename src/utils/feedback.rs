use teloxide::prelude::*;
use teloxide::types::{ParseMode, MessageId};
use crate::utils::markdown::escape_markdown;

/// Feedback types for different command outcomes
#[derive(Debug, Clone)]
pub enum FeedbackType {
    Success,
    Warning,
    Error,
    Info,
    Processing,
}

impl FeedbackType {
    fn emoji(&self) -> &'static str {
        match self {
            FeedbackType::Success => "✅",
            FeedbackType::Warning => "⚠️", 
            FeedbackType::Error => "❌",
            FeedbackType::Info => "ℹ️",
            FeedbackType::Processing => "⏳",
        }
    }
}

/// Centralized feedback system for bot commands
pub struct CommandFeedback {
    bot: Bot,
    chat_id: ChatId,
}

impl CommandFeedback {
    pub fn new(bot: Bot, chat_id: ChatId) -> Self {
        Self { bot, chat_id }
    }

    /// Send immediate feedback message
    pub async fn send(&self, feedback_type: FeedbackType, message: &str) -> ResponseResult<Message> {
        let formatted_message = format!("{} {}", feedback_type.emoji(), escape_markdown(message));
        
        self.bot
            .send_message(self.chat_id, formatted_message)
            .parse_mode(ParseMode::MarkdownV2)
            .await
    }

    /// Send a processing message that can be updated later
    pub async fn send_processing(&self, message: &str) -> ResponseResult<Message> {
        self.send(FeedbackType::Processing, message).await
    }

    /// Update an existing message with new feedback
    pub async fn update_message(
        &self, 
        message_id: MessageId, 
        feedback_type: FeedbackType, 
        message: &str
    ) -> ResponseResult<Message> {
        let formatted_message = format!("{} {}", feedback_type.emoji(), escape_markdown(message));
        
        self.bot
            .edit_message_text(self.chat_id, message_id, formatted_message)
            .parse_mode(ParseMode::MarkdownV2)
            .await
    }

    /// Send success feedback
    pub async fn success(&self, message: &str) -> ResponseResult<Message> {
        self.send(FeedbackType::Success, message).await
    }

    /// Send error feedback
    pub async fn error(&self, message: &str) -> ResponseResult<Message> {
        self.send(FeedbackType::Error, message).await
    }

    /// Send warning feedback
    pub async fn warning(&self, message: &str) -> ResponseResult<Message> {
        self.send(FeedbackType::Warning, message).await
    }

    /// Send info feedback
    pub async fn info(&self, message: &str) -> ResponseResult<Message> {
        self.send(FeedbackType::Info, message).await
    }

    /// Send detailed command help with formatting
    #[allow(dead_code)]
    pub async fn send_command_help(&self, command: &str, description: &str, examples: &[&str]) -> ResponseResult<Message> {
        let mut help_text = format!("**{}**\n\n{}\n\n", escape_markdown(command), escape_markdown(description));
        
        if !examples.is_empty() {
            help_text.push_str("**Examples:**\n");
            for example in examples {
                help_text.push_str(&format!("• `{}`\n", escape_markdown(example)));
            }
        }
        
        self.bot
            .send_message(self.chat_id, help_text)
            .parse_mode(ParseMode::MarkdownV2)
            .await
    }

    /// Send validation error with helpful suggestion
    pub async fn validation_error(&self, error: &str, suggestion: &str) -> ResponseResult<Message> {
        let message = format!("{error}\n\n💡 **Suggestion:** {suggestion}");
        self.send(FeedbackType::Error, &message).await
    }
}

/// Progress tracker for multi-step operations
pub struct ProgressTracker {
    feedback: CommandFeedback,
    message_id: Option<MessageId>,
    total_steps: u32,
    current_step: u32,
}

impl ProgressTracker {
    pub fn new(feedback: CommandFeedback, total_steps: u32) -> Self {
        Self {
            feedback,
            message_id: None,
            total_steps,
            current_step: 0,
        }
    }

    /// Start progress tracking
    pub async fn start(&mut self, initial_message: &str) -> ResponseResult<()> {
        let progress_message = format!("{} (1/{})", initial_message, self.total_steps);
        let message = self.feedback.send_processing(&progress_message).await?;
        self.message_id = Some(message.id);
        self.current_step = 1;
        Ok(())
    }

    /// Update progress to next step
    pub async fn next_step(&mut self, step_message: &str) -> ResponseResult<()> {
        if let Some(message_id) = self.message_id {
            self.current_step += 1;
            let progress_message = format!("{} ({}/{})", step_message, self.current_step, self.total_steps);
            self.feedback.update_message(message_id, FeedbackType::Processing, &progress_message).await?;
        }
        Ok(())
    }

    /// Complete progress tracking with success message
    pub async fn complete(&mut self, completion_message: &str) -> ResponseResult<()> {
        if let Some(message_id) = self.message_id {
            self.feedback.update_message(message_id, FeedbackType::Success, completion_message).await?;
        }
        Ok(())
    }

    /// Complete progress tracking with error message
    pub async fn error(&mut self, error_message: &str) -> ResponseResult<()> {
        if let Some(message_id) = self.message_id {
            self.feedback.update_message(message_id, FeedbackType::Error, error_message).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_type_emojis() {
        assert_eq!(FeedbackType::Success.emoji(), "✅");
        assert_eq!(FeedbackType::Warning.emoji(), "⚠️");
        assert_eq!(FeedbackType::Error.emoji(), "❌");
        assert_eq!(FeedbackType::Info.emoji(), "ℹ️");
        assert_eq!(FeedbackType::Processing.emoji(), "⏳");
    }
}