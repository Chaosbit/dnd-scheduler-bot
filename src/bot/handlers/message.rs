use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use crate::bot::commands::Command;
use crate::database::connection::DatabaseManager;

pub async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    db: DatabaseManager,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Start => {
            bot.send_message(
                msg.chat.id,
                "🎲 Welcome to D&D Scheduler Bot!\n\nUse /schedule to create a new session poll.\nUse /help to see all commands.",
            ).await?;
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
        Command::Settings => {
            crate::bot::commands::settings::handle_settings(bot, msg, &db).await?;
        }
        Command::Stats => {
            crate::bot::commands::stats::handle_stats(bot, msg, &db).await?;
        }
    }
    Ok(())
}
