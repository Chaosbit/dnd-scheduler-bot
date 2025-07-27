# D&D Scheduler Bot

A Telegram bot for scheduling D&D sessions with minimal friction and maximum convenience.

## Quick Start

1. **Set up your bot token:**
   ```bash
   cp .env.example .env
   # Edit .env and add your Telegram bot token
   ```

2. **Run with Docker:**
   ```bash
   docker-compose up -d
   ```

3. **Or run locally:**
   ```bash
   cargo run --bin migrate  # Run migrations
   cargo run                # Start the bot
   ```

## Features

- 🎲 Create session polls with multiple time options
- ✅ Simple Yes/No/Maybe responses via inline buttons  
- 📊 Real-time availability tracking
- ⚙️ Group-specific settings and preferences
- 🔔 Reminder notifications
- 📈 Attendance statistics

## Commands

- `/schedule "Session Title" option1, option2, option3` - Create a new session poll
- `/settings` - Configure group preferences
- `/stats` - Show attendance statistics
- `/help` - Show all commands

## Development

Built with Rust using:
- [teloxide](https://github.com/teloxide/teloxide) for Telegram Bot API
- [sqlx](https://github.com/launchbadge/sqlx) for database operations
- [tokio](https://tokio.rs/) for async runtime

See the [project documentation](docs/) for detailed architecture and development guide.
