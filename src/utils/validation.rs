use anyhow::{Result, anyhow};

pub fn validate_session_title(title: &str) -> Result<()> {
    if title.is_empty() {
        return Err(anyhow!("Session title cannot be empty"));
    }
    
    if title.len() > 100 {
        return Err(anyhow!("Session title cannot exceed 100 characters"));
    }
    
    Ok(())
}

pub fn validate_telegram_chat_id(chat_id: i64) -> Result<()> {
    if chat_id == 0 {
        return Err(anyhow!("Invalid chat ID"));
    }
    
    Ok(())
}
