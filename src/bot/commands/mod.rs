pub mod schedule;

use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "D&D Scheduler Bot commands:")]
pub enum Command {
    #[command(description = "Display this help message")]
    Help,
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Create a new session poll", parse_with = "split")]
    Schedule { title: String, options: String },
    #[command(description = "Configure group settings")]
    Settings,
    #[command(description = "Show attendance statistics")]
    Stats,
}
