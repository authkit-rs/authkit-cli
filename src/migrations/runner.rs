use sqlx::{AnyPool, Row};
use std::collections::{HashMap, HashSet};

use crate::cli::DatabaseType;
use crate::config::AuthKitConfig;
use crate::error::{CliError, CliResult};
use crate::migrations::{get_migrations_from_config, AppliedMigration, Migration, MigrationState};

/// Migration runner
pub struct MigrationRunner<'a> {
    pool: &'a AnyPool,
    db_type: DatabaseType,
}

impl<'a> MigrationRunner<'a> {
    pub fn new(pool: &'a AnyPool, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }

    /// Ensure the migrations tracking table exists
    pub async fn ensure_migrations_table(&self) -> CliResult<()> {
        let sql = match self.db_type {
            DatabaseType::Sqlite => {
                r#"
                CREATE TABLE IF NOT EXISTS _authkit_migrations (
                    version INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    applied_at INTEGER NOT NULL,
                    checksum TEXT NOT NULL
                )
                "#
            }
            DatabaseType::Postgres => {
                r#"
                CREATE TABLE IF NOT EXISTS _authkit_migrations (
                    version INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    applied_at BIGINT NOT NULL,
                    checksum TEXT NOT NULL
                )
                "#
            }
        };

        sqlx::query(sql).execute(self.pool).await?;
        Ok(())
    }

    /// Get all applied migrations from the database
    pub async fn get_applied_migrations(&self) -> CliResult<Vec<AppliedMigration>> {
        let rows = sqlx::query(
            "SELECT version, name, applied_at, checksum FROM _authkit_migrations ORDER BY version",
        )
        .fetch_all(self.pool)
        .await?;

        let mut migrations = Vec::new();
        for row in rows {
            let version: i32 = row.get("version");
            let name: String = row.get("name");
            let applied_at: i64 = row.get("applied_at");
            let checksum: String = row.get("checksum");

            migrations.push(AppliedMigration {
                version: version as u32,
                name,
                applied_at,
                checksum,
            });
        }

        Ok(migrations)
    }

    /// Get pending migrations based on config
    pub fn get_pending_migrations<'m>(
        &self,
        available: &'m [Migration],
        applied: &[AppliedMigration],
    ) -> Vec<&'m Migration> {
        let applied_versions: HashSet<u32> = applied.iter().map(|m| m.version).collect();

        available
            .iter()
            .filter(|m| !applied_versions.contains(&m.version))
            .collect()
    }

    /// Get migration status
    pub fn get_migration_status(
        &self,
        available: &[Migration],
        applied: &[AppliedMigration],
    ) -> Vec<(u32, String, MigrationState, Option<i64>)> {
        let applied_map: HashMap<u32, &AppliedMigration> =
            applied.iter().map(|m| (m.version, m)).collect();

        let mut statuses = Vec::new();

        // Check all available migrations
        for migration in available {
            if let Some(applied_migration) = applied_map.get(&migration.version) {
                statuses.push((
                    migration.version,
                    migration.name.clone(),
                    MigrationState::Applied,
                    Some(applied_migration.applied_at),
                ));
            } else {
                statuses.push((
                    migration.version,
                    migration.name.clone(),
                    MigrationState::Pending,
                    None,
                ));
            }
        }

        // Check for missing migrations (applied but not in available)
        let available_versions: HashSet<u32> = available.iter().map(|m| m.version).collect();

        for applied_migration in applied {
            if !available_versions.contains(&applied_migration.version) {
                statuses.push((
                    applied_migration.version,
                    applied_migration.name.clone(),
                    MigrationState::Missing,
                    Some(applied_migration.applied_at),
                ));
            }
        }

        statuses.sort_by_key(|(v, _, _, _)| *v);
        statuses
    }

    /// Apply a single migration
    pub async fn apply_migration(&self, migration: &Migration) -> CliResult<()> {
        // Execute each statement individually (important for PostgreSQL)
        for statement in migration.up_sql.split(';') {
            let trimmed = statement.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Strip leading comment lines to get the actual SQL statement
            let sql = Self::strip_leading_comments(trimmed);
            if sql.is_empty() {
                continue;
            }

            sqlx::query(&sql).execute(self.pool).await.map_err(|e| {
                CliError::Migration(format!(
                    "Failed to execute migration {}: {}",
                    migration.name, e
                ))
            })?;
        }

        // Record the migration
        self.record_migration(migration).await?;

        Ok(())
    }

    /// Record a migration in the tracking table
    async fn record_migration(&self, migration: &Migration) -> CliResult<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            "INSERT INTO _authkit_migrations (version, name, applied_at, checksum) VALUES ($1, $2, $3, $4)",
        )
        .bind(migration.version as i32)
        .bind(&migration.name)
        .bind(now)
        .bind(&migration.checksum)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Strip leading comment lines from a SQL statement
    /// Comments start with "--" and continue to end of line
    fn strip_leading_comments(sql: &str) -> String {
        let mut lines: Vec<&str> = Vec::new();
        let mut found_non_comment = false;

        for line in sql.lines() {
            let trimmed_line = line.trim();
            if !found_non_comment {
                // Skip lines that are empty or are comments
                if trimmed_line.is_empty() || trimmed_line.starts_with("--") {
                    continue;
                }
                found_non_comment = true;
            }
            lines.push(line);
        }

        lines.join("\n").trim().to_string()
    }

    /// Run all pending migrations based on config
    #[allow(dead_code)]
    pub async fn run_pending(&self, config: &AuthKitConfig) -> CliResult<Vec<String>> {
        self.ensure_migrations_table().await?;

        let available = get_migrations_from_config(config);
        let applied = self.get_applied_migrations().await?;
        let pending = self.get_pending_migrations(&available, &applied);

        let mut applied_names = Vec::new();

        for migration in pending {
            self.apply_migration(migration).await?;
            applied_names.push(migration.name.clone());
        }

        Ok(applied_names)
    }

    /// Verify checksums of applied migrations
    #[allow(dead_code)]
    pub async fn verify_checksums(&self, config: &AuthKitConfig) -> CliResult<()> {
        let available = get_migrations_from_config(config);
        let applied = self.get_applied_migrations().await?;

        let available_map: HashMap<u32, &Migration> =
            available.iter().map(|m| (m.version, m)).collect();

        for applied_migration in &applied {
            if let Some(migration) = available_map.get(&applied_migration.version) {
                if migration.checksum != applied_migration.checksum {
                    return Err(CliError::ChecksumMismatch {
                        version: applied_migration.version,
                        expected: applied_migration.checksum.clone(),
                        actual: migration.checksum.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Rollback a single migration
    #[allow(dead_code)]
    pub async fn rollback_migration(&self, migration: &Migration) -> CliResult<()> {
        // Execute each statement individually
        for statement in migration.down_sql.split(';') {
            let trimmed = statement.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Strip leading comment lines to get the actual SQL statement
            let sql = Self::strip_leading_comments(trimmed);
            if sql.is_empty() {
                continue;
            }

            sqlx::query(&sql).execute(self.pool).await.map_err(|e| {
                CliError::Migration(format!(
                    "Failed to rollback migration {}: {}",
                    migration.name, e
                ))
            })?;
        }

        // Remove the migration record
        self.remove_migration_record(migration.version).await?;

        Ok(())
    }

    /// Remove a migration record from the tracking table
    #[allow(dead_code)]
    async fn remove_migration_record(&self, version: u32) -> CliResult<()> {
        sqlx::query("DELETE FROM _authkit_migrations WHERE version = $1")
            .bind(version as i32)
            .execute(self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_leading_comments_simple() {
        let sql = "-- This is a comment\nCREATE TABLE users (id TEXT)";
        let result = MigrationRunner::strip_leading_comments(sql);
        assert_eq!(result, "CREATE TABLE users (id TEXT)");
    }

    #[test]
    fn test_strip_leading_comments_multiple_comments() {
        let sql = "-- Comment 1\n-- Comment 2\n-- Comment 3\nCREATE TABLE users (id TEXT)";
        let result = MigrationRunner::strip_leading_comments(sql);
        assert_eq!(result, "CREATE TABLE users (id TEXT)");
    }

    #[test]
    fn test_strip_leading_comments_with_blank_lines() {
        let sql = "-- Comment\n\n-- Another comment\n\nCREATE TABLE users (id TEXT)";
        let result = MigrationRunner::strip_leading_comments(sql);
        assert_eq!(result, "CREATE TABLE users (id TEXT)");
    }

    #[test]
    fn test_strip_leading_comments_no_comments() {
        let sql = "CREATE TABLE users (id TEXT)";
        let result = MigrationRunner::strip_leading_comments(sql);
        assert_eq!(result, "CREATE TABLE users (id TEXT)");
    }

    #[test]
    fn test_strip_leading_comments_only_comments() {
        let sql = "-- Just a comment\n-- Another comment";
        let result = MigrationRunner::strip_leading_comments(sql);
        assert_eq!(result, "");
    }

    #[test]
    fn test_strip_leading_comments_preserves_inline_comments() {
        let sql = "-- Leading comment\nCREATE TABLE users (\n    id TEXT, -- inline comment\n    name TEXT\n)";
        let result = MigrationRunner::strip_leading_comments(sql);
        assert_eq!(
            result,
            "CREATE TABLE users (\n    id TEXT, -- inline comment\n    name TEXT\n)"
        );
    }

    #[test]
    fn test_strip_leading_comments_multiline_statement() {
        let sql = r#"-- Accounts table: Links authentication providers to users
-- For email/password, provider = 'credential' and password_hash is set
-- For OAuth (future), provider = 'google'/'github'/etc
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL
)"#;
        let result = MigrationRunner::strip_leading_comments(sql);
        assert!(result.starts_with("CREATE TABLE IF NOT EXISTS accounts"));
        assert!(result.contains("id TEXT PRIMARY KEY"));
    }
}
