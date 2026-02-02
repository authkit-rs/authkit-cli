CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT 0,
    email_verified_at INTEGER
)
