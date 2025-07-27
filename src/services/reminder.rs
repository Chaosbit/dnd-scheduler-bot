use tokio_cron_scheduler::{JobScheduler, Job};
use chrono::{Utc, Duration};
use teloxide::{Bot, prelude::*};
use crate::database::{connection::DatabaseManager, models::*};
use crate::utils::datetime::format_datetime;
use std::sync::Arc;

pub struct ReminderService {
    bot: Bot,
    db: Arc<DatabaseManager>,
    scheduler: JobScheduler,
}

impl ReminderService {
    pub async fn new(bot: Bot, db: Arc<DatabaseManager>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let scheduler = JobScheduler::new().await?;
        
        Ok(Self {
            bot,
            db,
            scheduler,
        })
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Run reminder checks twice daily at 9 AM and 6 PM UTC
        let bot = self.bot.clone();
        let db = self.db.clone();
        
        let reminder_job = Job::new_async("0 0 9,18 * * *", move |_uuid, _l| {
            let bot = bot.clone();
            let db = db.clone();
            Box::pin(async move {
                if let Err(e) = check_and_send_reminders(bot, db).await {
                    tracing::error!("Failed to send reminders: {}", e);
                }
            })
        })?;
        
        self.scheduler.add(reminder_job).await?;
        self.scheduler.start().await?;
        
        tracing::info!("Reminder service started - checking twice daily at 9 AM and 6 PM UTC");
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.scheduler.shutdown().await?;
        Ok(())
    }
    
    // Manual trigger for testing
    pub async fn check_reminders_now(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        check_and_send_reminders(self.bot.clone(), self.db.clone()).await
    }
}

async fn check_and_send_reminders(
    bot: Bot,
    db: Arc<DatabaseManager>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let now = Utc::now();
    
    // Define reminder intervals
    let reminder_intervals = vec![
        (Duration::days(14), "📅 **2 Week Reminder**"),
        (Duration::days(7), "📅 **1 Week Reminder**"), 
        (Duration::days(3), "📅 **3 Day Reminder**"),
    ];
    
    // Get all confirmed sessions
    let confirmed_sessions = get_confirmed_sessions(&db.pool).await?;
    
    for session in confirmed_sessions {
        // Get the confirmed session option
        let session_options = SessionOption::find_by_session(&db.pool, &session.id).await?;
        let confirmed_option = session_options.iter()
            .find(|opt| opt.confirmed)
            .ok_or("No confirmed option found for confirmed session")?;
        
        // Parse the session datetime
        let session_datetime = chrono::DateTime::parse_from_rfc3339(&confirmed_option.datetime)?
            .with_timezone(&Utc);
        
        // Check if we need to send any reminders
        for (interval, reminder_type) in &reminder_intervals {
            let reminder_time = session_datetime - *interval;
            let time_diff = (now - reminder_time).num_hours().abs();
            
            // Send reminder if we're within 1 hour of the reminder time
            // and haven't sent this reminder before
            if time_diff <= 1 {
                if !has_reminder_been_sent(&db.pool, &session.id, interval.num_days()).await? {
                    let success = send_session_reminder(
                        &bot,
                        &session,
                        confirmed_option,
                        reminder_type,
                        &session_datetime,
                        db.as_ref(),
                    ).await;
                    
                    if success {
                        // Mark reminder as sent
                        mark_reminder_sent(&db.pool, &session.id, interval.num_days()).await?;
                        tracing::info!(
                            "Sent {} reminder for session: {}",
                            interval.num_days(),
                            session.title
                        );
                    }
                }
            }
        }
    }
    
    Ok(())
}

async fn send_session_reminder(
    bot: &Bot,
    session: &Session,
    confirmed_option: &SessionOption,
    reminder_type: &str,
    session_datetime: &chrono::DateTime<Utc>,
    db: &DatabaseManager,
) -> bool {
    let formatted_datetime = format_datetime(session_datetime);
    let duration_hours = confirmed_option.duration / 60;
    let duration_display = if duration_hours >= 1 {
        format!("{}h", duration_hours)
    } else {
        format!("{}min", confirmed_option.duration)
    };
    
    // Get participants who said "yes"
    let responses = match Response::find_by_session(&db.pool, &session.id).await {
        Ok(r) => r,
        Err(_) => Vec::new(),
    };
    
    let participants: Vec<_> = responses.iter()
        .filter(|r| r.option_id == confirmed_option.id && r.response == "yes")
        .filter_map(|r| r.username.as_deref())
        .collect();
    
    let participant_list = if participants.is_empty() {
        "No participants confirmed yet".to_string()
    } else if participants.len() <= 5 {
        participants.join(", ")
    } else {
        format!("{} and {} others", participants[..3].join(", "), participants.len() - 3)
    };
    
    let message_text = format!(
        "{}\n\n🎲 **{}**\n\n📅 **When:** {}\n⏱️ **Duration:** {}\n👥 **Participants:** {}\n\n🔗 Session ID: `{}`",
        reminder_type,
        escape_markdown(&session.title),
        escape_markdown(&formatted_datetime),
        duration_display,
        escape_markdown(&participant_list),
        session.id
    );
    
    match bot.send_message(teloxide::types::ChatId(session.group_id), message_text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await 
    {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("Failed to send reminder to group {}: {}", session.group_id, e);
            false
        }
    }
}

// Database helper functions
async fn get_confirmed_sessions(
    pool: &sqlx::SqlitePool,
) -> Result<Vec<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        "SELECT id, group_id, title, message_id, status, deadline, created_by, created_at 
         FROM sessions 
         WHERE status = 'confirmed' 
         ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}

async fn has_reminder_been_sent(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    days_before: i64,
) -> Result<bool, sqlx::Error> {
    Reminder::exists(pool, session_id, days_before).await
}

async fn mark_reminder_sent(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    days_before: i64,
) -> Result<(), sqlx::Error> {
    Reminder::create(pool, session_id.to_string(), days_before).await?;
    Ok(())
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