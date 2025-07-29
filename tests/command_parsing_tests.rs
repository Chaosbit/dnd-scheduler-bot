use dnd_scheduler_bot::bot::commands::Command;
use teloxide::utils::command::BotCommands;

#[cfg(test)]
mod command_parsing_tests {
    use super::*;

    #[test]
    fn test_help_command_parsing() {
        let input = "/help";
        let result = Command::parse(input, "testbot");
        assert!(result.is_ok());
        matches!(result.unwrap(), Command::Help);
    }

    #[test]
    fn test_start_command_parsing() {
        let input = "/start";
        let result = Command::parse(input, "testbot");
        assert!(result.is_ok());
        matches!(result.unwrap(), Command::Start);
    }

    #[test]
    fn test_list_command_parsing() {
        let input = "/list";
        let result = Command::parse(input, "testbot");
        assert!(result.is_ok());
        matches!(result.unwrap(), Command::List);
    }

    #[test]
    fn test_settings_command_parsing() {
        let input = "/settings";
        let result = Command::parse(input, "testbot");
        assert!(result.is_ok());
        matches!(result.unwrap(), Command::Settings);
    }

    #[test]
    fn test_stats_command_parsing() {
        let input = "/stats";
        let result = Command::parse(input, "testbot");
        assert!(result.is_ok());
        matches!(result.unwrap(), Command::Stats);
    }

    #[test]
    fn test_testreminders_command_parsing() {
        let input = "/testreminders";
        let result = Command::parse(input, "testbot");
        assert!(result.is_ok());
        matches!(result.unwrap(), Command::TestReminders);
    }

    // Schedule command tests - quoted arguments
    #[test]
    fn test_schedule_command_with_quoted_arguments() {
        let input = "/schedule \"Weekly D&D Session\" \"Friday 19:00, Saturday 14:30\"";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Schedule { title, options } => {
                assert_eq!(title, "Weekly D&D Session");
                assert_eq!(options, "Friday 19:00, Saturday 14:30");
            }
            _ => panic!("Expected Schedule command"),
        }
    }

    #[test]
    fn test_schedule_command_with_quoted_title_only() {
        let input = "/schedule \"Test Session\" Friday 19:00";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Schedule { title, options } => {
                assert_eq!(title, "Test Session");
                assert_eq!(options, "Friday 19:00");
            }
            _ => panic!("Expected Schedule command"),
        }
    }

    #[test]
    fn test_schedule_command_with_spaces_in_quoted_title() {
        let input = "/schedule \"My Amazing D&D Campaign Session\" \"Next Friday 20:00\"";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Schedule { title, options } => {
                assert_eq!(title, "My Amazing D&D Campaign Session");
                assert_eq!(options, "Next Friday 20:00");
            }
            _ => panic!("Expected Schedule command"),
        }
    }

    #[test]
    fn test_schedule_command_with_unquoted_arguments() {
        let input = "/schedule TestSession Friday 19:00, Saturday 14:30";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Schedule { title, options } => {
                assert_eq!(title, "TestSession");
                assert_eq!(options, "Friday 19:00, Saturday 14:30");
            }
            _ => panic!("Expected Schedule command"),
        }
    }

    #[test]
    fn test_schedule_command_with_single_word_title() {
        let input = "/schedule Adventure Monday 18:00";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Schedule { title, options } => {
                assert_eq!(title, "Adventure");
                assert_eq!(options, "Monday 18:00");
            }
            _ => panic!("Expected Schedule command"),
        }
    }

    #[test]
    fn test_schedule_command_with_multiple_time_options() {
        let input = "/schedule \"Session\" \"Friday 19:00, Saturday 14:30, Sunday 16:00\"";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Schedule { title, options } => {
                assert_eq!(title, "Session");
                assert_eq!(options, "Friday 19:00, Saturday 14:30, Sunday 16:00");
            }
            _ => panic!("Expected Schedule command"),
        }
    }

    #[test]
    fn test_schedule_command_empty_arguments() {
        let input = "/schedule";
        let result = Command::parse(input, "testbot");
        assert!(result.is_err());
    }

    #[test]
    fn test_schedule_command_only_title() {
        let input = "/schedule \"Only Title\"";
        let result = Command::parse(input, "testbot");
        
        // This should still work with empty options
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Schedule { title, options } => {
                assert_eq!(title, "Only Title");
                assert_eq!(options, "");
            }
            _ => panic!("Expected Schedule command"),
        }
    }

    #[test]
    fn test_schedule_command_malformed_quotes() {
        let input = "/schedule \"Unclosed quote Friday 19:00";
        let result = Command::parse(input, "testbot");
        
        // Should handle malformed quotes gracefully
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Schedule { title, options } => {
                assert_eq!(title, "Unclosed quote Friday 19:00");
                assert_eq!(options, "");
            }
            _ => panic!("Expected Schedule command"),
        }
    }

    // Confirm command tests
    #[test]
    fn test_confirm_command_parsing() {
        let input = "/confirm abc123def456";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Confirm { session_id } => {
                assert_eq!(session_id, "abc123def456");
            }
            _ => panic!("Expected Confirm command"),
        }
    }

    #[test]
    fn test_confirm_command_with_uuid() {
        let input = "/confirm 550e8400-e29b-41d4-a716-446655440000";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Confirm { session_id } => {
                assert_eq!(session_id, "550e8400-e29b-41d4-a716-446655440000");
            }
            _ => panic!("Expected Confirm command"),
        }
    }

    #[test]
    fn test_confirm_command_empty_session_id() {
        let input = "/confirm";
        let result = Command::parse(input, "testbot");
        assert!(result.is_err());
    }

    // Cancel command tests
    #[test]
    fn test_cancel_command_parsing() {
        let input = "/cancel abc123def456";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Cancel { session_id } => {
                assert_eq!(session_id, "abc123def456");
            }
            _ => panic!("Expected Cancel command"),
        }
    }

    #[test]
    fn test_cancel_command_empty_session_id() {
        let input = "/cancel";
        let result = Command::parse(input, "testbot");
        assert!(result.is_err());
    }

    // Deadline command tests
    #[test]
    fn test_deadline_command_parsing() {
        let input = "/deadline abc123def456 Friday 18:00";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Deadline { session_id, datetime } => {
                assert_eq!(session_id, "abc123def456");
                assert_eq!(datetime, "Friday 18:00");
            }
            _ => panic!("Expected Deadline command"),
        }
    }

    #[test]
    fn test_deadline_command_with_complex_datetime() {
        let input = "/deadline session123 Next Friday 19:30";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Deadline { session_id, datetime } => {
                assert_eq!(session_id, "session123");
                assert_eq!(datetime, "Next Friday 19:30");
            }
            _ => panic!("Expected Deadline command"),
        }
    }

    #[test]
    fn test_deadline_command_with_date_and_time() {
        let input = "/deadline abc123 Monday 14:30";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Deadline { session_id, datetime } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(datetime, "Monday 14:30");
            }
            _ => panic!("Expected Deadline command"),
        }
    }

    #[test]
    fn test_deadline_command_only_session_id() {
        let input = "/deadline abc123";
        let result = Command::parse(input, "testbot");
        assert!(result.is_err());
    }

    #[test]
    fn test_deadline_command_empty_arguments() {
        let input = "/deadline";
        let result = Command::parse(input, "testbot");
        assert!(result.is_err());
    }

    // Edge cases and error handling
    #[test]
    fn test_unknown_command() {
        let input = "/unknown_command";
        let result = Command::parse(input, "testbot");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_with_bot_username() {
        let input = "/help@testbot";
        let result = Command::parse(input, "testbot");
        assert!(result.is_ok());
        matches!(result.unwrap(), Command::Help);
    }

    #[test]
    fn test_command_with_different_bot_username() {
        let input = "/help@otherbot";
        let result = Command::parse(input, "testbot");
        // Should fail because it's not for our bot
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_commands() {
        let input = "/HELP";
        let result = Command::parse(input, "testbot");
        assert!(result.is_ok());
        matches!(result.unwrap(), Command::Help);
    }

    #[test]
    fn test_schedule_command_with_emoji_in_title() {
        let input = "/schedule \"🎲 Epic D&D Session\" \"Friday 19:00\"";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Schedule { title, options } => {
                assert_eq!(title, "🎲 Epic D&D Session");
                assert_eq!(options, "Friday 19:00");
            }
            _ => panic!("Expected Schedule command"),
        }
    }

    #[test]
    fn test_schedule_command_with_special_characters() {
        let input = "/schedule \"Session & Adventure\" \"Friday 19:00 - Saturday 14:30\"";
        let result = Command::parse(input, "testbot");
        
        assert!(result.is_ok());
        match result.unwrap() {
            Command::Schedule { title, options } => {
                assert_eq!(title, "Session & Adventure");
                assert_eq!(options, "Friday 19:00 - Saturday 14:30");
            }
            _ => panic!("Expected Schedule command"),
        }
    }

    #[test]
    fn test_commands_description() {
        // Test that command descriptions are available
        let descriptions = Command::descriptions().to_string();
        assert!(descriptions.contains("help"));
        assert!(descriptions.contains("start"));
        assert!(descriptions.contains("schedule"));
        assert!(descriptions.contains("confirm"));
        assert!(descriptions.contains("cancel"));
        assert!(descriptions.contains("deadline"));
        assert!(descriptions.contains("list"));
        assert!(descriptions.contains("settings"));
        assert!(descriptions.contains("stats"));
    }

    // Real-world usage scenarios
    #[test]
    fn test_real_world_schedule_examples() {
        let test_cases = vec![
            ("/schedule \"Weekly D&D\" \"Friday 19:00\"", "Weekly D&D", "Friday 19:00"),
            ("/schedule Session \"Next Friday at 8pm\"", "Session", "Next Friday at 8pm"),
            ("/schedule \"Campaign Finale\" \"Saturday 14:00, Sunday 15:00\"", "Campaign Finale", "Saturday 14:00, Sunday 15:00"),
            ("/schedule Adventure Monday", "Adventure", "Monday"),
        ];

        for (input, expected_title, expected_options) in test_cases {
            let result = Command::parse(input, "testbot");
            assert!(result.is_ok(), "Failed to parse: {}", input);
            
            match result.unwrap() {
                Command::Schedule { title, options } => {
                    assert_eq!(title, expected_title, "Title mismatch for input: {}", input);
                    assert_eq!(options, expected_options, "Options mismatch for input: {}", input);
                }
                _ => panic!("Expected Schedule command for input: {}", input),
            }
        }
    }

    #[test]
    fn test_real_world_deadline_examples() {
        let test_cases = vec![
            ("/deadline abc123 Friday 18:00", "abc123", "Friday 18:00"),
            ("/deadline session456 Next Monday 14:30", "session456", "Next Monday 14:30"),
            ("/deadline uuid-123-456 Tomorrow 20:00", "uuid-123-456", "Tomorrow 20:00"),
        ];

        for (input, expected_session_id, expected_datetime) in test_cases {
            let result = Command::parse(input, "testbot");
            assert!(result.is_ok(), "Failed to parse: {}", input);
            
            match result.unwrap() {
                Command::Deadline { session_id, datetime } => {
                    assert_eq!(session_id, expected_session_id, "Session ID mismatch for input: {}", input);
                    assert_eq!(datetime, expected_datetime, "Datetime mismatch for input: {}", input);
                }
                _ => panic!("Expected Deadline command for input: {}", input),
            }
        }
    }
}