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
        Command::Settings => {
            bot.send_message(msg.chat.id, "⚙️ Settings coming soon!").await?;
        }
        Command::Stats => {
            bot.send_message(msg.chat.id, "📊 Stats coming soon!").await?;
        }
    }
    Ok(())
}
