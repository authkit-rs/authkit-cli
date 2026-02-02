pub mod runner;

use crate::cli::DatabaseType;

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

/// Get all migrations for a database type
pub fn get_migrations(db_type: DatabaseType) -> Vec<Migration> {
    match db_type {
        DatabaseType::Sqlite => sqlite_migrations(),
        DatabaseType::Postgres => postgres_migrations(),
    }
}

fn sqlite_migrations() -> Vec<Migration> {
    let migrations = vec![
        Migration {
            version: 1,
            name: "create_users_table".to_string(),
            up_sql: include_str!("sqlite/001_create_users_table.up.sql"),
            down_sql: include_str!("sqlite/001_create_users_table.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 2,
            name: "create_sessions_table".to_string(),
            up_sql: include_str!("sqlite/002_create_sessions_table.up.sql"),
            down_sql: include_str!("sqlite/002_create_sessions_table.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 3,
            name: "create_tokens_table".to_string(),
            up_sql: include_str!("sqlite/003_create_tokens_table.up.sql"),
            down_sql: include_str!("sqlite/003_create_tokens_table.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 4,
            name: "create_indexes".to_string(),
            up_sql: include_str!("sqlite/004_create_indexes.up.sql"),
            down_sql: include_str!("sqlite/004_create_indexes.down.sql"),
            checksum: String::new(),
        },
    ];

    migrations
        .into_iter()
        .map(|mut m| {
            m.checksum = compute_checksum(m.up_sql);
            m
        })
        .collect()
}

fn postgres_migrations() -> Vec<Migration> {
    let migrations = vec![
        Migration {
            version: 1,
            name: "create_users_table".to_string(),
            up_sql: include_str!("postgres/001_create_users_table.up.sql"),
            down_sql: include_str!("postgres/001_create_users_table.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 2,
            name: "create_sessions_table".to_string(),
            up_sql: include_str!("postgres/002_create_sessions_table.up.sql"),
            down_sql: include_str!("postgres/002_create_sessions_table.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 3,
            name: "create_tokens_table".to_string(),
            up_sql: include_str!("postgres/003_create_tokens_table.up.sql"),
            down_sql: include_str!("postgres/003_create_tokens_table.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 4,
            name: "create_users_email_index".to_string(),
            up_sql: include_str!("postgres/004_create_users_email_index.up.sql"),
            down_sql: include_str!("postgres/004_create_users_email_index.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 5,
            name: "create_sessions_user_id_index".to_string(),
            up_sql: include_str!("postgres/005_create_sessions_user_id_index.up.sql"),
            down_sql: include_str!("postgres/005_create_sessions_user_id_index.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 6,
            name: "create_sessions_expires_at_index".to_string(),
            up_sql: include_str!("postgres/006_create_sessions_expires_at_index.up.sql"),
            down_sql: include_str!("postgres/006_create_sessions_expires_at_index.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 7,
            name: "create_tokens_user_id_index".to_string(),
            up_sql: include_str!("postgres/007_create_tokens_user_id_index.up.sql"),
            down_sql: include_str!("postgres/007_create_tokens_user_id_index.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 8,
            name: "create_tokens_hash_index".to_string(),
            up_sql: include_str!("postgres/008_create_tokens_hash_index.up.sql"),
            down_sql: include_str!("postgres/008_create_tokens_hash_index.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 9,
            name: "create_tokens_expires_at_index".to_string(),
            up_sql: include_str!("postgres/009_create_tokens_expires_at_index.up.sql"),
            down_sql: include_str!("postgres/009_create_tokens_expires_at_index.down.sql"),
            checksum: String::new(),
        },
        Migration {
            version: 10,
            name: "create_tokens_type_index".to_string(),
            up_sql: include_str!("postgres/010_create_tokens_type_index.up.sql"),
            down_sql: include_str!("postgres/010_create_tokens_type_index.down.sql"),
            checksum: String::new(),
        },
    ];

    migrations
        .into_iter()
        .map(|mut m| {
            m.checksum = compute_checksum(m.up_sql);
            m
        })
        .collect()
}

/// Compute SHA-256 checksum for migration content
pub fn compute_checksum(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
