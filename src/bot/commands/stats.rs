use teloxide::prelude::*;
use crate::database::{connection::DatabaseManager, models::*};
use crate::utils::datetime::format_datetime;
use chrono::Utc;
use std::collections::HashMap;

pub async fn handle_stats(
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
    
    // Get detailed statistics
    let stats = match get_detailed_stats(&db.pool, group.id).await {
        Ok(stats) => stats,
        Err(e) => {
            tracing::error!("Failed to get stats: {}", e);
            bot.send_message(msg.chat.id, "❌ Error retrieving statistics.").await?;
            return Ok(());
        }
    };
    
    let mut message_text = String::from("📊 **Group Statistics**\n\n");
    
    // Session Statistics
    message_text.push_str(&format!(
        "🎲 **Sessions Overview:**\n\
        • Total Sessions: {}\n\
        • Active Sessions: {}\n\
        • Confirmed Sessions: {}\n\
        • Cancelled Sessions: {}\n\n",
        stats.total_sessions,
        stats.active_sessions,
        stats.confirmed_sessions,
        stats.cancelled_sessions
    ));
    
    // Response Statistics
    message_text.push_str(&format!(
        "📝 **Response Statistics:**\n\
        • Total Responses: {}\n\
        • Yes Responses: {} \\({:.1}%\\)\n\
        • No Responses: {} \\({:.1}%\\)\n\
        • Maybe Responses: {} \\({:.1}%\\)\n\n",
        stats.total_responses,
        stats.yes_responses,
        if stats.total_responses > 0 { stats.yes_responses as f64 / stats.total_responses as f64 * 100.0 } else { 0.0 },
        stats.no_responses,
        if stats.total_responses > 0 { stats.no_responses as f64 / stats.total_responses as f64 * 100.0 } else { 0.0 },
        stats.maybe_responses,
        if stats.total_responses > 0 { stats.maybe_responses as f64 / stats.total_responses as f64 * 100.0 } else { 0.0 }
    ));
    
    // User Participation
    if !stats.user_participation.is_empty() {
        message_text.push_str("👥 **Top Participants:**\n");
        let mut participants: Vec<_> = stats.user_participation.iter().collect();
        participants.sort_by(|a, b| b.1.cmp(a.1));
        
        for (i, (username, count)) in participants.iter().take(5).enumerate() {
            let medal = match i {
                0 => "🥇",
                1 => "🥈", 
                2 => "🥉",
                _ => "🏅"
            };
            let display_name = username.as_deref().unwrap_or("Anonymous");
            message_text.push_str(&format!("  {} {} \\({} responses\\)\n", medal, escape_markdown(display_name), count));
        }
        message_text.push('\n');
    }
    
    // Recent Activity
    if let Some(recent_session) = &stats.most_recent_session {
        let created_at = chrono::DateTime::parse_from_rfc3339(&recent_session.created_at)
            .map(|dt| format_datetime(&dt.with_timezone(&Utc)))
            .unwrap_or_else(|_| recent_session.created_at.clone());
        
        message_text.push_str(&format!(
            "🕐 **Recent Activity:**\n\
            • Last Session: {}\n\
            • Created: {}\n\
            • Status: {}\n\n",
            escape_markdown(&recent_session.title),
            escape_markdown(&created_at),
            match recent_session.status.as_str() {
                "active" => "🟢 Active",
                "confirmed" => "✅ Confirmed",
                "cancelled" => "❌ Cancelled",
                _ => "⚪ Unknown"
            }
        ));
    }
    
    // Footer
    message_text.push_str("💡 Use `/settings` for group configuration");
    
    bot.send_message(msg.chat.id, message_text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    
    Ok(())
}

struct DetailedStats {
    total_sessions: i32,
    active_sessions: i32,
    confirmed_sessions: i32,
    cancelled_sessions: i32,
    total_responses: i32,
    yes_responses: i32,
    no_responses: i32,
    maybe_responses: i32,
    user_participation: HashMap<Option<String>, i32>,
    most_recent_session: Option<Session>,
}

async fn get_detailed_stats(
    pool: &sqlx::SqlitePool,
    group_id: i64,
) -> Result<DetailedStats, sqlx::Error> {
    // Get session counts by status
    let session_counts = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total,
            COALESCE(SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END), 0) as active,
            COALESCE(SUM(CASE WHEN status = 'confirmed' THEN 1 ELSE 0 END), 0) as confirmed,
            COALESCE(SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END), 0) as cancelled
        FROM sessions 
        WHERE group_id = ?
        "#,
        group_id
    )
    .fetch_one(pool)
    .await?;
    
    // Get response counts by type
    let response_counts = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total,
            COALESCE(SUM(CASE WHEN r.response = 'yes' THEN 1 ELSE 0 END), 0) as yes_count,
            COALESCE(SUM(CASE WHEN r.response = 'no' THEN 1 ELSE 0 END), 0) as no_count,
            COALESCE(SUM(CASE WHEN r.response = 'maybe' THEN 1 ELSE 0 END), 0) as maybe_count
        FROM responses r
        JOIN sessions s ON r.session_id = s.id
        WHERE s.group_id = ?
        "#,
        group_id
    )
    .fetch_one(pool)
    .await?;
    
    // Get user participation
    let user_responses = sqlx::query!(
        r#"
        SELECT r.username, CAST(COUNT(*) AS INTEGER) as response_count
        FROM responses r
        JOIN sessions s ON r.session_id = s.id
        WHERE s.group_id = ?
        GROUP BY r.username
        "#,
        group_id
    )
    .fetch_all(pool)
    .await?;
    
    let mut user_participation = HashMap::new();
    for row in user_responses {
        user_participation.insert(row.username, row.response_count as i32);
    }
    
    // Get most recent session
    let most_recent_session = sqlx::query_as::<_, Session>(
        "SELECT id, group_id, title, message_id, status, deadline, created_by, created_at 
         FROM sessions 
         WHERE group_id = ? 
         ORDER BY created_at DESC 
         LIMIT 1"
    )
    .bind(group_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(DetailedStats {
        total_sessions: session_counts.total as i32,
        active_sessions: session_counts.active.unwrap_or(0) as i32,
        confirmed_sessions: session_counts.confirmed.unwrap_or(0) as i32,
        cancelled_sessions: session_counts.cancelled.unwrap_or(0) as i32,
        total_responses: response_counts.total as i32,
        yes_responses: response_counts.yes_count.unwrap_or(0) as i32,
        no_responses: response_counts.no_count.unwrap_or(0) as i32,
        maybe_responses: response_counts.maybe_count.unwrap_or(0) as i32,
        user_participation,
        most_recent_session,
    })
}

// Helper function to escape markdown characters
fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!")
}