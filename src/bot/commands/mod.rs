pub mod schedule;
pub mod session_management;
pub mod list;
pub mod settings;
pub mod stats;
pub mod reminders;

use teloxide::utils::command::BotCommands;

fn parse_schedule_args(input: String) -> Result<(String, String), teloxide::utils::command::ParseError> {
    let input = input.trim();
    
    if input.is_empty() {
        return Err(teloxide::utils::command::ParseError::IncorrectFormat("Expected: /schedule Title Time options".into()));
    }
    
    // Handle case where first argument is quoted
    if input.starts_with('"') {
        // Find the closing quote for the first argument
        if let Some(closing_quote_pos) = input[1..].find('"') {
            let title = input[1..closing_quote_pos + 1].to_string();
            let rest = input[closing_quote_pos + 2..].trim();
            
            // Allow empty options for quoted title case
            if rest.is_empty() {
                return Ok((title, String::new()));
            }
            
            // Check if the rest is also quoted
            if rest.starts_with('"') && rest.ends_with('"') && rest.len() > 1 {
                let options = rest[1..rest.len()-1].to_string();
                Ok((title, options))
            } else {
                // Rest is not quoted, use as-is
                Ok((title, rest.to_string()))
            }
        } else {
            // Unclosed quote - treat everything after quote as title (graceful handling)
            let title = input[1..].to_string();
            Ok((title, String::new()))
        }
    } else {
        // Handle unquoted arguments: /schedule Title Rest of the options
        match input.split_once(' ') {
            Some((title, options)) => {
                let title = title.trim();
                let options = options.trim();
                
                if title.is_empty() {
                    return Err(teloxide::utils::command::ParseError::IncorrectFormat("Title cannot be empty".into()));
                }
                
                // If options are quoted, remove the quotes
                if options.starts_with('"') && options.ends_with('"') && options.len() > 1 {
                    Ok((title.to_string(), options[1..options.len()-1].to_string()))
                } else {
                    Ok((title.to_string(), options.to_string()))
                }
            },
            None => {
                // Only one argument provided, use as title with empty options
                Ok((input.to_string(), String::new()))
            }
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
#[command(description = "D&D Scheduler Bot commands:")]
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
