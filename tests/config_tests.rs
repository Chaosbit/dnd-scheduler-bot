use dnd_scheduler_bot::config::Config;
use std::env;
use std::sync::Mutex;

// Mutex to ensure config tests run sequentially to avoid environment variable conflicts
static CONFIG_TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_config_from_env_with_all_vars() {
    let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
    
    // Set all environment variables
    env::set_var("TELEGRAM_BOT_TOKEN", "test_token_123");
    env::set_var("DATABASE_URL", "sqlite:test.db");
    env::set_var("HTTP_PORT", "8080");
    
    let config = Config::from_env().unwrap();
    
    assert_eq!(config.telegram_bot_token, "test_token_123");
    assert_eq!(config.database_url, "sqlite:test.db");
    assert_eq!(config.http_port, 8080);
    
    // Clean up
    env::remove_var("TELEGRAM_BOT_TOKEN");
    env::remove_var("DATABASE_URL");
    env::remove_var("HTTP_PORT");
}

#[test]
fn test_config_from_env_with_defaults() {
    let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
    
    // Only set required token, let others use defaults
    env::set_var("TELEGRAM_BOT_TOKEN", "required_token");
    env::remove_var("DATABASE_URL");
    env::remove_var("HTTP_PORT");
    
    let config = Config::from_env().unwrap();
    
    assert_eq!(config.telegram_bot_token, "required_token");
    assert_eq!(config.database_url, "sqlite:./data/scheduler.db");
    assert_eq!(config.http_port, 3000);
    
    // Clean up
    env::remove_var("TELEGRAM_BOT_TOKEN");
}

#[test]
fn test_config_missing_required_token() {
    let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
    
    // Remove the required token
    env::remove_var("TELEGRAM_BOT_TOKEN");
    
    let result = Config::from_env();
    assert!(result.is_err());
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("TELEGRAM_BOT_TOKEN must be set"));
}

#[test]
fn test_config_invalid_port() {
    let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
    
    env::set_var("TELEGRAM_BOT_TOKEN", "test_token");
    env::set_var("HTTP_PORT", "invalid_port");
    
    let result = Config::from_env();
    assert!(result.is_err());
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid HTTP_PORT"));
    
    // Clean up
    env::remove_var("TELEGRAM_BOT_TOKEN");
    env::remove_var("HTTP_PORT");
}

#[test]
fn test_config_port_edge_cases() {
    let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
    
    env::set_var("TELEGRAM_BOT_TOKEN", "test_token");
    
    // Test port 0
    env::set_var("HTTP_PORT", "0");
    let config = Config::from_env().unwrap();
    assert_eq!(config.http_port, 0);
    
    // Test max port
    env::set_var("HTTP_PORT", "65535");
    let config = Config::from_env().unwrap();
    assert_eq!(config.http_port, 65535);
    
    // Test negative port (should fail)
    env::set_var("HTTP_PORT", "-1");
    let result = Config::from_env();
    assert!(result.is_err());
    
    // Clean up
    env::remove_var("TELEGRAM_BOT_TOKEN");
    env::remove_var("HTTP_PORT");
}

#[test]
fn test_config_empty_values() {
    let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
    
    // Test empty token (should fail)
    env::set_var("TELEGRAM_BOT_TOKEN", "");
    let result = Config::from_env();
    assert!(result.is_err());
    
    // Test with valid token and empty database URL (should use default)
    env::set_var("TELEGRAM_BOT_TOKEN", "valid_token");
    env::set_var("DATABASE_URL", "");
    let config = Config::from_env().unwrap();
    assert_eq!(config.database_url, "sqlite:./data/scheduler.db");
    
    // Clean up
    env::remove_var("TELEGRAM_BOT_TOKEN");
    env::remove_var("DATABASE_URL");
}

#[test]
fn test_config_whitespace_handling() {
    let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
    
    env::set_var("TELEGRAM_BOT_TOKEN", "  token_with_spaces  ");
    env::set_var("DATABASE_URL", "  sqlite:spaced.db  ");
    env::set_var("HTTP_PORT", "  3000  ");
    
    let config = Config::from_env().unwrap();
    
    // Environment variables should preserve whitespace as-is
    assert_eq!(config.telegram_bot_token, "  token_with_spaces  ");
    assert_eq!(config.database_url, "  sqlite:spaced.db  ");
    assert_eq!(config.http_port, 3000); // Port parsing should handle whitespace
    
    // Clean up
    env::remove_var("TELEGRAM_BOT_TOKEN");
    env::remove_var("DATABASE_URL");
    env::remove_var("HTTP_PORT");
}