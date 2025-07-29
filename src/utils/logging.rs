use tracing::{error, info, warn, debug};

/// Logs command start with consistent format
pub fn log_command_start(command: &str, user: &str, user_id: i64, chat_id: i64, details: Option<&str>) {
    match details {
        Some(d) => info!(
            "CMD_START: {} by {}({}) in chat {} - {}",
            command, user, user_id, chat_id, d
        ),
        None => info!(
            "CMD_START: {} by {}({}) in chat {}",
            command, user, user_id, chat_id
        ),
    }
}

/// Logs command completion with consistent format
pub fn log_command_success(command: &str, user: &str, user_id: i64, chat_id: i64, details: Option<&str>) {
    match details {
        Some(d) => info!(
            "CMD_SUCCESS: {} by {}({}) in chat {} - {}",
            command, user, user_id, chat_id, d
        ),
        None => info!(
            "CMD_SUCCESS: {} by {}({}) in chat {}",
            command, user, user_id, chat_id
        ),
    }
}

/// Logs command errors with consistent format
pub fn log_command_error(command: &str, user: &str, user_id: i64, chat_id: i64, error: &str) {
    error!(
        "CMD_ERROR: {} by {}({}) in chat {} - {}",
        command, user, user_id, chat_id, error
    );
}

/// Logs validation errors with consistent format
pub fn log_validation_error(command: &str, field: &str, value: &str, error: &str, user: &str, user_id: i64, chat_id: i64) {
    warn!(
        "VALIDATION_ERROR: {} - {} field '{}' invalid: {} - user {}({}) in chat {}",
        command, field, value, error, user, user_id, chat_id
    );
}

/// Logs database operations with consistent format
pub fn log_database_operation(operation: &str, table: &str, details: Option<&str>) {
    match details {
        Some(d) => debug!("DB_OP: {} on {} - {}", operation, table, d),
        None => debug!("DB_OP: {} on {}", operation, table),
    }
}

/// Logs database errors with consistent format
pub fn log_database_error(operation: &str, table: &str, error: &str, details: Option<&str>) {
    match details {
        Some(d) => error!("DB_ERROR: {} on {} failed: {} - {}", operation, table, error, d),
        None => error!("DB_ERROR: {} on {} failed: {}", operation, table, error),
    }
}

/// Logs timeout events with consistent format
pub fn log_timeout(operation: &str, duration_secs: u64, details: Option<&str>) {
    match details {
        Some(d) => warn!("TIMEOUT: {} after {}s - {}", operation, duration_secs, d),
        None => warn!("TIMEOUT: {} after {}s", operation, duration_secs),
    }
}

/// Logs system events with consistent format
pub fn log_system_event(event: &str, details: Option<&str>) {
    match details {
        Some(d) => info!("SYSTEM: {} - {}", event, d),
        None => info!("SYSTEM: {}", event),
    }
}