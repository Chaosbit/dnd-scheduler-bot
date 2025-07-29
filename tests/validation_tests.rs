use dnd_scheduler_bot::utils::validation::*;

#[cfg(test)]
mod validation_tests {
    use super::*;

    // Session title validation tests
    #[test]
    fn test_valid_session_titles() {
        let valid_titles = vec![
            "D&D Session".to_string(),
            "Weekly Adventure".to_string(),
            "Campaign Finale".to_string(),
            "Short".to_string(),
            "A".repeat(100), // Exactly 100 characters
            "🎲 Epic Adventure".to_string(),
            "Session #1".to_string(),
            "My D&D Campaign - Session 12".to_string(),
            "Adventure & Exploration".to_string(),
        ];

        for title in valid_titles {
            assert!(validate_session_title(&title).is_ok(), "Should accept title: {}", title);
        }
    }

    #[test]
    fn test_invalid_session_titles() {
        let invalid_titles = vec![
            "".to_string(), // Empty
            "AB".to_string(), // Too short (less than 3 characters)
            "A".repeat(101), // Too long (more than 100 characters)
            "   ".to_string(), // Only whitespace
        ];

        for title in invalid_titles {
            assert!(validate_session_title(&title).is_err(), "Should reject title: {}", title);
        }
    }

    #[test]
    fn test_session_title_whitespace_handling() {
        // Should trim whitespace
        assert!(validate_session_title("  Valid Title  ").is_ok());
        
        // But should reject if only whitespace after trimming
        assert!(validate_session_title("   ").is_err());
    }

    // Telegram chat ID validation tests
    #[test]
    fn test_valid_telegram_chat_ids() {
        let valid_chat_ids = vec![
            -1001234567890_i64, // Supergroup
            -987654321_i64,     // Group
            123456789_i64,      // Private chat (positive)
        ];

        for chat_id in valid_chat_ids {
            assert!(validate_telegram_chat_id(chat_id).is_ok(), "Should accept chat_id: {}", chat_id);
        }
    }

    #[test]
    fn test_invalid_telegram_chat_ids() {
        let invalid_chat_ids = vec![
            0_i64,    // Zero is invalid
            -1_i64,   // Small negative numbers are invalid
            -100_i64, // Too small for a valid group
        ];

        for chat_id in invalid_chat_ids {
            assert!(validate_telegram_chat_id(chat_id).is_err(), "Should reject chat_id: {}", chat_id);
        }
    }

    // Time options validation tests
    #[test]
    fn test_valid_time_options() {
        let valid_options = vec![
            "Friday 19:00".to_string(),
            "Monday 14:30".to_string(),
            "Friday 19:00, Saturday 14:30".to_string(),
            "Monday 18:00, Tuesday 19:00, Wednesday 20:00".to_string(),
            "Next Friday 19:00".to_string(),
            "Tomorrow 14:30".to_string(),
            "Sunday 16:00".to_string(),
            "Friday 19.00".to_string(), // Alternative time format
            "Monday 14.30, Tuesday 15.45".to_string(),
        ];

        for options in valid_options {
            let result = validate_time_options(&options);
            assert!(result.is_ok(), "Should accept time options: {} - Error: {:?}", options, result.err());
        }
    }

    #[test]
    fn test_invalid_time_options() {
        let invalid_options = vec![
            "".to_string(), // Empty
            "   ".to_string(), // Only whitespace
            "Invalid time format".to_string(),
            "25:00".to_string(), // Invalid hour
            "12:60".to_string(), // Invalid minute
            "Friday".to_string(), // Missing time
            "19:00".to_string(), // Missing day
            ",".to_string(), // Only comma
            "Friday 19:00,".to_string(), // Trailing comma
            ",Saturday 14:30".to_string(), // Leading comma
        ];

        for options in invalid_options {
            assert!(validate_time_options(&options).is_err(), "Should reject time options: {}", options);
        }
    }

    #[test]
    fn test_time_options_parsing() {
        let options = "Friday 19:00, Saturday 14:30, Sunday 16:00";
        let result = validate_time_options(&options);
        
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], "Friday 19:00");
        assert_eq!(parsed[1], "Saturday 14:30");
        assert_eq!(parsed[2], "Sunday 16:00");
    }

    #[test]
    fn test_time_options_whitespace_handling() {
        let options = "  Friday 19:00  ,  Saturday 14:30  ";
        let result = validate_time_options(&options);
        
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], "Friday 19:00");
        assert_eq!(parsed[1], "Saturday 14:30");
    }

    #[test]
    fn test_single_time_option() {
        let options = "Friday 19:00";
        let result = validate_time_options(&options);
        
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], "Friday 19:00");
    }

    // Response type validation tests
    #[test]
    fn test_valid_response_types() {
        let valid_responses = vec![
            "yes".to_string(),
            "no".to_string(), 
            "maybe".to_string(),
            "YES".to_string(), // Should handle case insensitivity
            "No".to_string(),
            "MAYBE".to_string(),
        ];

        for response in valid_responses {
            assert!(validate_response_type(&response).is_ok(), "Should accept response: {}", response);
        }
    }

    #[test]
    fn test_invalid_response_types() {
        let invalid_responses = vec![
            "".to_string(),
            "yep".to_string(),
            "nope".to_string(),
            "perhaps".to_string(),
            "ok".to_string(),
            "sure".to_string(),
            "y".to_string(),
            "n".to_string(),
            "m".to_string(),
            "true".to_string(),
            "false".to_string(),
            "1".to_string(),
            "0".to_string(),
        ];

        for response in invalid_responses {
            assert!(validate_response_type(&response).is_err(), "Should reject response: {}", response);
        }
    }

    // Edge cases and special scenarios
    #[test]
    fn test_time_options_with_different_formats() {
        let test_cases = vec![
            ("Friday 19:00", 1),
            ("Friday 19.00", 1),
            ("Mon 14:30", 1),
            ("Monday 14.30", 1),
            ("Next Friday 19:00", 1),
            ("Tomorrow 14:30", 1),
        ];

        for (options, expected_count) in test_cases {
            let result = validate_time_options(&options);
            assert!(result.is_ok(), "Should accept: {}", options);
            assert_eq!(result.unwrap().len(), expected_count, "Wrong count for: {}", options);
        }
    }

    #[test]
    fn test_session_title_unicode_support() {
        let unicode_titles = vec![
            "🎲 D&D Session".to_string(),
            "Dungeons & Dragons".to_string(),
            "Приключение".to_string(), // Russian
            "冒険".to_string(), // Japanese
            "Aventure".to_string(), // French with accent
        ];

        for title in unicode_titles {
            assert!(validate_session_title(&title).is_ok(), "Should accept unicode title: {}", title);
        }
    }

    #[test]
    fn test_telegram_chat_id_boundaries() {
        // Test boundary values for Telegram chat IDs
        let boundary_tests = vec![
            (-1002147483648_i64, true),  // Largest supergroup ID
            (-1000000000000_i64, true),  // Valid supergroup range
            (-2147483648_i64, true),     // Largest group ID
            (-1000000000_i64, true),     // Valid group range
            (2147483647_i64, true),      // Largest user ID
            (1_i64, true),               // Smallest positive user ID
            (-999999999_i64, false),     // Just outside valid group range
            (-1_i64, false),             // Invalid small negative
        ];

        for (chat_id, should_be_valid) in boundary_tests {
            let result = validate_telegram_chat_id(chat_id);
            if should_be_valid {
                assert!(result.is_ok(), "Should accept chat_id: {}", chat_id);
            } else {
                assert!(result.is_err(), "Should reject chat_id: {}", chat_id);
            }
        }
    }

    #[test]
    fn test_time_options_maximum_count() {
        // Test with many time options to ensure no arbitrary limits
        let many_options: Vec<String> = (1..=10)
            .map(|i| format!("Day{} 1{}:00", i, i))
            .collect();
        let options_string = many_options.join(", ");
        
        let result = validate_time_options(&options_string);
        assert!(result.is_ok(), "Should handle many time options");
        assert_eq!(result.unwrap().len(), 10);
    }

    #[test]
    fn test_session_title_length_boundaries() {
        // Test exact boundary conditions
        assert!(validate_session_title(&"A".repeat(3)).is_ok());  // Minimum length
        assert!(validate_session_title(&"A".repeat(100)).is_ok()); // Maximum length
        assert!(validate_session_title(&"A".repeat(2)).is_err());  // Below minimum
        assert!(validate_session_title(&"A".repeat(101)).is_err()); // Above maximum
    }

    #[test]
    fn test_response_type_case_insensitivity() {
        let case_variants = vec![
            ("yes", true), ("YES", true), ("Yes", true), ("yEs", true),
            ("no", true), ("NO", true), ("No", true), ("nO", true),
            ("maybe", true), ("MAYBE", true), ("Maybe", true), ("mAyBe", true),
        ];

        for (response, should_be_valid) in case_variants {
            let result = validate_response_type(&response);
            if should_be_valid {
                assert!(result.is_ok(), "Should accept case variant: {}", response);
            } else {
                assert!(result.is_err(), "Should reject case variant: {}", response);
            }
        }
    }

    #[test]
    fn test_time_options_error_messages() {
        let result = validate_time_options("");
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.to_string().contains("empty"));

        let result = validate_time_options("Invalid format");
        assert!(result.is_err());
        // Error message should be helpful
        let error = result.err().unwrap();
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn test_session_title_error_messages() {
        let result = validate_session_title("");
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.to_string().contains("3"));

        let result = validate_session_title(&"A".repeat(101));
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.to_string().contains("100"));
    }
}