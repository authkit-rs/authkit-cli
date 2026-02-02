use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "authkit")]
#[command(author, version, about = "AuthKit database schema management CLI")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize AuthKit configuration file
    Init(InitArgs),

    /// Generate migration SQL files based on enabled features
    Generate(GenerateArgs),

    /// Apply pending migrations to the database
    Migrate(MigrateArgs),

    /// Show migration status
    Status(StatusArgs),

    /// Drop all AuthKit tables (destructive)
    Destroy(DestroyArgs),

    /// Display current schema
    Schema(SchemaArgs),
}

#[derive(Parser)]
pub struct InitArgs {
    /// Target database type
    #[arg(long, value_enum, default_value = "postgres")]
    pub db: DatabaseType,

    /// Output path for config file
    #[arg(long, default_value = "./authkit.toml")]
    pub output: String,

    /// Overwrite existing config file
    #[arg(long)]
    pub force: bool,
}

#[derive(Parser)]
pub struct GenerateArgs {
    /// Path to authkit.toml config file
    #[arg(long, default_value = "./authkit.toml")]
    pub config: String,

    /// Output directory for migration files
    #[arg(long, default_value = "./migrations")]
    pub output: String,

    /// Overwrite existing files
    #[arg(long)]
    pub force: bool,
}

#[derive(Parser)]
pub struct MigrateArgs {
    /// Database connection URL
    #[arg(long, env = "AUTHKIT_DATABASE_URL")]
    pub db_url: String,

    /// Path to authkit.toml config file
    #[arg(long, default_value = "./authkit.toml")]
    pub config: String,

    /// Show what would be executed without applying
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser)]
pub struct StatusArgs {
    /// Database connection URL
    #[arg(long, env = "AUTHKIT_DATABASE_URL")]
    pub db_url: String,

    /// Path to authkit.toml config file
    #[arg(long, default_value = "./authkit.toml")]
    pub config: String,
}

#[derive(Parser)]
pub struct DestroyArgs {
    /// Database connection URL
    #[arg(long, env = "AUTHKIT_DATABASE_URL")]
    pub db_url: String,

    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,
}

#[derive(Parser)]
pub struct SchemaArgs {
    /// Path to authkit.toml config file
    #[arg(long, default_value = "./authkit.toml")]
    pub config: Option<String>,

    /// Target database type (overrides config)
    #[arg(long, value_enum)]
    pub db: Option<DatabaseType>,

    /// Output format
    #[arg(long, value_enum, default_value = "sql")]
    pub format: OutputFormat,

    /// Database URL (to show actual schema)
    #[arg(long, env = "AUTHKIT_DATABASE_URL")]
    pub db_url: Option<String>,
}

#[derive(Clone, Copy, ValueEnum, Debug, PartialEq, Eq)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::Sqlite => write!(f, "sqlite"),
            DatabaseType::Postgres => write!(f, "postgres"),
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Sql,
    Json,
    Table,
}
