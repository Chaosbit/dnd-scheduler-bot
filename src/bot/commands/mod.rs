pub mod schedule;
pub mod session_management;
pub mod list;
pub mod settings;
pub mod stats;
pub mod reminders;

use teloxide::utils::command::BotCommands;

fn parse_schedule_args(input: String) -> Result<(String, String), teloxide::utils::command::ParseError> {
    // Handle quoted arguments: /schedule "Title with spaces" "Time options"
    if input.starts_with('"') {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                    if !in_quotes && !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                ' ' if !in_quotes => {
                    // Skip spaces outside quotes
                    continue;
                }
                _ => {
                    current.push(ch);
                }
            }
        }
        
        // Add final part if we ended inside quotes
        if !current.is_empty() {
            parts.push(current);
        }
        
        if parts.len() >= 2 {
            Ok((parts[0].clone(), parts[1..].join(" ")))
        } else if parts.len() == 1 {
            // Only one quoted argument, use the rest as second argument
            let remaining = input.split_once('"').and_then(|(_, rest)| rest.split_once('"'))
                .map(|(_, rest)| rest.trim().to_string())
                .unwrap_or_default();
            Ok((parts[0].clone(), remaining))
        } else {
            Err(teloxide::utils::command::ParseError::IncorrectFormat("Expected: /schedule \"Title\" \"Time options\"".into()))
        }
    } else {
        // Handle unquoted arguments: /schedule Title Rest of the options
        match input.split_once(' ') {
            Some((title, options)) => Ok((title.to_string(), options.to_string())),
            None => Err(teloxide::utils::command::ParseError::IncorrectFormat("Expected: /schedule Title Time options".into())),
        }
    }
}

fn parse_deadline_args(input: String) -> Result<(String, String), teloxide::utils::command::ParseError> {
    match input.split_once(' ') {
        Some((session_id, datetime)) => Ok((session_id.to_string(), datetime.to_string())),
        None => Err(teloxide::utils::command::ParseError::IncorrectFormat("Expected: /deadline <session_id> <datetime>".into())),
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "D&D Scheduler Bot commands:")]
pub enum Command {
    #[command(description = "Display this help message")]
    Help,
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Create a new session poll", parse_with = parse_schedule_args)]
    Schedule { title: String, options: String },
    #[command(description = "Confirm a session and set it as final")]
    Confirm { session_id: String },
    #[command(description = "Cancel a session")]
    Cancel { session_id: String },
    #[command(description = "Set a deadline for responses", parse_with = parse_deadline_args)]
    Deadline { session_id: String, datetime: String },
    #[command(description = "List active sessions")]
    List,
    #[command(description = "Test reminder system (admin only)")]
    TestReminders,
    #[command(description = "Configure group settings")]
    Settings,
    #[command(description = "Show attendance statistics")]
    Stats,
}
