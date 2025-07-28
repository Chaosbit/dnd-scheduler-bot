use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use crate::bot::commands::Command;
use crate::database::connection::DatabaseManager;
use crate::utils::feedback::CommandFeedback;

pub async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    db: DatabaseManager,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            let feedback = CommandFeedback::new(bot.clone(), msg.chat.id);
            let help_text = format!(
                "🎲 **D&D Scheduler Bot Commands**\n\n{}\n\n💡 **Quick Start:**\n• Use `/schedule \"Session Title\" \"Friday 19:00, Saturday 14:30\"` to create a poll\n• Players click buttons to vote\n• Use `/confirm <session_id>` to lock in the winning time\n\n📚 **Need more help?** Each command provides detailed error messages and suggestions when used incorrectly.",
                Command::descriptions()
            );
            feedback.info(&help_text).await?;
        }
        Command::Start => {
            let feedback = CommandFeedback::new(bot.clone(), msg.chat.id);
            let welcome_text = "Welcome to D&D Scheduler Bot!\n\nI help you schedule D&D sessions by creating polls where players can vote on their preferred times.\n\n🚀 **Get Started:**\n• Use /schedule to create your first session poll\n• Use /help to see all available commands\n\n🎯 **Pro Tip:** I provide detailed feedback and suggestions for every command!";
            feedback.success(welcome_text).await?;
        }
        Command::Schedule { title, options } => {
            crate::bot::commands::schedule::handle_schedule(bot, msg, title, options, &db).await?;
        }
        Command::Confirm { session_id } => {
            crate::bot::commands::session_management::handle_confirm(bot, msg, session_id, &db).await?;
        }
        Command::Cancel { session_id } => {
            crate::bot::commands::session_management::handle_cancel(bot, msg, session_id, &db).await?;
        }
        Command::Deadline { session_id, datetime } => {
            crate::bot::commands::session_management::handle_deadline(bot, msg, session_id, datetime, &db).await?;
        }
        Command::List => {
            crate::bot::commands::list::handle_list(bot, msg, &db).await?;
        }
        Command::TestReminders => {
            crate::bot::commands::reminders::handle_test_reminders(bot, msg, &db).await?;
        }
        Command::Settings => {
            crate::bot::commands::settings::handle_settings(bot, msg, &db).await?;
        }
        Command::Stats => {
            crate::bot::commands::stats::handle_stats(bot, msg, &db).await?;
        }
    }
    Ok(())
}
