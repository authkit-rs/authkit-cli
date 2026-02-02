//! Base schema for email/password authentication
//!
//! This is the core schema that provides:
//! - users: Core user data
//! - accounts: Links users to authentication providers
//! - sessions: Active user sessions
//! - verification: Tokens for password reset, magic links, etc.

/// PostgreSQL schema - UP migration
pub const POSTGRES_UP: &str = r#"
-- AuthKit Base Schema
-- Feature: email_password

-- Users table: Core user data
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

-- Accounts table: Links authentication providers to users
-- For email/password, provider = 'credential' and password_hash is set
-- For OAuth (future), provider = 'google'/'github'/etc
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_account_id TEXT NOT NULL,
    password_hash TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE(provider, provider_account_id)
);

-- Sessions table: Active user sessions
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expires_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    ip_address TEXT,
    user_agent TEXT
);

-- Verification table: Tokens for password reset, magic links, etc.
CREATE TABLE IF NOT EXISTS verification (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    identifier TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    token_type TEXT NOT NULL,
    expires_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    used_at BIGINT
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_accounts_user_id ON accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_accounts_provider ON accounts(provider, provider_account_id);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_verification_token_hash ON verification(token_hash);
CREATE INDEX IF NOT EXISTS idx_verification_identifier ON verification(identifier);
CREATE INDEX IF NOT EXISTS idx_verification_expires_at ON verification(expires_at);
"#;

/// PostgreSQL schema - DOWN migration
pub const POSTGRES_DOWN: &str = r#"
-- Drop indexes first
DROP INDEX IF EXISTS idx_verification_expires_at;
DROP INDEX IF EXISTS idx_verification_identifier;
DROP INDEX IF EXISTS idx_verification_token_hash;
DROP INDEX IF EXISTS idx_sessions_expires_at;
DROP INDEX IF EXISTS idx_sessions_token;
DROP INDEX IF EXISTS idx_sessions_user_id;
DROP INDEX IF EXISTS idx_accounts_provider;
DROP INDEX IF EXISTS idx_accounts_user_id;
DROP INDEX IF EXISTS idx_users_email;

-- Drop tables in reverse order (respecting foreign keys)
DROP TABLE IF EXISTS verification;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS accounts;
DROP TABLE IF EXISTS users;
"#;

/// SQLite schema - UP migration
pub const SQLITE_UP: &str = r#"
-- AuthKit Base Schema
-- Feature: email_password

-- Users table: Core user data
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Accounts table: Links authentication providers to users
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_account_id TEXT NOT NULL,
    password_hash TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(provider, provider_account_id)
);

-- Sessions table: Active user sessions
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    ip_address TEXT,
    user_agent TEXT
);

-- Verification table: Tokens for password reset, magic links, etc.
CREATE TABLE IF NOT EXISTS verification (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    identifier TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    token_type TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    used_at INTEGER
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_accounts_user_id ON accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_accounts_provider ON accounts(provider, provider_account_id);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_verification_token_hash ON verification(token_hash);
CREATE INDEX IF NOT EXISTS idx_verification_identifier ON verification(identifier);
CREATE INDEX IF NOT EXISTS idx_verification_expires_at ON verification(expires_at);
"#;

/// SQLite schema - DOWN migration
pub const SQLITE_DOWN: &str = r#"
-- Drop indexes first
DROP INDEX IF EXISTS idx_verification_expires_at;
DROP INDEX IF EXISTS idx_verification_identifier;
DROP INDEX IF EXISTS idx_verification_token_hash;
DROP INDEX IF EXISTS idx_sessions_expires_at;
DROP INDEX IF EXISTS idx_sessions_token;
DROP INDEX IF EXISTS idx_sessions_user_id;
DROP INDEX IF EXISTS idx_accounts_provider;
DROP INDEX IF EXISTS idx_accounts_user_id;
DROP INDEX IF EXISTS idx_users_email;

-- Drop tables in reverse order (respecting foreign keys)
DROP TABLE IF EXISTS verification;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS accounts;
DROP TABLE IF EXISTS users;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_up_contains_all_tables() {
        assert!(POSTGRES_UP.contains("CREATE TABLE IF NOT EXISTS users"));
        assert!(POSTGRES_UP.contains("CREATE TABLE IF NOT EXISTS accounts"));
        assert!(POSTGRES_UP.contains("CREATE TABLE IF NOT EXISTS sessions"));
        assert!(POSTGRES_UP.contains("CREATE TABLE IF NOT EXISTS verification"));
    }

    #[test]
    fn test_sqlite_up_contains_all_tables() {
        assert!(SQLITE_UP.contains("CREATE TABLE IF NOT EXISTS users"));
        assert!(SQLITE_UP.contains("CREATE TABLE IF NOT EXISTS accounts"));
        assert!(SQLITE_UP.contains("CREATE TABLE IF NOT EXISTS sessions"));
        assert!(SQLITE_UP.contains("CREATE TABLE IF NOT EXISTS verification"));
    }

    #[test]
    fn test_down_migrations_drop_all_tables() {
        assert!(POSTGRES_DOWN.contains("DROP TABLE IF EXISTS users"));
        assert!(POSTGRES_DOWN.contains("DROP TABLE IF EXISTS accounts"));
        assert!(POSTGRES_DOWN.contains("DROP TABLE IF EXISTS sessions"));
        assert!(POSTGRES_DOWN.contains("DROP TABLE IF EXISTS verification"));
    }
}
