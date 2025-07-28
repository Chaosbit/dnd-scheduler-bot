-- Add database indexes for better query performance
-- These indexes optimize the most common query patterns in the application

-- Index for finding session options by session_id (used in list command and callback handler)
CREATE INDEX IF NOT EXISTS idx_session_options_session_id ON session_options(session_id);

-- Index for finding responses by session_id (used in list command and callback handler)  
CREATE INDEX IF NOT EXISTS idx_responses_session_id ON responses(session_id);

-- Composite index for responses by session_id and option_id (used for vote counting)
CREATE INDEX IF NOT EXISTS idx_responses_session_option ON responses(session_id, option_id);

-- Index for finding responses by user_id (useful for user activity tracking)
CREATE INDEX IF NOT EXISTS idx_responses_user_id ON responses(user_id);

-- Index for finding sessions by group_id (used in list command)
CREATE INDEX IF NOT EXISTS idx_sessions_group_id ON sessions(group_id);

-- Index for finding sessions by status (used to filter active/confirmed sessions)
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);

-- Composite index for sessions by group_id and status (optimizes list command query)
CREATE INDEX IF NOT EXISTS idx_sessions_group_status ON sessions(group_id, status);

-- Index for finding groups by telegram_chat_id (used in all commands)
CREATE INDEX IF NOT EXISTS idx_groups_telegram_chat_id ON groups(telegram_chat_id);

-- Index for finding reminders by session_id (used in reminder service)
CREATE INDEX IF NOT EXISTS idx_reminders_session_id ON reminders(session_id);

-- Composite index for checking if reminder already sent
CREATE INDEX IF NOT EXISTS idx_reminders_session_days ON reminders(session_id, days_before);