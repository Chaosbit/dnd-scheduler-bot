use teloxide::prelude::*;
use crate::database::{connection::DatabaseManager, models::*};
use crate::utils::{
    datetime::{parse_datetime, format_datetime},
    feedback::CommandFeedback
};
use chrono::Utc;

pub async fn handle_confirm(
    bot: Bot,
    msg: Message,
    session_id: String,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let user_id = msg.from().map(|u| u.id.0 as i64).unwrap_or(0);
    let feedback = CommandFeedback::new(bot.clone(), msg.chat.id);
    
    // Send processing message
    let processing_msg = feedback.send_processing("Confirming session...").await?;
    
    // Validate session exists and belongs to this group
    let session = match Session::find_by_id(&db.pool, &session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            let error_msg = "Session not found";
            let suggestion = format!("Please check the session ID. Use /list to see active sessions.");
            feedback.validation_error(error_msg, &suggestion).await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find session: {}", e);
            feedback.error("Failed to retrieve session information from database").await?;
            return Ok(());
        }
    };
    
    // Check if user is the session creator
    if session.created_by != user_id {
        let error_msg = "Permission denied: Only the session creator can confirm sessions";
        let suggestion = "Ask the session creator to run this command, or use /list to see who created each session.";
        feedback.validation_error(error_msg, suggestion).await?;
        return Ok(());
    }
    
    // Check if session belongs to this group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => group,
        Ok(None) => {
            feedback.error("Group not found in database").await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find group: {}", e);
            feedback.error("Failed to retrieve group information").await?;
            return Ok(());
        }
    };
    
    if session.group_id != group.id {
        let error_msg = "Session doesn't belong to this group";
        let suggestion = "This session was created in a different group. Use /list to see sessions for this group.";
        feedback.validation_error(error_msg, suggestion).await?;
        return Ok(());
    }
    
    // Check if session is already confirmed or cancelled
    if session.status != "active" {
        let error_msg = format!("Session is already {}", session.status);
        let suggestion = match session.status.as_str() {
            "confirmed" => "This session has already been confirmed. Use /list to see current status.",
            "cancelled" => "This session has been cancelled. Create a new session with /schedule if needed.",
            _ => "This session is not in active status. Use /list to check current status."
        };
        feedback.validation_error(&error_msg, suggestion).await?;
        return Ok(());
    }
    
    // Get session options to find the most popular one
    let options = match SessionOption::find_by_session(&db.pool, &session_id).await {
        Ok(options) => {
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
                &format!("Analyzing {} time options...", options.len())).await?;
            options
        },
        Err(e) => {
            tracing::error!("Failed to get session options: {}", e);
            feedback.error("Failed to retrieve session time options").await?;
            return Ok(());
        }
    };
    
    // Get all responses
    let responses = match Response::find_by_session(&db.pool, &session_id).await {
        Ok(responses) => {
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
                &format!("Processing {} player responses...", responses.len())).await?;
            responses
        },
        Err(e) => {
            tracing::error!("Failed to get responses: {}", e);
            feedback.error("Failed to retrieve player responses").await?;
            return Ok(());
        }
    };
    
    // Find the option with the most "yes" votes
    let mut best_option_id = None;
    let mut max_yes_votes = 0;
    
    for option in &options {
        let yes_count = responses.iter()
            .filter(|r| r.option_id == option.id && r.response == "yes")
            .count();
        
        if yes_count > max_yes_votes {
            max_yes_votes = yes_count;
            best_option_id = Some(option.id.clone());
        }
    }
    
    match best_option_id {
        Some(option_id) => {
            // Mark the winning option as confirmed and update session status
            if let Err(e) = confirm_session_and_option(&db.pool, &session_id, &option_id).await {
                tracing::error!("Failed to confirm session: {}", e);
                feedback.error("Failed to save session confirmation to database").await?;
                return Ok(());
            }
            
            // Find the confirmed option for display
            let confirmed_option = match options.iter().find(|o| o.id == option_id) {
                Some(option) => option,
                None => {
                    tracing::error!("Confirmed option not found in session options");
                    feedback.error("Internal error: Confirmed option not found").await?;
                    return Ok(());
                }
            };
            let datetime_str = chrono::DateTime::parse_from_rfc3339(&confirmed_option.datetime)
                .map(|dt| format_datetime(&dt.with_timezone(&Utc)))
                .unwrap_or_else(|_| confirmed_option.datetime.clone());
            
            // Send detailed success message
            let success_message = format!(
                "Session '{}' confirmed successfully!\n\n📅 Confirmed Time: {}\n👥 {} players will attend\n\n🎯 All participants have been notified. The session is now locked in!",
                session.title, 
                datetime_str, 
                max_yes_votes
            );
            
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Success, &success_message).await?;
        }
        None => {
            let error_msg = "Cannot confirm session: No time options have 'yes' votes";
            let suggestion = "Ask players to vote on the available time options first. Use /list to see current voting status.";
            feedback.validation_error(error_msg, suggestion).await?;
        }
    }
    
    Ok(())
}

pub async fn handle_cancel(
    bot: Bot,
    msg: Message,
    session_id: String,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let user_id = msg.from().map(|u| u.id.0 as i64).unwrap_or(0);
    let feedback = CommandFeedback::new(bot.clone(), msg.chat.id);
    
    // Send processing message
    let processing_msg = feedback.send_processing("Cancelling session...").await?;
    
    // Validate session exists and belongs to this group
    let session = match Session::find_by_id(&db.pool, &session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            let error_msg = "Session not found";
            let suggestion = "Please check the session ID. Use /list to see active sessions.";
            feedback.validation_error(error_msg, suggestion).await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find session: {}", e);
            feedback.error("Failed to retrieve session information from database").await?;
            return Ok(());
        }
    };
    
    // Check if user is the session creator
    if session.created_by != user_id {
        let error_msg = "Permission denied: Only the session creator can cancel sessions";
        let suggestion = "Ask the session creator to run this command, or use /list to see who created each session.";
        feedback.validation_error(error_msg, suggestion).await?;
        return Ok(());
    }
    
    // Check if session belongs to this group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => group,
        Ok(None) => {
            feedback.error("Group not found in database").await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find group: {}", e);
            feedback.error("Failed to retrieve group information").await?;
            return Ok(());
        }
    };
    
    if session.group_id != group.id {
        let error_msg = "Session doesn't belong to this group";
        let suggestion = "This session was created in a different group. Use /list to see sessions for this group.";
        feedback.validation_error(error_msg, suggestion).await?;
        return Ok(());
    }
    
    // Check if session is already cancelled
    if session.status == "cancelled" {
        let error_msg = "Session is already cancelled";
        let suggestion = "This session has already been cancelled. Use /list to see current status of all sessions.";
        feedback.validation_error(error_msg, suggestion).await?;
        return Ok(());
    }
    
    // Check if session is confirmed (warn but allow cancellation)
    if session.status == "confirmed" {
        feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
            "Warning: Cancelling a confirmed session...").await?;
    }
    
    // Cancel the session
    if let Err(e) = cancel_session(&db.pool, &session_id).await {
        tracing::error!("Failed to cancel session: {}", e);
        feedback.error("Failed to save session cancellation to database").await?;
        return Ok(());
    }
    
    // Send detailed success message
    let success_message = format!(
        "Session '{}' cancelled successfully!\\n\\n📅 The session has been removed from the schedule\\n👥 All participants have been notified\\n\\n💡 Use /schedule to create a new session if needed",
        session.title
    );
    
    feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Success, &success_message).await?;
    
    Ok(())
}

pub async fn handle_deadline(
    bot: Bot,
    msg: Message,
    session_id: String,
    datetime: String,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let user_id = msg.from().map(|u| u.id.0 as i64).unwrap_or(0);
    let feedback = CommandFeedback::new(bot.clone(), msg.chat.id);
    
    // Send processing message
    let processing_msg = feedback.send_processing("Setting session deadline...").await?;
    
    // Validate session exists and belongs to this group
    let session = match Session::find_by_id(&db.pool, &session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            let error_msg = "Session not found";
            let suggestion = "Please check the session ID. Use /list to see active sessions.";
            feedback.validation_error(error_msg, suggestion).await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find session: {}", e);
            feedback.error("Failed to retrieve session information from database").await?;
            return Ok(());
        }
    };
    
    // Check if user is the session creator
    if session.created_by != user_id {
        let error_msg = "Permission denied: Only the session creator can set deadlines";
        let suggestion = "Ask the session creator to run this command, or use /list to see who created each session.";
        feedback.validation_error(error_msg, suggestion).await?;
        return Ok(());
    }
    
    // Check if session belongs to this group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => group,
        Ok(None) => {
            feedback.error("Group not found in database").await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find group: {}", e);
            feedback.error("Failed to retrieve group information").await?;
            return Ok(());
        }
    };
    
    if session.group_id != group.id {
        let error_msg = "Session doesn't belong to this group";
        let suggestion = "This session was created in a different group. Use /list to see sessions for this group.";
        feedback.validation_error(error_msg, suggestion).await?;
        return Ok(());
    }
    
    // Parse the deadline datetime
    let deadline_dt = match parse_datetime(&datetime) {
        Ok(dt) => {
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
                "Validating deadline datetime...").await?;
            dt
        },
        Err(_) => {
            let error_msg = format!("Could not parse deadline: '{}'", datetime);
            let suggestion = "Please use formats like 'Friday 19:00', 'Monday 14:30', or 'Tuesday 20:00'";
            feedback.validation_error(&error_msg, suggestion).await?;
            return Ok(());
        }
    };
    
    // Check if deadline is in the future
    if deadline_dt <= Utc::now() {
        let error_msg = "Deadline must be in the future";
        let suggestion = "Please specify a date and time that hasn't passed yet. Use formats like 'Friday 19:00' or 'Monday 14:30'.";
        feedback.validation_error(error_msg, suggestion).await?;
        return Ok(());
    }
    
    // Set the deadline
    if let Err(e) = set_session_deadline(&db.pool, &session_id, &deadline_dt.to_rfc3339()).await {
        tracing::error!("Failed to set deadline: {}", e);
        feedback.error("Failed to save deadline to database").await?;
        return Ok(());
    }
    
    // Send detailed success message
    let deadline_str = format_datetime(&deadline_dt);
    let success_message = format!(
        "Session deadline set successfully!\\n\\n📅 Session: {}\\n⏰ Responses due by: {}\\n\\n💡 Players will be reminded before the deadline\\n👥 Use /list to check current voting status",
        session.title, 
        deadline_str
    );
    
    feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Success, &success_message).await?;
    
    Ok(())
}

// Database helper functions
async fn confirm_session_and_option(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    option_id: &str,
) -> Result<(), sqlx::Error> {
    // Start a transaction
    let mut tx = pool.begin().await?;
    
    // Update session status to confirmed
    sqlx::query!(
        "UPDATE sessions SET status = 'confirmed' WHERE id = ?",
        session_id
    )
    .execute(&mut *tx)
    .await?;
    
    // Mark the winning option as confirmed
    sqlx::query!(
        "UPDATE session_options SET confirmed = true WHERE id = ?",
        option_id
    )
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    Ok(())
}

async fn cancel_session(
    pool: &sqlx::SqlitePool,
    session_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE sessions SET status = 'cancelled' WHERE id = ?",
        session_id
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

async fn set_session_deadline(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    deadline: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE sessions SET deadline = ? WHERE id = ?",
        deadline,
        session_id
    )
    .execute(pool)
    .await?;
    
    Ok(())
}