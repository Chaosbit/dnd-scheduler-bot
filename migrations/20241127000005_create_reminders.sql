-- Create reminders table for tracking sent session reminders
CREATE TABLE IF NOT EXISTS reminders (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    days_before INTEGER NOT NULL,
    sent_at TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    UNIQUE(session_id, days_before)
);