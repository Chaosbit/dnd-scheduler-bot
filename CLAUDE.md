# Claude Code Assistant - D&D Scheduler Bot

## Project Overview
A Telegram bot written in Rust for scheduling D&D sessions. The bot allows users to create session polls with multiple time options and collect responses from group members.

## Build Instructions

### Prerequisites
- Rust toolchain
- SQLite3
- Environment variables

### Build Process
1. Set the DATABASE_URL environment variable:
   ```bash
   export DATABASE_URL="sqlite:/home/paul/Repos/dnd-scheduler-bot/data/bot.db"
   ```

2. Create and initialize the database:
   ```bash
   mkdir -p data
   sqlite3 data/bot.db < migrations/001_initial.sql
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run the project:
   ```bash
   cargo run
   ```

### Test Commands
- `cargo test` - Run unit tests
- `cargo check` - Quick syntax check

## Key Dependencies
- `teloxide` 0.12 - Telegram bot framework
- `sqlx` 0.6 - Database toolkit (compatible with teloxide)
- `tokio` - Async runtime
- `chrono` - Date/time handling
- `uuid` - Unique identifier generation

## Database Schema
- `groups` - Telegram chat groups with settings
- `sessions` - D&D session polls
- `session_options` - Time/date options for sessions
- `responses` - User responses to session options

## Known Issues
- SQLite compatibility requires string storage for dates instead of DateTime types
- Database models use String for created_at fields, convert to/from RFC3339 format
- Dependency injection for handlers uses closure approach for database access

## Architecture
```
src/
├── bot/
│   ├── commands/     # Bot command handlers
│   └── handlers/     # Message and callback handlers
├── database/
│   ├── models/       # Database model structs
│   └── connection.rs # Database connection management
├── services/         # Business logic services
└── utils/           # Utility functions
```

## Environment Variables
- `DATABASE_URL` - SQLite database file path (required for build)
- `TELOXIDE_TOKEN` - Telegram bot token (required for runtime)

## Git Workflow Guidelines
Please use clean git flow when working on this project:
1. Create feature branches for new work: `git checkout -b feature/description`
2. Make focused commits with descriptive messages
3. Use conventional commit format when possible: `feat:`, `fix:`, `test:`, `refactor:`
4. Keep commits atomic - one logical change per commit
5. Test before committing: run `cargo test` and `cargo check`
6. Add comprehensive commit messages explaining what and why

## Recent Fixes Applied
1. Fixed sqlx version compatibility with teloxide (downgraded from 0.7 to 0.6)
2. Resolved database model type mismatches for SQLite compatibility
3. Added Clone derive to DatabaseManager for dependency injection
4. Fixed handler dependency injection using closure approach
5. Removed RETURNING clauses for SQLite compatibility
6. Added comprehensive feedback system with progress tracking and user-friendly messages
7. Created extensive test coverage for command parsing, validation, and feedback systems