-- Initial database schema for D&D Scheduler Bot

CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    telegram_chat_id INTEGER UNIQUE NOT NULL,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    default_duration INTEGER NOT NULL DEFAULT 240, -- minutes
    reminder_hours INTEGER NOT NULL DEFAULT 24,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    group_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    message_id INTEGER,
    status TEXT NOT NULL DEFAULT 'active', -- active, confirmed, cancelled
    deadline TEXT,
    created_by INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS session_options (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    datetime TEXT NOT NULL,
    duration INTEGER NOT NULL DEFAULT 240, -- minutes
    confirmed BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS responses (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    option_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    username TEXT,
    response TEXT NOT NULL, -- 'yes', 'no', 'maybe'
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (option_id) REFERENCES session_options(id) ON DELETE CASCADE,
    UNIQUE(session_id, option_id, user_id)
);

-- Indexes for better performance
CREATE INDEX IF NOT EXISTS idx_groups_chat_id ON groups(telegram_chat_id);
CREATE INDEX IF NOT EXISTS idx_sessions_group_id ON sessions(group_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_session_options_session_id ON session_options(session_id);
CREATE INDEX IF NOT EXISTS idx_responses_session_id ON responses(session_id);
CREATE INDEX IF NOT EXISTS idx_responses_user_id ON responses(user_id);
