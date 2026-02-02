use crate::cli::DatabaseType;
use crate::error::{CliError, CliResult};
use sqlx::{AnyPool, Row};

pub struct Database {
    pub pool: AnyPool,
    pub db_type: DatabaseType,
}

impl Database {
    /// Connect to database from URL
    pub async fn connect(url: &str) -> CliResult<Self> {
        let db_type = Self::detect_type(url)?;

        // Install the appropriate driver
        sqlx::any::install_default_drivers();

        let pool = AnyPool::connect(url).await?;

        Ok(Self { pool, db_type })
    }

    /// Detect database type from URL
    pub fn detect_type(url: &str) -> CliResult<DatabaseType> {
        if url.starts_with("sqlite:") {
            Ok(DatabaseType::Sqlite)
        } else if url.starts_with("postgres:") || url.starts_with("postgresql:") {
            Ok(DatabaseType::Postgres)
        } else {
            Err(CliError::UnknownDatabase(url.to_string()))
        }
    }

    /// Get row count for a table
    pub async fn count_rows(&self, table: &str) -> CliResult<i64> {
        let query = format!("SELECT COUNT(*) as count FROM {}", table);
        let row = sqlx::query(&query).fetch_one(&self.pool).await?;
        let count: i64 = row.get("count");
        Ok(count)
    }

    /// Check if a table exists
    pub async fn table_exists(&self, table: &str) -> CliResult<bool> {
        let result = match self.db_type {
            DatabaseType::Sqlite => {
                let query = "SELECT name FROM sqlite_master WHERE type='table' AND name = $1";
                sqlx::query(query)
                    .bind(table)
                    .fetch_optional(&self.pool)
                    .await?
            }
            DatabaseType::Postgres => {
                let query = "SELECT tablename FROM pg_tables WHERE tablename = $1";
                sqlx::query(query)
                    .bind(table)
                    .fetch_optional(&self.pool)
                    .await?
            }
        };

        Ok(result.is_some())
    }

    /// Drop a table
    pub async fn drop_table(&self, table: &str) -> CliResult<()> {
        // Note: We can't use bind for table names, but these are hardcoded constants
        let query = match self.db_type {
            DatabaseType::Sqlite => format!("DROP TABLE IF EXISTS {}", table),
            DatabaseType::Postgres => format!("DROP TABLE IF EXISTS {} CASCADE", table),
        };
        sqlx::query(&query).execute(&self.pool).await?;
        Ok(())
    }
}
