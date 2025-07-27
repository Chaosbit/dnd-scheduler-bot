use chrono::{DateTime, Utc};
use anyhow::Result;

pub fn parse_datetime(_input: &str) -> Result<DateTime<Utc>> {
    // TODO: Implement flexible date/time parsing
    // For now, return a dummy implementation
    Ok(Utc::now())
}

pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%A, %B %d at %I:%M %p").to_string()
}
