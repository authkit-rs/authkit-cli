use chrono::{TimeZone, Utc};
use colored::Colorize;
use tabled::{Table, Tabled};

use crate::cli::StatusArgs;
use crate::config::AuthKitConfig;
use crate::database::Database;
use crate::error::CliResult;
use crate::migrations::{get_migrations_from_config, runner::MigrationRunner, MigrationState};

#[derive(Tabled)]
struct MigrationRow {
    #[tabled(rename = "#")]
    version: String,
    #[tabled(rename = "Feature")]
    name: String,
    #[tabled(rename = "Applied At")]
    applied_at: String,
    #[tabled(rename = "Status")]
    status: String,
}

pub async fn run(args: StatusArgs) -> CliResult<()> {
    // Load configuration
    let config = AuthKitConfig::load(&args.config)?;
    let db_type = config.database_type()?;

    println!();
    println!("Configuration: {}", args.config.cyan());
    println!();

    // Show enabled features
    println!("Enabled features:");
    for feature in config.enabled_features() {
        println!("  {} {}", "✓".green(), feature.display_name());
    }
    println!();

    let db = Database::connect(&args.db_url).await?;
    let runner = MigrationRunner::new(&db.pool, db.db_type);

    // Check if migrations table exists
    runner.ensure_migrations_table().await?;

    let available = get_migrations_from_config(&config);
    let applied = runner.get_applied_migrations().await?;
    let statuses = runner.get_migration_status(&available, &applied);

    let db_type_name = match db.db_type {
        crate::cli::DatabaseType::Sqlite => "SQLite",
        crate::cli::DatabaseType::Postgres => "PostgreSQL",
    };

    println!("Database: {} ({})", args.db_url, db_type_name);
    println!("Config Database Type: {}", db_type.to_string().cyan());
    println!(
        "Schema Version: {}",
        applied.last().map(|m| m.version).unwrap_or(0)
    );
    println!();

    if statuses.is_empty() {
        println!(
            "{} No migrations defined for enabled features",
            "!".yellow()
        );
        return Ok(());
    }

    let rows: Vec<MigrationRow> = statuses
        .iter()
        .map(|(version, name, state, applied_at)| {
            let applied_at_str = applied_at
                .map(|ts| {
                    Utc.timestamp_opt(ts, 0)
                        .single()
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "-".to_string())
                })
                .unwrap_or_else(|| "-".to_string());

            let status_str = match state {
                MigrationState::Applied => state.as_str().green().to_string(),
                MigrationState::Pending => state.as_str().yellow().to_string(),
                MigrationState::Missing => state.as_str().red().to_string(),
            };

            MigrationRow {
                version: format!("{:03}", version),
                name: name.clone(),
                applied_at: applied_at_str,
                status: status_str,
            }
        })
        .collect();

    let table = Table::new(rows).to_string();
    println!("{}", table);
    println!();

    let pending_count = statuses
        .iter()
        .filter(|(_, _, state, _)| *state == MigrationState::Pending)
        .count();

    let missing_count = statuses
        .iter()
        .filter(|(_, _, state, _)| *state == MigrationState::Missing)
        .count();

    if pending_count == 0 && missing_count == 0 {
        println!("{} Database is up to date", "✓".green());
    } else {
        if pending_count > 0 {
            println!("{} {} pending migration(s)", "!".yellow(), pending_count);
            println!("  Run {} to apply", "authkit migrate --db-url <URL>".cyan());
        }
        if missing_count > 0 {
            println!(
                "{} {} migration(s) in database not found in config",
                "!".red(),
                missing_count
            );
            println!("  This may indicate features were disabled or migrations were modified");
        }
    }

    Ok(())
}
