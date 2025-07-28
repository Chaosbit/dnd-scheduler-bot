//! # DND Scheduler Bot
//! 
//! A Telegram bot for scheduling D&D sessions with polling, reminders, and session management.
//! 
//! ## Features
//! - Schedule D&D sessions with multiple time options
//! - Poll participants for availability 
//! - Automatic reminders (2 weeks, 1 week, 3 days before)
//! - Session confirmation and management
//! - Persistent storage with SQLite

/// Bot command handlers and message processing
pub mod bot;
/// Configuration management and environment variables
pub mod config;
/// Database models, connections, and migrations
pub mod database;
/// Background services like reminders and notifications
pub mod services;
/// Utility functions for datetime, validation, and formatting
pub mod utils;
