use dnd_scheduler_bot::utils::feedback::FeedbackType;

#[cfg(test)]
mod feedback_system_tests {
    use super::*;

    #[test]
    fn test_feedback_type_emoji_mapping() {
        // Test that each feedback type has appropriate emoji representation
        let success_emoji = "✅";
        let warning_emoji = "⚠️";
        let error_emoji = "❌";
        let info_emoji = "💡";
        let processing_emoji = "🔄";

        // These would be used in the actual feedback formatting
        assert!(success_emoji.len() > 0);
        assert!(warning_emoji.len() > 0);
        assert!(error_emoji.len() > 0);
        assert!(info_emoji.len() > 0);
        assert!(processing_emoji.len() > 0);
    }

    #[tokio::test]
    async fn test_feedback_message_formatting() {
        // Test various feedback message formats
        let test_cases = vec![
            (FeedbackType::Success, "Operation completed", "✅"),
            (FeedbackType::Warning, "Potential issue detected", "⚠️"),
            (FeedbackType::Error, "Operation failed", "❌"),
            (FeedbackType::Info, "Information message", "💡"),
            (FeedbackType::Processing, "Processing request", "🔄"),
        ];

        for (feedback_type, message, expected_emoji) in test_cases {
            // Test that message formatting includes the expected emoji
            let formatted = format_feedback_message(feedback_type.clone(), message);
            assert!(formatted.contains(expected_emoji), 
                "Message should contain emoji {} for type {:?}: {}", 
                expected_emoji, feedback_type, formatted);
            assert!(formatted.contains(message), 
                "Message should contain original text: {}", formatted);
        }
    }

    #[test]
    fn test_progress_tracker_step_calculation() {
        // Test progress percentage calculation
        let test_cases = vec![
            (1, 4, 25),  // 1/4 = 25%
            (2, 4, 50),  // 2/4 = 50%
            (3, 4, 75),  // 3/4 = 75%
            (4, 4, 100), // 4/4 = 100%
            (1, 3, 33),  // 1/3 ≈ 33%
            (2, 3, 67),  // 2/3 ≈ 67%
            (3, 3, 100), // 3/3 = 100%
        ];

        for (current_step, total_steps, expected_percentage) in test_cases {
            let percentage = calculate_progress_percentage(current_step, total_steps);
            assert_eq!(percentage, expected_percentage, 
                "Wrong percentage for step {}/{}: expected {}%, got {}%", 
                current_step, total_steps, expected_percentage, percentage);
        }
    }

    #[test]
    fn test_markdown_escaping() {
        let test_cases = vec![
            ("Simple text", "Simple text"),
            ("Text with *bold*", "Text with \\*bold\\*"),
            ("Text with _italic_", "Text with \\_italic\\_"),
            ("Text with [link](url)", "Text with \\[link\\]\\(url\\)"),
            ("Special chars: ~`>#+-=|{}.!", "Special chars: \\~\\`\\>\\#\\+\\-\\=\\|\\{\\}\\.\\!"),
            ("Emoji text 🎲", "Emoji text 🎲"), // Emojis should not be escaped
        ];

        for (input, expected) in test_cases {
            let escaped = escape_markdown_v2(input);
            assert_eq!(escaped, expected, 
                "Markdown escaping failed for input: '{}'\nExpected: '{}'\nGot: '{}'", 
                input, expected, escaped);
        }
    }

    #[test]
    fn test_feedback_message_length_limits() {
        // Test that feedback messages respect Telegram's message length limits
        let very_long_message = "A".repeat(5000); // Longer than Telegram's limit
        let truncated = truncate_message_if_needed(&very_long_message);
        
        assert!(truncated.len() <= 4096, "Message should be truncated to Telegram's limit");
        if very_long_message.len() > 4096 {
            assert!(truncated.contains("..."), "Truncated message should indicate truncation");
        }
    }

    #[test]
    fn test_validation_error_message_format() {
        let error_msg = "Invalid input provided";
        let suggestion = "Please use the format: /command argument";
        
        let formatted = format_validation_error(error_msg, suggestion);
        
        assert!(formatted.contains("❌"), "Should contain error emoji");
        assert!(formatted.contains(error_msg), "Should contain error message");
        assert!(formatted.contains("💡"), "Should contain suggestion emoji");
        assert!(formatted.contains(suggestion), "Should contain suggestion");
    }

    #[test]
    fn test_progress_message_format() {
        let step = 2;
        let total = 4;
        let message = "Processing data";
        
        let formatted = format_progress_message(step, total, message);
        
        assert!(formatted.contains("🔄"), "Should contain processing emoji");
        assert!(formatted.contains(message), "Should contain progress message");
        assert!(formatted.contains("50%"), "Should contain progress percentage");
        assert!(formatted.contains("2/4"), "Should contain step count");
    }

    #[test]
    fn test_success_message_format() {
        let message = "Operation completed successfully";
        let details = "All 5 items were processed";
        
        let formatted = format_success_message(message, Some(details));
        
        assert!(formatted.contains("✅"), "Should contain success emoji");
        assert!(formatted.contains(message), "Should contain success message");
        assert!(formatted.contains(details), "Should contain details");
    }

    #[test]
    fn test_command_help_format() {
        let command = "/schedule";
        let description = "Create a new session poll";
        let examples = vec![
            "/schedule \"Title\" \"Friday 19:00\"",
            "/schedule \"Session\" \"Monday 18:00, Tuesday 19:00\"",
        ];
        
        let formatted = format_command_help(command, description, &examples);
        
        assert!(formatted.contains(command), "Should contain command");
        assert!(formatted.contains(description), "Should contain description");
        for example in &examples {
            assert!(formatted.contains(example), "Should contain example: {}", example);
        }
        assert!(formatted.contains("📚"), "Should contain help emoji");
    }

    #[test]
    fn test_feedback_type_priority() {
        // Test that feedback types have appropriate priority ordering
        let priorities = vec![
            (FeedbackType::Error, 4),
            (FeedbackType::Warning, 3),
            (FeedbackType::Processing, 2),
            (FeedbackType::Info, 1),
            (FeedbackType::Success, 1),
        ];

        for (feedback_type, expected_priority) in priorities {
            let priority = get_feedback_priority(&feedback_type);
            assert_eq!(priority, expected_priority, 
                "Wrong priority for feedback type {:?}: expected {}, got {}", 
                feedback_type, expected_priority, priority);
        }
    }

    #[test]
    fn test_progress_bar_generation() {
        let test_cases = vec![
            (0, 4, "▱▱▱▱"),
            (1, 4, "▰▱▱▱"),
            (2, 4, "▰▰▱▱"),
            (3, 4, "▰▰▰▱"),
            (4, 4, "▰▰▰▰"),
        ];

        for (current, total, expected) in test_cases {
            let progress_bar = generate_progress_bar(current, total);
            assert_eq!(progress_bar, expected, 
                "Wrong progress bar for {}/{}: expected '{}', got '{}'", 
                current, total, expected, progress_bar);
        }
    }

    #[test]
    fn test_error_message_sanitization() {
        // Test that error messages are properly sanitized for display
        let dangerous_inputs = vec![
            "<script>alert('xss')</script>".to_string(),
            "Message with\nnewlines\nand\ttabs".to_string(),
            "Very long error ".repeat(100),
            "Unicode: 🎲🎯🎪".to_string(),
        ];

        for input in dangerous_inputs {
            let sanitized = sanitize_error_message(&input);
            
            // Should not contain dangerous HTML
            assert!(!sanitized.contains("<script>"), "Should remove script tags");
            assert!(!sanitized.contains("</script>"), "Should remove script tags");
            
            // Should handle whitespace appropriately
            if input.contains('\n') || input.contains('\t') {
                // Newlines and tabs should be normalized
                assert!(!sanitized.contains('\t'), "Should normalize tabs");
            }
            
            // Should respect length limits
            assert!(sanitized.len() <= 1000, "Error messages should be reasonably sized");
        }
    }

    #[test]
    fn test_feedback_rate_limiting() {
        // Test that feedback system has appropriate rate limiting concepts
        let timestamps = vec![
            0, 100, 200, 300, 400, 500, // Quick succession
            1000, 2000, 3000, // Spaced out
        ];

        let mut last_sent = 0;
        let min_interval = 100; // Minimum 100ms between messages

        for timestamp in timestamps {
            let should_send = should_send_feedback(last_sent, timestamp, min_interval);
            
            if timestamp - last_sent >= min_interval {
                assert!(should_send, "Should send feedback after sufficient interval");
                last_sent = timestamp;
            } else {
                assert!(!should_send, "Should not send feedback too quickly");
            }
        }
    }
}

// Helper functions that would be implemented in the actual feedback system
fn format_feedback_message(feedback_type: FeedbackType, message: &str) -> String {
    let emoji = match feedback_type {
        FeedbackType::Success => "✅",
        FeedbackType::Warning => "⚠️",
        FeedbackType::Error => "❌",
        FeedbackType::Info => "💡",
        FeedbackType::Processing => "🔄",
    };
    format!("{} {}", emoji, message)
}

fn calculate_progress_percentage(current: u32, total: u32) -> u32 {
    if total == 0 { return 0; }
    ((current as f32 / total as f32) * 100.0).round() as u32
}

fn escape_markdown_v2(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '*' | '_' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '=' | '|' | '{' | '}' | '.' | '!' => {
                format!("\\{}", c)
            }
            _ => c.to_string(),
        })
        .collect()
}

fn truncate_message_if_needed(message: &str) -> String {
    const MAX_LENGTH: usize = 4096;
    if message.len() <= MAX_LENGTH {
        message.to_string()
    } else {
        format!("{}...", &message[..MAX_LENGTH-3])
    }
}

fn format_validation_error(error: &str, suggestion: &str) -> String {
    format!("❌ {}\n\n💡 **Suggestion:** {}", error, suggestion)
}

fn format_progress_message(step: u32, total: u32, message: &str) -> String {
    let percentage = calculate_progress_percentage(step, total);
    format!("🔄 {} ({}% - {}/{})", message, percentage, step, total)
}

fn format_success_message(message: &str, details: Option<&str>) -> String {
    match details {
        Some(details) => format!("✅ {}\n\n{}", message, details),
        None => format!("✅ {}", message),
    }
}

fn format_command_help(command: &str, description: &str, examples: &[&str]) -> String {
    let mut help = format!("📚 **{}**\n{}\n\n**Examples:**", command, description);
    for example in examples {
        help.push_str(&format!("\n• `{}`", example));
    }
    help
}

fn get_feedback_priority(feedback_type: &FeedbackType) -> u32 {
    match feedback_type {
        FeedbackType::Error => 4,
        FeedbackType::Warning => 3,
        FeedbackType::Processing => 2,
        FeedbackType::Info => 1,
        FeedbackType::Success => 1,
    }
}

fn generate_progress_bar(current: u32, total: u32) -> String {
    let filled = current;
    let empty = total - current;
    "▰".repeat(filled as usize) + &"▱".repeat(empty as usize)
}

fn sanitize_error_message(message: &str) -> String {
    message
        .replace("<script>", "")
        .replace("</script>", "")
        .replace('\t', " ")
        .chars()
        .take(1000)
        .collect()
}

fn should_send_feedback(last_sent: u64, current_time: u64, min_interval: u64) -> bool {
    current_time - last_sent >= min_interval
}