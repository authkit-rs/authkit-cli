use thiserror::Error;

pub type CliResult<T> = Result<T, CliError>;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum CliError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown database type in URL: {0}")]
    UnknownDatabase(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Checksum mismatch for migration {version}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        version: u32,
        expected: String,
        actual: String,
    },

    #[error("File already exists: {0}. Use --force to overwrite.")]
    FileExists(String),

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("{0}")]
    Other(String),
}
