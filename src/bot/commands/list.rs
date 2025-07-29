use teloxide::prelude::*;
use crate::database::{connection::DatabaseManager, models::*};
use crate::utils::{datetime::format_datetime, markdown::escape_markdown, feedback::CommandFeedback};
use chrono::Utc;
use std::collections::HashMap;

pub async fn handle_list(
    bot: Bot,
    msg: Message,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let user_id = msg.from().map(|u| u.id.0).unwrap_or(0);
    let username = msg.from().and_then(|u| u.username.as_ref()).map_or("unknown", |v| v);
    
    tracing::info!(
        "List command initiated by user {} ({}) in chat {}",
        username, user_id, chat_id
    );
    
    let feedback = CommandFeedback::new(bot.clone(), msg.chat.id);
    
    // Send processing message
    let processing_msg = feedback.send_processing("Loading active sessions...").await?;
    
    // Get the group
    tracing::debug!("Looking up group for chat_id: {}", chat_id);
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => {
            tracing::debug!("Found group {} for chat {}", group.id, chat_id);
            group
        },
        Ok(None) => {
            tracing::info!("No group found for chat_id: {}", chat_id);
            let error_msg = "No sessions found for this group";
            let suggestion = "Create your first session with /schedule \"Session Title\" \"Friday 19:00, Saturday 14:30\"";
            feedback.validation_error(error_msg, suggestion).await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Database error finding group for chat_id {}: {}", chat_id, e);
            feedback.error("Failed to retrieve group information from database").await?;
            return Ok(());
        }
    };
    
    // Get all active sessions for this group with timeout
    tracing::debug!("Fetching sessions for group_id: {}", group.id);
    let sessions = match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        get_sessions_by_group(&db.pool, group.id)
    ).await {
        Ok(Ok(sessions)) => {
            tracing::info!(
                "Found {} sessions for group {} in chat {}",
                sessions.len(), group.id, chat_id
            );
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
                &format!("Found {} sessions, loading details...", sessions.len())).await?;
            sessions
        },
        Ok(Err(e)) => {
            tracing::error!("Database error fetching sessions for group {}: {}", group.id, e);
            feedback.error("Failed to retrieve sessions from database").await?;
            return Ok(());
        }
        Err(_) => {
            tracing::error!("Timeout fetching sessions for group {} after 10 seconds", group.id);
            feedback.error("Database query timeout - please try again").await?;
            return Ok(());
        }
    };
    
    if sessions.is_empty() {
        tracing::info!("No active sessions found for group {} in chat {}", group.id, chat_id);
        let info_message = "No active sessions found\\n\\n📋 This group doesn't have any active or confirmed sessions\\n\\n💡 Create your first session with:\\n`/schedule \"Session Title\" \"Friday 19:00, Saturday 14:30\"`";
        feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Info, info_message).await?;
        return Ok(());
    }
    
    let mut message_text = String::from("📋 **Active Sessions**\n\n");
    
    // Batch fetch all session options and responses to avoid N+1 queries
    let session_ids: Vec<String> = sessions.iter().map(|s| s.id.clone()).collect();
    
    // Add timeout and better error handling for batch operations
    tracing::debug!("Fetching session options for {} sessions: {:?}", session_ids.len(), session_ids);
    let all_options = match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        SessionOption::find_by_sessions(&db.pool, &session_ids)
    ).await {
        Ok(Ok(options)) => {
            tracing::debug!("Successfully fetched {} session options", options.len());
            options
        },
        Ok(Err(e)) => {
            tracing::error!("Database error batch fetching session options for sessions {:?}: {}", session_ids, e);
            feedback.error("Failed to load session time options from database").await?;
            return Ok(());
        }
        Err(_) => {
            tracing::error!("Timeout fetching session options for sessions {:?} after 10 seconds", session_ids);
            feedback.error("Database query timeout - please try again").await?;
            return Ok(());
        }
    };
    
    feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
        "Loading voting data...").await?;
    
    tracing::debug!("Fetching responses for {} sessions: {:?}", session_ids.len(), session_ids);
    let all_responses = match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        Response::find_by_sessions(&db.pool, &session_ids)
    ).await {
        Ok(Ok(responses)) => {
            tracing::debug!("Successfully fetched {} responses", responses.len());
            responses
        },
        Ok(Err(e)) => {
            tracing::warn!("Database error batch fetching responses for sessions {:?}: {}", session_ids, e);
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
                "Generating session list (voting data unavailable)...").await?;
            Vec::new()
        }
        Err(_) => {
            tracing::warn!("Timeout fetching responses for sessions {:?} after 10 seconds - continuing without voting data", session_ids);
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
                "Generating session list (voting data timeout)...").await?;
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
    
    // Send the complete session list
    feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Success, &message_text).await?;
    
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

