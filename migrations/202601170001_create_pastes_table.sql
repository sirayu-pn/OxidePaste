-- Pastes table with support for expiration and password protection
CREATE TABLE IF NOT EXISTS pastes (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    language TEXT DEFAULT 'plaintext',
    password_hash TEXT,
    expires_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    view_count INTEGER DEFAULT 0
);

-- Index for faster expiration queries
CREATE INDEX IF NOT EXISTS idx_pastes_expires_at ON pastes(expires_at);