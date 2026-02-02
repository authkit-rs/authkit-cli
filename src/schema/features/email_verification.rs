//! Email Verification feature schema
//!
//! This feature adds email verification support by:
//! - Adding email_verified and email_verified_at columns to users table

/// PostgreSQL schema - UP migration
pub const POSTGRES_UP: &str = r#"
-- AuthKit Email Verification Feature
-- Adds email verification support to users table

-- Add email verification columns to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_verified BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_verified_at BIGINT;

-- Create index for email verification status queries
CREATE INDEX IF NOT EXISTS idx_users_email_verified ON users(email_verified);
"#;

/// PostgreSQL schema - DOWN migration
pub const POSTGRES_DOWN: &str = r#"
-- Remove email verification feature

-- Drop index first
DROP INDEX IF EXISTS idx_users_email_verified;

-- Remove email verification columns from users table
ALTER TABLE users DROP COLUMN IF EXISTS email_verified_at;
ALTER TABLE users DROP COLUMN IF EXISTS email_verified;
"#;

/// SQLite schema - UP migration
/// Note: SQLite has limited ALTER TABLE support, so we use a different approach
pub const SQLITE_UP: &str = r#"
-- AuthKit Email Verification Feature
-- Adds email verification support to users table

-- SQLite: Add email verification columns
-- Note: SQLite 3.35.0+ supports ADD COLUMN, older versions need table recreation
ALTER TABLE users ADD COLUMN email_verified INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN email_verified_at INTEGER;

-- Create index for email verification status queries
CREATE INDEX IF NOT EXISTS idx_users_email_verified ON users(email_verified);
"#;

/// SQLite schema - DOWN migration
pub const SQLITE_DOWN: &str = r#"
-- Remove email verification feature
-- Note: SQLite doesn't support DROP COLUMN in older versions
-- This requires table recreation for full compatibility

-- Drop the index
DROP INDEX IF EXISTS idx_users_email_verified;

-- For SQLite 3.35.0+, we can drop columns directly
-- For older versions, a table recreation would be needed
ALTER TABLE users DROP COLUMN email_verified_at;
ALTER TABLE users DROP COLUMN email_verified;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_up_adds_columns() {
        assert!(POSTGRES_UP.contains("ALTER TABLE users ADD COLUMN"));
        assert!(POSTGRES_UP.contains("email_verified"));
        assert!(POSTGRES_UP.contains("email_verified_at"));
    }

    #[test]
    fn test_postgres_down_removes_columns() {
        assert!(POSTGRES_DOWN.contains("ALTER TABLE users DROP COLUMN"));
        assert!(POSTGRES_DOWN.contains("email_verified"));
    }

    #[test]
    fn test_sqlite_up_adds_columns() {
        assert!(SQLITE_UP.contains("ALTER TABLE users ADD COLUMN"));
        assert!(SQLITE_UP.contains("email_verified"));
    }
}
