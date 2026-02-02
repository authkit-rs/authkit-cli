pub mod runner;

use crate::config::AuthKitConfig;
use crate::schema;

/// A single migration
#[derive(Clone)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up_sql: &'static str,
    pub down_sql: &'static str,
    pub checksum: String,
}

/// A migration that has been applied to the database
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppliedMigration {
    pub version: u32,
    pub name: String,
    pub applied_at: i64,
    pub checksum: String,
}

/// Migration state
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MigrationState {
    /// Migration is available and has been applied
    Applied,
    /// Migration is available but not yet applied
    Pending,
    /// Migration was applied but is no longer in the available list
    Missing,
}

impl MigrationState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Applied => "Applied",
            Self::Pending => "Pending",
            Self::Missing => "Missing",
        }
    }
}

/// Get migrations for enabled features from config
pub fn get_migrations_from_config(config: &AuthKitConfig) -> Vec<Migration> {
    let db_type = config.database_type().expect("Invalid database type");
    let features = config.enabled_features();
    schema::get_migrations_for_features(&features, db_type)
}

/// Compute SHA-256 checksum for migration content
pub fn compute_checksum(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_checksum() {
        let checksum1 = compute_checksum("CREATE TABLE users");
        let checksum2 = compute_checksum("CREATE TABLE users");
        let checksum3 = compute_checksum("CREATE TABLE sessions");

        assert_eq!(checksum1, checksum2);
        assert_ne!(checksum1, checksum3);
        assert_eq!(checksum1.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_migration_state_str() {
        assert_eq!(MigrationState::Applied.as_str(), "Applied");
        assert_eq!(MigrationState::Pending.as_str(), "Pending");
        assert_eq!(MigrationState::Missing.as_str(), "Missing");
    }
}
