use teloxide::prelude::*;
use crate::database::connection::DatabaseManager;

pub async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    _db: DatabaseManager,
) -> ResponseResult<()> {
    if let Some(data) = q.data {
        // Parse callback data: "session_id:option_id:response"
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() == 3 {
            let _session_id = parts[0];
            let _option_id = parts[1];
            let response = parts[2];
            
            let _user = &q.from;
            // TODO: Update response in database and refresh message
            bot.answer_callback_query(q.id)
                .text(format!("Marked as {}", response))
                .await?;
        }
    }
    
    Ok(())
}
