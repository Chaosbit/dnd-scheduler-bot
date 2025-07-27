use anyhow::{anyhow, Result};

pub fn validate_session_title(title: &str) -> Result<()> {
    let title = title.trim();
    
    if title.is_empty() {
        return Err(anyhow!("Session title cannot be empty"));
    }
    
    if title.len() > 100 {
        return Err(anyhow!("Session title cannot be longer than 100 characters"));
    }
    
    // Check for potentially problematic characters
    if title.contains('\n') || title.contains('\r') {
        return Err(anyhow!("Session title cannot contain line breaks"));
    }
    
    Ok(())
}

pub fn validate_telegram_chat_id(chat_id: i64) -> Result<()> {
    // Telegram chat IDs should be non-zero
    if chat_id == 0 {
        return Err(anyhow!("Chat ID cannot be zero"));
    }
    
    // Telegram chat IDs for groups are typically negative
    // Private chats have positive IDs (up to about 2^31)
    // Super groups have very large negative IDs (< -1000000000000)
    if chat_id < -2000000000000 || chat_id > 2147483647 {
        return Err(anyhow!("Invalid Telegram chat ID range"));
    }
    
    Ok(())
}

pub fn validate_time_options(options: &str) -> Result<Vec<String>> {
    let options = options.trim();
    
    if options.is_empty() {
        return Err(anyhow!("Time options cannot be empty"));
    }
    
    let option_list: Vec<String> = options
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    if option_list.is_empty() {
        return Err(anyhow!("Must provide at least one time option"));
    }
    
    if option_list.len() > 10 {
        return Err(anyhow!("Cannot have more than 10 time options"));
    }
    
    // Validate each option length
    for option in &option_list {
        if option.len() > 50 {
            return Err(anyhow!("Time option '{}' is too long (max 50 characters)", option));
        }
    }
    
    Ok(option_list)
}

pub fn validate_response_type(response: &str) -> Result<()> {
    match response.to_lowercase().as_str() {
        "yes" | "no" | "maybe" => Ok(()),
        _ => Err(anyhow!("Response must be 'yes', 'no', or 'maybe'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_session_title_valid() {
        assert!(validate_session_title("Valid Title").is_ok());
        assert!(validate_session_title("Game Night 2024").is_ok());
        assert!(validate_session_title("Adventure with Special Characters: ()[]{}!@#$%^&*").is_ok());
        assert!(validate_session_title("  Trimmed Title  ").is_ok());
    }

    #[test]
    fn test_validate_session_title_empty() {
        assert!(validate_session_title("").is_err());
        assert!(validate_session_title("   ").is_err());
        assert!(validate_session_title("\t\n").is_err());
    }

    #[test]
    fn test_validate_session_title_too_long() {
        let long_title = "a".repeat(101);
        assert!(validate_session_title(&long_title).is_err());
        
        let max_title = "a".repeat(100);
        assert!(validate_session_title(&max_title).is_ok());
    }

    #[test]
    fn test_validate_session_title_line_breaks() {
        assert!(validate_session_title("Title\nwith\nnewlines").is_err());
        assert!(validate_session_title("Title\rwith\rcarriage\rreturns").is_err());
        assert!(validate_session_title("Title\r\nwith\r\nboth").is_err());
    }

    #[test]
    fn test_validate_telegram_chat_id_valid() {
        // Private chat (positive)
        assert!(validate_telegram_chat_id(12345).is_ok());
        assert!(validate_telegram_chat_id(987654321).is_ok());
        
        // Group chat (negative)
        assert!(validate_telegram_chat_id(-12345).is_ok());
        assert!(validate_telegram_chat_id(-987654321).is_ok());
        
        // Super group (very negative)
        assert!(validate_telegram_chat_id(-1001234567890).is_ok());
    }

    #[test]
    fn test_validate_telegram_chat_id_invalid() {
        // Zero
        assert!(validate_telegram_chat_id(0).is_err());
        
        // Out of expected ranges
        assert!(validate_telegram_chat_id(-3000000000000).is_err());
        assert!(validate_telegram_chat_id(3000000000).is_err());
    }

    #[test]
    fn test_validate_time_options_valid() {
        let result = validate_time_options("Friday 19:00, Saturday 14:00");
        assert!(result.is_ok());
        let options = result.unwrap();
        assert_eq!(options.len(), 2);
        assert_eq!(options[0], "Friday 19:00");
        assert_eq!(options[1], "Saturday 14:00");
    }

    #[test]
    fn test_validate_time_options_single() {
        let result = validate_time_options("Monday 20:00");
        assert!(result.is_ok());
        let options = result.unwrap();
        assert_eq!(options.len(), 1);
        assert_eq!(options[0], "Monday 20:00");
    }

    #[test]
    fn test_validate_time_options_trimming() {
        let result = validate_time_options("  Friday 19:00  ,  Saturday 14:00  ");
        assert!(result.is_ok());
        let options = result.unwrap();
        assert_eq!(options[0], "Friday 19:00");
        assert_eq!(options[1], "Saturday 14:00");
    }

    #[test]
    fn test_validate_time_options_empty() {
        assert!(validate_time_options("").is_err());
        assert!(validate_time_options("   ").is_err());
        assert!(validate_time_options(",,,").is_err());
    }

    #[test]
    fn test_validate_time_options_too_many() {
        let many_options = (0..11)
            .map(|i| format!("Option {}", i))
            .collect::<Vec<_>>()
            .join(", ");
        assert!(validate_time_options(&many_options).is_err());
    }

    #[test]
    fn test_validate_time_options_too_long() {
        let long_option = "a".repeat(51);
        assert!(validate_time_options(&long_option).is_err());
        
        let max_option = "a".repeat(50);
        assert!(validate_time_options(&max_option).is_ok());
    }

    #[test]
    fn test_validate_response_type_valid() {
        assert!(validate_response_type("yes").is_ok());
        assert!(validate_response_type("no").is_ok());
        assert!(validate_response_type("maybe").is_ok());
        
        // Case insensitive
        assert!(validate_response_type("YES").is_ok());
        assert!(validate_response_type("No").is_ok());
        assert!(validate_response_type("MAYBE").is_ok());
    }

    #[test]
    fn test_validate_response_type_invalid() {
        assert!(validate_response_type("invalid").is_err());
        assert!(validate_response_type("").is_err());
        assert!(validate_response_type("true").is_err());
        assert!(validate_response_type("false").is_err());
        assert!(validate_response_type("y").is_err());
        assert!(validate_response_type("n").is_err());
    }

    #[test]
    fn test_validate_response_type_edge_cases() {
        assert!(validate_response_type("  yes  ").is_err()); // Whitespace not trimmed
        assert!(validate_response_type("yes\n").is_err());   // With newline
    }
}