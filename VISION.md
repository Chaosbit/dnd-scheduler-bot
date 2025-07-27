# D&D Session Scheduler Bot

A Telegram bot for scheduling D&D sessions with minimal friction and maximum convenience.

## Overview

This bot helps D&D groups coordinate session scheduling directly within their existing Telegram group chat. No external apps, no account creation, just simple commands and inline responses.

## Architecture

**Single Process Design**
- Rust application with SQLite database
- Telegram Bot API integration
- Docker containerized for easy deployment
- All data stored locally for privacy

**Tech Stack**
- **Runtime**: Rust 1.70+
- **Framework**: Tokio async runtime with Axum (for health checks/webhooks)
- **Database**: SQLite with sqlx for compile-time checked queries
- **Telegram**: teloxide framework with macros for command handling
- **Scheduling**: tokio-cron-scheduler for reminder notifications
- **Logging**: tracing with structured logging
- **Container**: Multi-stage Docker build with Alpine Linux

## Core Features (MVP)

### 1. Session Scheduling
```
/schedule "Next Adventure" 
  - Friday Dec 1st 7pm
  - Saturday Dec 2nd 2pm  
  - Sunday Dec 3rd 6pm
```

### 2. Availability Responses
- Inline keyboard buttons: ✅ Available | ❌ Not Available | ❓ Maybe
- Real-time updates showing who responded
- Visual progress indicator

### 3. Session Management
```
/confirm Friday Dec 1st 7pm
/cancel session_id
/reschedule session_id
```

### 4. Group Settings
```
/settings
  - Set default session length
  - Configure reminder timing
  - Set group timezone
```

## User Stories Implementation

### DM Workflow
1. **Create Poll**: `/schedule "Session Title" date1 time1, date2 time2, date3 time3`
2. **Monitor Responses**: Bot updates message with current availability
3. **Set Deadline**: `/deadline session_id 2023-12-01 18:00`
4. **Confirm Session**: `/confirm session_id option_number`

### Player Workflow  
1. **Respond to Poll**: Click inline buttons on scheduling message
2. **Update Response**: Click different button to change availability
3. **Get Reminders**: Automatic notifications for unresponded polls

### Group Features
1. **Attendance Tracking**: `/stats` shows participation history
2. **Recurring Sessions**: `/recurring weekly friday 19:00`
3. **Quick Reschedule**: `/reschedule session_id` with new date options

## Database Schema

```sql
-- Groups table
CREATE TABLE groups (
  id INTEGER PRIMARY KEY,
  telegram_chat_id INTEGER UNIQUE,
  timezone TEXT DEFAULT 'UTC',
  default_duration INTEGER DEFAULT 240, -- minutes
  reminder_hours INTEGER DEFAULT 24,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Sessions table  
CREATE TABLE sessions (
  id INTEGER PRIMARY KEY,
  group_id INTEGER,
  title TEXT,
  message_id INTEGER,
  status TEXT DEFAULT 'active', -- active, confirmed, cancelled
  deadline DATETIME,
  created_by INTEGER,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (group_id) REFERENCES groups(id)
);

-- Session options (time slots)
CREATE TABLE session_options (
  id INTEGER PRIMARY KEY,
  session_id INTEGER,
  datetime TEXT,
  duration INTEGER, -- minutes
  confirmed BOOLEAN DEFAULT FALSE,
  FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Player responses
CREATE TABLE responses (
  id INTEGER PRIMARY KEY,
  session_id INTEGER,
  option_id INTEGER,
  user_id INTEGER,
  username TEXT,
  response TEXT, -- 'yes', 'no', 'maybe'
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (session_id) REFERENCES sessions(id),
  FOREIGN KEY (option_id) REFERENCES session_options(id)
);
```

## Bot Commands

### Admin Commands (DMs only)
- `/schedule <title> <options>` - Create new session poll
- `/confirm <session_id> <option_number>` - Confirm a session
- `/cancel <session_id>` - Cancel a session
- `/deadline <session_id> <datetime>` - Set response deadline

### Group Commands  
- `/stats` - Show attendance statistics
- `/settings` - Configure group preferences
- `/help` - Show available commands
- `/upcoming` - List confirmed sessions

### Inline Responses
- Availability buttons on each scheduling message
- Real-time updates without refreshing

## Project Structure

```
dnd-scheduler-bot/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── bot/
│   │   ├── mod.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── schedule.rs
│   │   │   ├── confirm.rs
│   │   │   ├── settings.rs
│   │   │   └── stats.rs
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── callback.rs
│   │       └── message.rs
│   ├── database/
│   │   ├── mod.rs
│   │   ├── connection.rs
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── group.rs
│   │       ├── session.rs
│   │       └── response.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── scheduler.rs
│   │   ├── notifications.rs
│   │   └── timezone.rs
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── datetime.rs
│   │   └── validation.rs
│   └── config.rs
├── migrations/
│   ├── 001_initial.sql
│   ├── 002_add_deadlines.sql
│   └── 003_add_recurring.sql
├── Dockerfile
├── docker-compose.yml
├── Cargo.toml
├── .env.example
└── README.md
```

## Configuration

### Environment Variables
```bash
TELEGRAM_BOT_TOKEN=your_bot_token
DATABASE_URL=sqlite:./data/scheduler.db
HTTP_PORT=3000
RUST_LOG=info
```

### Cargo.toml Dependencies
```toml
[package]
name = "dnd-scheduler-bot"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
teloxide = { version = "0.12", features = ["macros", "sqlite-storage"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono", "uuid"] }
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tokio-cron-scheduler = "0.9"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
uuid = { version = "1.0", features = ["v4"] }
dotenvy = "0.15"
```

### Docker Deployment

#### Dockerfile
```dockerfile
# Build stage
FROM rust:1.70-alpine AS builder

# Install dependencies
RUN apk add --no-cache musl-dev sqlite-dev

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy real source and build
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache sqlite ca-certificates

WORKDIR /app

# Copy binary and migrations
COPY --from=builder /app/target/release/dnd-scheduler-bot .
COPY --from=builder /app/migrations ./migrations

# Create data directory
RUN mkdir -p data

EXPOSE 3000

CMD ["./dnd-scheduler-bot"]
```

#### docker-compose.yml
```yaml
version: '3.8'
services:
  dnd-scheduler:
    build: .
    environment:
      - TELEGRAM_BOT_TOKEN=${BOT_TOKEN}
      - DATABASE_URL=sqlite:/app/data/scheduler.db
      - HTTP_PORT=3000
      - RUST_LOG=info
    volumes:
      - ./data:/app/data
    ports:
      - "3000:3000"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## Development Roadmap

### Phase 1: Core MVP
- [x] Basic scheduling commands
- [x] Inline button responses  
- [x] Session confirmation
- [x] SQLite database setup

### Phase 2: Enhanced UX
- [ ] Recurring session templates
- [ ] Attendance statistics
- [ ] Reminder notifications
- [ ] Timezone handling

### Phase 3: Advanced Features
- [ ] Session notes and summaries
- [ ] Player character tracking
- [ ] Campaign management
- [ ] Export to calendar apps

### Phase 4: Integrations
- [ ] Discord bridge (optional)
- [ ] Web dashboard for DMs
- [ ] Mobile app notifications
- [ ] Voice message summaries

## Extension Points

The bot is designed to be easily extensible:

1. **New Commands**: Add files to `src/bot/commands/`
2. **Database Models**: Extend schema in `src/database/models/`
3. **External APIs**: Add services in `src/services/`
4. **Message Handlers**: Customize responses in `src/bot/handlers/`

## Getting Started

### Development Setup
```bash
# Clone and setup
git clone <repository>
cd dnd-scheduler-bot

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Configure environment
cp .env.example .env
# Edit .env with your bot token

# Run database migrations
cargo run --bin migrate

# Run locally with hot reload
cargo install cargo-watch
cargo watch -x run

# Or run normally
cargo run

# Build optimized release
cargo build --release

# Run with Docker
docker-compose up -d

# View logs
docker-compose logs -f dnd-scheduler
```

### Creating a Telegram Bot
1. Message [@BotFather](https://t.me/botfather) on Telegram
2. Use `/newbot` command and follow instructions  
3. Copy the bot token to your `.env` file
4. Add your bot to your D&D group chat
5. Make the bot an admin (needed for message management)

## Contributing

1. Follow the existing code structure
2. Add tests for new features
3. Update this documentation
4. Ensure Docker build succeeds
5. Test with a real Telegram group

## License

MIT License - Feel free to modify and distribute.

---

*Built for D&D groups who want to spend more time playing and less time scheduling.*