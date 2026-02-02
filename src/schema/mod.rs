//! Feature-based schema generation
//!
//! This module provides SQL schema for each feature, organized by database type.
//! Migrations are generated per-feature rather than per-table.

pub mod features;

use crate::cli::DatabaseType;
use crate::config::Feature;
use crate::migrations::Migration;

/// Get the migration for a specific feature and database type
pub fn get_feature_migration(feature: Feature, db_type: DatabaseType) -> Migration {
    let (up_sql, down_sql) = match (feature, db_type) {
        // Base (email_password) migrations
        (Feature::EmailPassword, DatabaseType::Postgres) => {
            (features::base::POSTGRES_UP, features::base::POSTGRES_DOWN)
        }
        (Feature::EmailPassword, DatabaseType::Sqlite) => {
            (features::base::SQLITE_UP, features::base::SQLITE_DOWN)
        }

        // Email verification migrations
        (Feature::EmailVerification, DatabaseType::Postgres) => (
            features::email_verification::POSTGRES_UP,
            features::email_verification::POSTGRES_DOWN,
        ),
        (Feature::EmailVerification, DatabaseType::Sqlite) => (
            features::email_verification::SQLITE_UP,
            features::email_verification::SQLITE_DOWN,
        ),
    };

    Migration {
        version: feature.version(),
        name: feature.migration_name().to_string(),
        up_sql,
        down_sql,
        checksum: crate::migrations::compute_checksum(up_sql),
    }
}

/// Get all migrations for the enabled features
pub fn get_migrations_for_features(features: &[Feature], db_type: DatabaseType) -> Vec<Migration> {
    features
        .iter()
        .map(|f| get_feature_migration(*f, db_type))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_migration_postgres() {
        let migration = get_feature_migration(Feature::EmailPassword, DatabaseType::Postgres);
        assert_eq!(migration.version, 1);
        assert_eq!(migration.name, "base");
        assert!(migration.up_sql.contains("CREATE TABLE"));
        assert!(migration.down_sql.contains("DROP TABLE"));
    }

    #[test]
    fn test_email_verification_migration_postgres() {
        let migration = get_feature_migration(Feature::EmailVerification, DatabaseType::Postgres);
        assert_eq!(migration.version, 2);
        assert_eq!(migration.name, "email_verification");
        assert!(migration.up_sql.contains("ALTER TABLE"));
    }

    #[test]
    fn test_migrations_for_features() {
        let features = vec![Feature::EmailPassword, Feature::EmailVerification];
        let migrations = get_migrations_for_features(&features, DatabaseType::Postgres);
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[1].version, 2);
    }
}
