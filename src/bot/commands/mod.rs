pub mod schedule;
pub mod session_management;
pub mod list;
pub mod settings;
pub mod stats;

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
    #[command(description = "Confirm a session and set it as final")]
    Confirm { session_id: String },
    #[command(description = "Cancel a session")]
    Cancel { session_id: String },
    #[command(description = "Set a deadline for responses", parse_with = "split")]
    Deadline { session_id: String, datetime: String },
    #[command(description = "List active sessions")]
    List,
    #[command(description = "Configure group settings")]
    Settings,
    #[command(description = "Show attendance statistics")]
    Stats,
}
