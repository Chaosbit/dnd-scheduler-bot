use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::database::{connection::DatabaseManager, models::*};
use crate::utils::{validation::validate_telegram_chat_id, feedback::CommandFeedback};

pub async fn handle_settings(
    bot: Bot,
    msg: Message,
    db: &DatabaseManager,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let _user_id = msg.from().map(|u| u.id.0 as i64).unwrap_or(0);
    let feedback = CommandFeedback::new(bot.clone(), msg.chat.id);
    
    // Send processing message
    let processing_msg = feedback.send_processing("Loading group settings...").await?;
    
    // Validate chat ID
    if let Err(e) = validate_telegram_chat_id(chat_id) {
        let error_msg = format!("Invalid chat configuration: {e}");
        let suggestion = "This command can only be used in properly configured chat groups.";
        feedback.validation_error(&error_msg, suggestion).await?;
        return Ok(());
    }
    
    // Check if user is admin (for now, anyone can access settings)
    // In a real implementation, you might want to check if user is a group admin
    
    // Get or create group
    let group = match Group::find_by_chat_id(&db.pool, chat_id).await {
        Ok(Some(group)) => {
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
                "Loading group statistics...").await?;
            group
        },
        Ok(None) => {
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
                "Setting up new group...").await?;
            // Create group if it doesn't exist
            match Group::create(&db.pool, chat_id).await {
                Ok(group) => group,
                Err(e) => {
                    tracing::error!("Failed to create group: {}", e);
                    feedback.error("Failed to initialize group settings in database").await?;
                    return Ok(());
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to find group: {}", e);
            feedback.error("Failed to access group information from database").await?;
            return Ok(());
        }
    };
    
    // Get group statistics
    let stats = match get_group_stats(&db.pool, group.id).await {
        Ok(stats) => stats,
        Err(e) => {
            tracing::warn!("Failed to get group stats: {}", e);
            feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Processing, 
                "Loading settings (statistics unavailable)...").await?;
            GroupStats::default()
        }
    };
    
    let message_text = format!(
        "⚙️ **Group Settings**\n\n\
        📊 **Group Statistics:**\n\
        • Total Sessions: {}\n\
        • Active Sessions: {}\n\
        • Confirmed Sessions: {}\n\
        • Total Responses: {}\n\n\
        🔧 **Available Settings:**\n\
        • Timezone: UTC \\(coming soon\\)\n\
        • Default Duration: 4 hours \\(coming soon\\)\n\
        • Auto\\-confirm: Disabled \\(coming soon\\)\n\n\
        💡 **Tips:**\n\
        • Use `/list` to see all active sessions\n\
        • Session creators can use `/confirm` and `/cancel`\n\
        • Set deadlines with `/deadline <session_id> <time>`",
        stats.total_sessions,
        stats.active_sessions,
        stats.confirmed_sessions,
        stats.total_responses
    );
    
    // Create inline keyboard for future settings
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🕐 Timezone Settings", "settings:timezone"),
            InlineKeyboardButton::callback("⏱️ Default Duration", "settings:duration"),
        ],
        vec![
            InlineKeyboardButton::callback("🤖 Auto-confirm", "settings:autoconfirm"),
            InlineKeyboardButton::callback("📊 Full Stats", "settings:stats"),
        ],
        vec![
            InlineKeyboardButton::callback("❌ Close", "settings:close"),
        ],
    ]);
    
    // Send the settings message with enhanced feedback
    let _settings_response = bot.send_message(msg.chat.id, message_text)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    
    // Update processing message to show completion
    let completion_message = format!(
        "Group settings loaded successfully!\\n\\n📊 Found {} total sessions and {} responses\\n\\n💡 Use the buttons below to configure settings",
        stats.total_sessions, 
        stats.total_responses
    );
    
    feedback.update_message(processing_msg.id, crate::utils::feedback::FeedbackType::Success, &completion_message).await?;
    
    Ok(())
}

#[derive(Default)]
struct GroupStats {
    total_sessions: i32,
    active_sessions: i32,
    confirmed_sessions: i32,
    total_responses: i32,
}

async fn get_group_stats(
    pool: &sqlx::SqlitePool,
    group_id: i64,
) -> Result<GroupStats, sqlx::Error> {
    // Get session counts
    let session_counts = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total,
            COALESCE(SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END), 0) as active,
            COALESCE(SUM(CASE WHEN status = 'confirmed' THEN 1 ELSE 0 END), 0) as confirmed
        FROM sessions 
        WHERE group_id = ?
        "#,
        group_id
    )
    .fetch_one(pool)
    .await?;
    
    // Get total responses count
    let response_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as total
        FROM responses r
        JOIN sessions s ON r.session_id = s.id
        WHERE s.group_id = ?
        "#,
        group_id
    )
    .fetch_one(pool)
    .await?;
    
    Ok(GroupStats {
        total_sessions: session_counts.total as i32,
        active_sessions: session_counts.active.unwrap_or(0) as i32,
        confirmed_sessions: session_counts.confirmed.unwrap_or(0) as i32,
        total_responses: response_count.total as i32,
    })
}