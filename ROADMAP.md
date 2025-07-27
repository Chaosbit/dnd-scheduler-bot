# D&D Scheduler Bot - Development Roadmap

## Current Status: Phase 1 - Foundation

### ✅ Completed
- [x] Initial project structure and dependencies
- [x] Basic database models (Group, Session, SessionOption, Response)
- [x] SQLite database setup and migrations
- [x] Fixed sqlx version compatibility with teloxide (downgraded to 0.6)
- [x] Basic command structure and bot handlers
- [x] Database connection management with Clone support
- [x] Documentation (CLAUDE.md, VISION.md, ROADMAP.md)

### 🔄 In Progress
- [ ] **Fix remaining compilation errors** (Current blocker)
  - Group.id Option<i64> vs i64 mismatch
  - Database query result type mismatches
  - Handler error type compatibility
  - Unused import warnings

## Phase 1: Foundation & Core Setup

### High Priority (Immediate)
- [ ] **Complete build fixes**
  - Fix Group.id type handling
  - Resolve sqlx query result type mismatches
  - Fix teloxide handler error types
  
- [ ] **Environment Configuration**
  - Create .env.example file
  - Add dotenvy dependency for environment loading
  - Support TELOXIDE_TOKEN, DATABASE_URL, HTTP_PORT
  
- [ ] **Basic Bot Functionality**
  - Get bot running with basic /help and /start commands
  - Test Telegram connection
  - Verify database operations

### Medium Priority
- [ ] **Migration Binary**
  - Create src/bin/migrate.rs
  - Proper database initialization workflow
  - Migration runner with sqlx

## Phase 2: Core MVP Features

### Schedule Command Enhancement
- [ ] **Improved Date/Time Parsing**
  - Support flexible date formats ("Friday Dec 1st 7pm")
  - Parse multiple time options from command
  - Timezone handling with group settings
  
- [ ] **Enhanced Schedule Command**
  - Parse title and multiple date/time options
  - Create proper inline keyboard layout
  - Store session with parsed datetime options

### Response System
- [ ] **Inline Button Responses**
  - Complete callback handler implementation
  - Support ✅ Available | ❌ Not Available | ❓ Maybe responses
  - Store user responses in database
  
- [ ] **Real-time Message Updates**
  - Update poll messages with current response counts
  - Show who has responded to each option
  - Visual progress indicators

### Session Management
- [ ] **Confirm/Cancel Commands**
  - `/confirm <session_id> <option_number>` implementation
  - `/cancel <session_id>` implementation
  - Update session status in database
  
- [ ] **Deadline Management**
  - `/deadline <session_id> <datetime>` command
  - Automatic deadline enforcement
  - Deadline reminder notifications

## Phase 3: Enhanced Features

### Group Management
- [ ] **Settings Command**
  - `/settings` interface for group configuration
  - Timezone setting
  - Default session duration
  - Reminder timing preferences
  
- [ ] **Statistics & Tracking**
  - `/stats` command for attendance history
  - `/upcoming` command for confirmed sessions
  - User participation tracking

### Notification System
- [ ] **Reminder System**
  - tokio-cron-scheduler integration
  - Automatic reminders for pending responses
  - Deadline notifications
  - Session confirmation reminders

## Phase 4: Infrastructure & Deployment

### Health Monitoring
- [ ] **Axum Health Endpoint**
  - HTTP server for health checks
  - /health endpoint implementation
  - Monitoring integration

### Containerization
- [ ] **Docker Setup**
  - Multi-stage Dockerfile with Alpine Linux
  - docker-compose.yml configuration
  - Volume management for database persistence
  
- [ ] **Deployment Configuration**
  - Environment variable documentation
  - Production deployment guide
  - Backup and recovery procedures

## Phase 5: Advanced Features (Future)

### Extended Functionality
- [ ] **Recurring Sessions**
  - `/recurring weekly friday 19:00` command
  - Template-based session creation
  - Automatic scheduling
  
- [ ] **Session Notes**
  - Add notes to confirmed sessions
  - Session summaries
  - Campaign tracking integration

### Integrations
- [ ] **Calendar Export**
  - iCal format support
  - Google Calendar integration
  - Outlook compatibility
  
- [ ] **External Integrations**
  - Discord bridge (optional)
  - Web dashboard for DMs
  - Mobile app notifications

## Technical Debt & Improvements

### Code Quality
- [ ] **Error Handling**
  - Proper error types throughout codebase
  - User-friendly error messages
  - Logging and monitoring
  
- [ ] **Testing**
  - Unit tests for database models
  - Integration tests for bot commands
  - End-to-end testing with Telegram API
  
- [ ] **Documentation**
  - API documentation
  - Development setup guide
  - Contribution guidelines

### Performance
- [ ] **Database Optimization**
  - Query optimization
  - Connection pooling
  - Database indexes
  
- [ ] **Memory Management**
  - Efficient message caching
  - Response cleanup
  - Resource monitoring

## Dependencies & Technical Notes

### Current Stack
- **teloxide**: 0.12 (Telegram bot framework)
- **sqlx**: 0.6 (Database toolkit, compatible with teloxide)
- **tokio**: 1.0 (Async runtime)
- **axum**: 0.7 (HTTP server for health checks)
- **chrono**: 0.4 (Date/time handling)
- **uuid**: 1.0 (Unique identifiers)
- **dotenvy**: 0.15 (Environment configuration)

### Architecture Decisions
- **Single Process**: All functionality in one Rust binary
- **SQLite**: Local database for simplicity and privacy
- **String-based Dates**: SQLite compatibility over type safety
- **Closure-based DI**: Handler dependency injection pattern

## Release Planning

### v0.1.0 - MVP Release
- Basic scheduling functionality
- Inline response system
- Session confirmation
- Core commands working

### v0.2.0 - Enhanced UX
- Settings management
- Statistics tracking
- Reminder system
- Better date parsing

### v0.3.0 - Production Ready
- Docker deployment
- Health monitoring
- Comprehensive testing
- Documentation complete

### v1.0.0 - Feature Complete
- All core features implemented
- Stable API
- Production deployments
- Community feedback integrated

---

## Current Focus

**Immediate Goal**: Get the bot compiling and running with basic functionality
**Next Milestone**: Complete Phase 1 foundation and move to core MVP features
**Success Criteria**: Users can create session polls and respond with inline buttons

**Last Updated**: 2025-01-27
**Current Phase**: 1 (Foundation)
**Progress**: ~60% complete