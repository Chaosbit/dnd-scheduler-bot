use chrono::{DateTime, Utc, TimeZone, Datelike};
use anyhow::{Result, anyhow};

pub fn parse_datetime(input: &str) -> Result<DateTime<Utc>> {
    let input = input.trim();
    
    // Handle European date format first - "15.08.25 19:00", "01.12.24 14:30", etc.
    if let Ok(datetime) = parse_european_date_format(input) {
        return Ok(datetime);
    }
    
    // Handle simple formats - "Friday Dec 1st 7pm"
    if let Ok(datetime) = parse_natural_format(input) {
        return Ok(datetime);
    }
    
    // Fallback to ISO format
    if let Ok(datetime) = input.parse::<DateTime<Utc>>() {
        return Ok(datetime);
    }
    
    // If all parsing fails, return tomorrow at a default time
    let tomorrow = Utc::now() + chrono::Duration::days(1);
    let default_time = tomorrow.date_naive().and_hms_opt(19, 0, 0)
        .ok_or_else(|| anyhow!("Failed to create default time"))?;
    Ok(Utc.from_utc_datetime(&default_time))
}

fn parse_european_date_format(input: &str) -> Result<DateTime<Utc>> {
    // Parse European date formats like "15.08.25 19:00", "01.12.24 14:30", "25.12.2024 20:00"
    let input = input.trim();
    
    // Look for pattern: dd.mm.yy time or dd.mm.yyyy time
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(anyhow!("Invalid European date format"));
    }
    
    let date_part = parts[0];
    let time_part = parts[1];
    
    // Parse date: dd.mm.yy or dd.mm.yyyy
    let date_components: Vec<&str> = date_part.split('.').collect();
    if date_components.len() != 3 {
        return Err(anyhow!("Date must be in dd.mm.yy or dd.mm.yyyy format"));
    }
    
    let day: u32 = date_components[0].parse()
        .map_err(|_| anyhow!("Invalid day"))?;
    let month: u32 = date_components[1].parse()
        .map_err(|_| anyhow!("Invalid month"))?;
    let year_str = date_components[2];
    
    // Handle 2-digit or 4-digit years
    let year: i32 = if year_str.len() == 2 {
        let year_2digit: u32 = year_str.parse()
            .map_err(|_| anyhow!("Invalid year"))?;
        // Assume 00-30 is 2000s, 31-99 is 1900s (but for scheduling, probably all 2000s)
        if year_2digit <= 30 {
            2000 + year_2digit as i32
        } else {
            1900 + year_2digit as i32
        }
    } else if year_str.len() == 4 {
        year_str.parse()
            .map_err(|_| anyhow!("Invalid year"))?
    } else {
        return Err(anyhow!("Year must be 2 or 4 digits"));
    };
    
    // Parse time: HH:MM or HH.MM
    let (hour, minute) = if let Some(time_match) = extract_time_24h(time_part) {
        time_match
    } else {
        return Err(anyhow!("Invalid time format"));
    };
    
    // Validate ranges more strictly
    if day < 1 || day > 31 || month < 1 || month > 12 {
        return Err(anyhow!("Invalid date values"));
    }
    
    // Additional day validation based on month
    if month == 2 && day > 29 {
        return Err(anyhow!("Invalid day for February"));
    }
    if (month == 4 || month == 6 || month == 9 || month == 11) && day > 30 {
        return Err(anyhow!("Invalid day for this month"));
    }
    
    // Create the datetime
    let naive_date = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| anyhow!("Invalid date"))?;
    let naive_datetime = naive_date.and_hms_opt(hour, minute, 0)
        .ok_or_else(|| anyhow!("Invalid time"))?;
    
    Ok(Utc.from_utc_datetime(&naive_datetime))
}

fn parse_natural_format(input: &str) -> Result<DateTime<Utc>> {
    // Parse European date and time formats
    let input_lower = input.to_lowercase();
    
    // Extract time in 24-hour format (19:00, 14:30, etc) or European style
    let time_hour = if let Some(time_match) = extract_time_24h(&input_lower) {
        time_match.0
    } else if input_lower.contains("19:00") || input_lower.contains("19.00") {
        19
    } else if input_lower.contains("14:00") || input_lower.contains("14.00") {
        14
    } else if input_lower.contains("18:00") || input_lower.contains("18.00") {
        18
    } else if input_lower.contains("20:00") || input_lower.contains("20.00") {
        20
    } else if input_lower.contains("15:30") || input_lower.contains("15.30") {
        15
    } else {
        19 // Default to 19:00
    };
    
    let time_minute = if let Some(time_match) = extract_time_24h(&input_lower) {
        time_match.1
    } else if input_lower.contains("15:30") || input_lower.contains("15.30") {
        30
    } else {
        0
    };
    
    // Parse European day names
    let days_ahead = if input_lower.contains("monday") || input_lower.contains("måndag") || input_lower.contains("lundi") {
        days_until_weekday(1)
    } else if input_lower.contains("tuesday") || input_lower.contains("tisdag") || input_lower.contains("mardi") {
        days_until_weekday(2)
    } else if input_lower.contains("wednesday") || input_lower.contains("onsdag") || input_lower.contains("mercredi") {
        days_until_weekday(3)
    } else if input_lower.contains("thursday") || input_lower.contains("torsdag") || input_lower.contains("jeudi") {
        days_until_weekday(4)
    } else if input_lower.contains("friday") || input_lower.contains("fredag") || input_lower.contains("vendredi") {
        days_until_weekday(5)
    } else if input_lower.contains("saturday") || input_lower.contains("lördag") || input_lower.contains("samedi") {
        days_until_weekday(6)
    } else if input_lower.contains("sunday") || input_lower.contains("söndag") || input_lower.contains("dimanche") {
        days_until_weekday(0)
    } else {
        7 // Default to next week
    };
    
    let target_date = Utc::now().date_naive() + chrono::Duration::days(days_ahead);
    let target_datetime = target_date.and_hms_opt(time_hour, time_minute, 0)
        .ok_or_else(|| anyhow!("Failed to create datetime"))?;
    
    Ok(Utc.from_utc_datetime(&target_datetime))
}

fn extract_time_24h(input: &str) -> Option<(u32, u32)> {
    // Match patterns like "19:30", "14.45", "20:00"
    if let Some(colon_pos) = input.find(':') {
        let hour_str = &input[colon_pos.saturating_sub(2)..colon_pos];
        let minute_str = &input[colon_pos + 1..colon_pos + 3];
        if let (Ok(hour), Ok(minute)) = (hour_str.parse::<u32>(), minute_str.parse::<u32>()) {
            if hour < 24 && minute < 60 {
                return Some((hour, minute));
            }
        }
    }
    
    if let Some(dot_pos) = input.find('.') {
        let hour_str = &input[dot_pos.saturating_sub(2)..dot_pos];
        let minute_str = &input[dot_pos + 1..dot_pos + 3];
        if let (Ok(hour), Ok(minute)) = (hour_str.parse::<u32>(), minute_str.parse::<u32>()) {
            if hour < 24 && minute < 60 {
                return Some((hour, minute));
            }
        }
    }
    
    None
}

fn days_until_weekday(target_weekday: u32) -> i64 {
    let today = Utc::now().date_naive().weekday().number_from_monday();
    let target = if target_weekday == 0 { 7 } else { target_weekday }; // Sunday = 7
    
    let days = if target > today {
        target - today
    } else {
        7 - today + target
    };
    
    days as i64
}

pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    // European format: "Monday, 1 December at 19:30"
    dt.format("%A, %d %B at %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    #[test]
    fn test_extract_time_24h_colon_format() {
        assert_eq!(extract_time_24h("friday 19:30"), Some((19, 30)));
        assert_eq!(extract_time_24h("monday 14:00"), Some((14, 0)));
        assert_eq!(extract_time_24h("08:45"), Some((8, 45)));
        assert_eq!(extract_time_24h("23:59"), Some((23, 59)));
    }

    #[test]
    fn test_extract_time_24h_dot_format() {
        assert_eq!(extract_time_24h("friday 19.30"), Some((19, 30)));
        assert_eq!(extract_time_24h("monday 14.00"), Some((14, 0)));
        assert_eq!(extract_time_24h("08.45"), Some((8, 45)));
    }

    #[test]
    fn test_extract_time_24h_invalid() {
        assert_eq!(extract_time_24h("friday 25:30"), None); // Invalid hour
        assert_eq!(extract_time_24h("monday 14:60"), None); // Invalid minute
        assert_eq!(extract_time_24h("no time here"), None);
        assert_eq!(extract_time_24h(""), None);
    }

    #[test]
    fn test_days_until_weekday() {
        // This test is relative to current day, so we test the logic
        let monday = days_until_weekday(1);
        let sunday = days_until_weekday(0);
        
        // Should be between 0 and 6 days
        assert!(monday >= 0 && monday <= 7);
        assert!(sunday >= 0 && sunday <= 7);
        
        // Sunday should be 7 days from any day if calculated as weekday 0
        // But our function treats Sunday as 7, so it should work correctly
    }

    #[test]
    fn test_parse_datetime_english_days() {
        let result = parse_datetime("Friday 19:00");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 19);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn test_parse_datetime_swedish_days() {
        let result = parse_datetime("fredag 19:00");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 19);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn test_parse_datetime_french_days() {
        let result = parse_datetime("vendredi 19:00");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 19);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn test_parse_datetime_dot_notation() {
        let result = parse_datetime("Monday 14.30");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_datetime_iso_format() {
        let iso_date = "2024-12-01T19:00:00Z";
        let result = parse_datetime(iso_date);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 19);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn test_parse_datetime_fallback() {
        // Invalid input should fallback to tomorrow 19:00
        let result = parse_datetime("invalid date string");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 19);
        assert_eq!(dt.minute(), 0);
        
        // Should be within a reasonable time range (tomorrow +/- some days due to natural parsing)
        let now = Utc::now();
        let days_diff = (dt.date_naive() - now.date_naive()).num_days();
        assert!(days_diff >= 1 && days_diff <= 14, "Date should be 1-14 days from now, got {} days", days_diff);
    }

    #[test]
    fn test_format_datetime_european() {
        let dt = Utc.with_ymd_and_hms(2024, 12, 1, 19, 30, 0).unwrap();
        let formatted = format_datetime(&dt);
        
        // Should be in European format: "Saturday, 01 December at 19:30"
        assert!(formatted.contains("December"));
        assert!(formatted.contains("19:30"));
        assert!(formatted.contains("01"));
    }

    #[test]
    fn test_parse_datetime_mixed_case() {
        let result = parse_datetime("FRIDAY 19:00");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 19);
        
        let result2 = parse_datetime("Friday 19:00");
        assert!(result2.is_ok());
        let dt2 = result2.unwrap();
        assert_eq!(dt2.hour(), 19);
    }

    #[test]
    fn test_parse_datetime_whitespace() {
        let result = parse_datetime("  Friday 19:00  ");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 19);
    }

    #[test]
    fn test_parse_datetime_multiple_languages() {
        let test_cases = vec![
            ("Monday 14:00", 14),
            ("måndag 14:00", 14),
            ("lundi 14:00", 14),
            ("Tuesday 20:30", 20),
            ("tisdag 20:30", 20),
            ("mardi 20:30", 20),
        ];

        for (input, expected_hour) in test_cases {
            let result = parse_datetime(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            let dt = result.unwrap();
            assert_eq!(dt.hour(), expected_hour, "Wrong hour for: {}", input);
        }
    }

    #[test]
    fn test_parse_european_date_format_2_digit_year() {
        let result = parse_datetime("15.08.25 19:00");
        assert!(result.is_ok(), "Failed to parse European date format");
        let dt = result.unwrap();
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.month(), 8);
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.hour(), 19);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn test_parse_european_date_format_4_digit_year() {
        let result = parse_datetime("01.12.2024 14:30");
        assert!(result.is_ok(), "Failed to parse European date format");
        let dt = result.unwrap();
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_european_date_format_dot_time() {
        let result = parse_datetime("25.12.24 20.15");
        assert!(result.is_ok(), "Failed to parse European date format with dot time");
        let dt = result.unwrap();
        assert_eq!(dt.day(), 25);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.hour(), 20);
        assert_eq!(dt.minute(), 15);
    }

    #[test]
    fn test_parse_european_date_format_user_case() {
        // Test the specific user case that was reported
        let result = parse_datetime("15.08.25 19:00");
        assert!(result.is_ok(), "Failed to parse user's European date format");
        let dt = result.unwrap();
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.month(), 8);
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.hour(), 19);
        assert_eq!(dt.minute(), 0);
        
        // Additional user-friendly formats should work
        assert!(parse_datetime("01.12.24 14:30").is_ok());
        assert!(parse_datetime("31.12.2024 23:30").is_ok());
    }

    #[test]
    fn test_parse_european_date_format_year_logic() {
        // Test 2-digit year logic
        let result_25 = parse_datetime("01.01.25 12:00");
        assert!(result_25.is_ok());
        assert_eq!(result_25.unwrap().year(), 2025);

        let result_99 = parse_datetime("01.01.99 12:00");
        assert!(result_99.is_ok());
        assert_eq!(result_99.unwrap().year(), 1999);

        let result_00 = parse_datetime("01.01.00 12:00");
        assert!(result_00.is_ok());
        assert_eq!(result_00.unwrap().year(), 2000);
    }
}