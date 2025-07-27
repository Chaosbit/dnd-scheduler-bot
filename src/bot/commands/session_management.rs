use teloxide::prelude::*;
use crate::database::{connection::DatabaseManager, models::*};
use crate::utils::datetime::{parse_datetime, format_datetime};
use chrono::Utc;

pub async fn handle_confirm(
    bot: Bot,
    msg: Message,
    session_id: String,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let user_id = msg.from().map(|u| u.id.0 as i64).unwrap_or(0);
    
    // Validate session exists and belongs to this group
    let session = match Session::find_by_id(&db.pool, &session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            bot.send_message(msg.chat.id, "❌ Session not found.").await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find session: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving session.").await?;
            return Ok(());
        }
    };
    
    // Check if user is the session creator
    if session.created_by != user_id {
        bot.send_message(msg.chat.id, "❌ Only the session creator can confirm sessions.").await?;
        return Ok(());
    }
    
    // Check if session belongs to this group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => group,
        Ok(None) => {
            bot.send_message(msg.chat.id, "❌ Group not found.").await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find group: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving group information.").await?;
            return Ok(());
        }
    };
    
    if session.group_id != group.id {
        bot.send_message(msg.chat.id, "❌ Session doesn't belong to this group.").await?;
        return Ok(());
    }
    
    // Check if session is already confirmed or cancelled
    if session.status != "active" {
        bot.send_message(msg.chat.id, 
            format!("❌ Session is already {}.", session.status)
        ).await?;
        return Ok(());
    }
    
    // Get session options to find the most popular one
    let options = match SessionOption::find_by_session(&db.pool, &session_id).await {
        Ok(options) => options,
        Err(e) => {
            tracing::error!("Failed to get session options: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving session options.").await?;
            return Ok(());
        }
    };
    
    // Get all responses
    let responses = match Response::find_by_session(&db.pool, &session_id).await {
        Ok(responses) => responses,
        Err(e) => {
            tracing::error!("Failed to get responses: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving responses.").await?;
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
                bot.send_message(msg.chat.id, "❌ Error confirming session.").await?;
                return Ok(());
            }
            
            // Find the confirmed option for display
            let confirmed_option = match options.iter().find(|o| o.id == option_id) {
                Some(option) => option,
                None => {
                    tracing::error!("Confirmed option not found in session options");
                    bot.send_message(msg.chat.id, "❌ Error: Confirmed option not found.").await?;
                    return Ok(());
                }
            };
            let datetime_str = chrono::DateTime::parse_from_rfc3339(&confirmed_option.datetime)
                .map(|dt| format_datetime(&dt.with_timezone(&Utc)))
                .unwrap_or_else(|_| confirmed_option.datetime.clone());
            
            bot.send_message(msg.chat.id, 
                format!("✅ **Session Confirmed!**\n\n📅 **{}**\n🕐 {}\n👥 {} players confirmed", 
                    session.title, datetime_str, max_yes_votes)
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
        None => {
            bot.send_message(msg.chat.id, 
                "❌ Cannot confirm session: no option has any 'yes' votes."
            ).await?;
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
    
    // Validate session exists and belongs to this group
    let session = match Session::find_by_id(&db.pool, &session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            bot.send_message(msg.chat.id, "❌ Session not found.").await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find session: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving session.").await?;
            return Ok(());
        }
    };
    
    // Check if user is the session creator
    if session.created_by != user_id {
        bot.send_message(msg.chat.id, "❌ Only the session creator can cancel sessions.").await?;
        return Ok(());
    }
    
    // Check if session belongs to this group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => group,
        Ok(None) => {
            bot.send_message(msg.chat.id, "❌ Group not found.").await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find group: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving group information.").await?;
            return Ok(());
        }
    };
    
    if session.group_id != group.id {
        bot.send_message(msg.chat.id, "❌ Session doesn't belong to this group.").await?;
        return Ok(());
    }
    
    // Check if session is already cancelled
    if session.status == "cancelled" {
        bot.send_message(msg.chat.id, "❌ Session is already cancelled.").await?;
        return Ok(());
    }
    
    // Cancel the session
    if let Err(e) = cancel_session(&db.pool, &session_id).await {
        tracing::error!("Failed to cancel session: {}", e);
        bot.send_message(msg.chat.id, "❌ Error cancelling session.").await?;
        return Ok(());
    }
    
    bot.send_message(msg.chat.id, 
        format!("❌ **Session Cancelled**\n\n📅 **{}** has been cancelled.", session.title)
    )
    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
    .await?;
    
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
    
    // Validate session exists and belongs to this group
    let session = match Session::find_by_id(&db.pool, &session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            bot.send_message(msg.chat.id, "❌ Session not found.").await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find session: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving session.").await?;
            return Ok(());
        }
    };
    
    // Check if user is the session creator
    if session.created_by != user_id {
        bot.send_message(msg.chat.id, "❌ Only the session creator can set deadlines.").await?;
        return Ok(());
    }
    
    // Check if session belongs to this group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => group,
        Ok(None) => {
            bot.send_message(msg.chat.id, "❌ Group not found.").await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to find group: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving group information.").await?;
            return Ok(());
        }
    };
    
    if session.group_id != group.id {
        bot.send_message(msg.chat.id, "❌ Session doesn't belong to this group.").await?;
        return Ok(());
    }
    
    // Parse the deadline datetime
    let deadline_dt = match parse_datetime(&datetime) {
        Ok(dt) => dt,
        Err(_) => {
            bot.send_message(msg.chat.id, 
                format!("❌ Could not parse deadline: '{}'\n\nPlease use formats like:\n• Friday 19:00\n• Monday 14.30\n• Tuesday 20:00", datetime)
            ).await?;
            return Ok(());
        }
    };
    
    // Check if deadline is in the future
    if deadline_dt <= Utc::now() {
        bot.send_message(msg.chat.id, "❌ Deadline must be in the future.").await?;
        return Ok(());
    }
    
    // Set the deadline
    if let Err(e) = set_session_deadline(&db.pool, &session_id, &deadline_dt.to_rfc3339()).await {
        tracing::error!("Failed to set deadline: {}", e);
        bot.send_message(msg.chat.id, "❌ Error setting deadline.").await?;
        return Ok(());
    }
    
    let deadline_str = format_datetime(&deadline_dt);
    bot.send_message(msg.chat.id, 
        format!("⏰ **Deadline Set**\n\n📅 **{}**\n🕐 Responses due by: {}", 
            session.title, deadline_str)
    )
    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
    .await?;
    
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