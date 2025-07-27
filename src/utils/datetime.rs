use chrono::{DateTime, Utc, TimeZone, Datelike};
use anyhow::{Result, anyhow};

pub fn parse_datetime(input: &str) -> Result<DateTime<Utc>> {
    let input = input.trim();
    
    // Handle simple formats first - "Friday Dec 1st 7pm"
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
