use teloxide::prelude::*;
use crate::database::{connection::DatabaseManager, models::*};
use crate::utils::{datetime::format_datetime, markdown::escape_markdown};
use chrono::Utc;
use std::collections::HashMap;

pub async fn handle_list(
    bot: Bot,
    msg: Message,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    
    // Get the group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => group,
        Ok(None) => {
            bot.send_message(msg.chat.id, "❌ Group not found. Create a session first with /schedule.").await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find group: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving group information.").await?;
            return Ok(());
        }
    };
    
    // Get all active sessions for this group
    let sessions = match get_sessions_by_group(&db.pool, group.id).await {
        Ok(sessions) => sessions,
        Err(e) => {
            tracing::error!("Failed to get sessions: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving sessions.").await?;
            return Ok(());
        }
    };
    
    if sessions.is_empty() {
        bot.send_message(msg.chat.id, 
            "📋 **No Active Sessions**\n\nCreate a new session with /schedule"
        )
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
        return Ok(());
    }
    
    let mut message_text = String::from("📋 **Active Sessions**\n\n");
    
    // Batch fetch all session options and responses to avoid N+1 queries
    let session_ids: Vec<String> = sessions.iter().map(|s| s.id.clone()).collect();
    
    let all_options = match SessionOption::find_by_sessions(&db.pool, &session_ids).await {
        Ok(options) => options,
        Err(e) => {
            tracing::error!("Failed to batch fetch session options: {}", e);
            bot.send_message(msg.chat.id, "❌ Error loading session data").await?;
            return Ok(());
        }
    };
    
    let all_responses = match Response::find_by_sessions(&db.pool, &session_ids).await {
        Ok(responses) => responses,
        Err(e) => {
            tracing::warn!("Failed to batch fetch responses: {}", e);
            Vec::new()
        }
    };
    
    // Group options and responses by session ID for efficient lookup
    let mut options_by_session: HashMap<String, Vec<&SessionOption>> = HashMap::new();
    for option in &all_options {
        options_by_session.entry(option.session_id.clone()).or_default().push(option);
    }
    
    let mut responses_by_session: HashMap<String, Vec<&Response>> = HashMap::new();
    for response in &all_responses {
        responses_by_session.entry(response.session_id.clone()).or_default().push(response);
    }
    
    for session in sessions {
        // Get session options from pre-fetched data
        let empty_options = Vec::new();
        let options = options_by_session.get(&session.id).unwrap_or(&empty_options);
        
        // Get responses for vote counts from pre-fetched data
        let empty_responses = Vec::new();
        let responses = responses_by_session.get(&session.id).unwrap_or(&empty_responses);
        
        // Group responses by option
        let mut responses_by_option: HashMap<String, Vec<&Response>> = HashMap::new();
        for response in responses {
            responses_by_option.entry(response.option_id.clone())
                .or_default()
                .push(response);
        }
        
        // Add session info
        let status_emoji = match session.status.as_str() {
            "active" => "🟢",
            "confirmed" => "✅",
            "cancelled" => "❌",
            _ => "⚪"
        };
        
        message_text.push_str(&format!(
            "{} **{}**\n📧 ID: `{}`\n",
            status_emoji,
            escape_markdown(&session.title),
            session.id
        ));
        
        // Show deadline if set
        if let Some(deadline_str) = &session.deadline {
            if let Ok(deadline) = chrono::DateTime::parse_from_rfc3339(deadline_str) {
                let deadline_formatted = format_datetime(&deadline.with_timezone(&Utc));
                message_text.push_str(&format!("⏰ Deadline: {}\n", escape_markdown(&deadline_formatted)));
            }
        }
        
        // Show options and vote counts
        message_text.push_str("📅 **Options:**\n");
        for (i, option) in options.iter().enumerate() {
            let datetime_str = chrono::DateTime::parse_from_rfc3339(&option.datetime)
                .map(|dt| format_datetime(&dt.with_timezone(&Utc)))
                .unwrap_or_else(|_| option.datetime.clone());
            
            let empty_vec = Vec::new();
            let option_responses = responses_by_option.get(&option.id).unwrap_or(&empty_vec);
            let yes_count = option_responses.iter().filter(|r| r.response == "yes").count();
            let no_count = option_responses.iter().filter(|r| r.response == "no").count();
            let maybe_count = option_responses.iter().filter(|r| r.response == "maybe").count();
            
            let confirmed_marker = if option.confirmed { " ✅" } else { "" };
            
            message_text.push_str(&format!(
                "  {}\\. {} \\(✅ {} • ❌ {} • ❓ {}\\){}\n",
                i + 1,
                escape_markdown(&datetime_str),
                yes_count,
                no_count,
                maybe_count,
                confirmed_marker
            ));
        }
        
        message_text.push('\n');
    }
    
    // Add command usage hints
    message_text.push_str("💡 **Commands:**\n");
    message_text.push_str("• `/confirm <session_id>` \\- Confirm session\n");
    message_text.push_str("• `/cancel <session_id>` \\- Cancel session\n");
    message_text.push_str("• `/deadline <session_id> <time>` \\- Set deadline\n");
    
    bot.send_message(msg.chat.id, message_text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    
    Ok(())
}

// Database helper function
async fn get_sessions_by_group(
    pool: &sqlx::SqlitePool,
    group_id: i64,
) -> Result<Vec<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        "SELECT id, group_id, title, message_id, status, deadline, created_by, created_at 
         FROM sessions 
         WHERE group_id = ? AND status IN ('active', 'confirmed') 
         ORDER BY created_at DESC"
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
}

