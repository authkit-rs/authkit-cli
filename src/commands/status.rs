use chrono::{TimeZone, Utc};
use colored::Colorize;
use tabled::{Table, Tabled};

use crate::cli::StatusArgs;
use crate::database::Database;
use crate::error::CliResult;
use crate::migrations::{get_migrations, runner::MigrationRunner, MigrationState};

#[derive(Tabled)]
struct MigrationRow {
    #[tabled(rename = "#")]
    version: String,
    #[tabled(rename = "Migration")]
    name: String,
    #[tabled(rename = "Applied At")]
    applied_at: String,
    #[tabled(rename = "Status")]
    status: String,
}

pub async fn run(args: StatusArgs) -> CliResult<()> {
    let db = Database::connect(&args.db_url).await?;
    let runner = MigrationRunner::new(&db.pool, db.db_type);

    // Check if migrations table exists
    runner.ensure_migrations_table().await?;

    let available = get_migrations(db.db_type);
    let applied = runner.get_applied_migrations().await?;
    let statuses = runner.get_migration_status(&available, &applied);

    let db_type_name = match db.db_type {
        crate::cli::DatabaseType::Sqlite => "SQLite",
        crate::cli::DatabaseType::Postgres => "PostgreSQL",
    };

    println!();
    println!("Database: {} ({})", args.db_url, db_type_name);
    println!(
        "Schema Version: {}",
        applied.last().map(|m| m.version).unwrap_or(0)
    );
    println!();

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

    if pending_count == 0 {
        println!("{} Database is up to date", "✓".green());
    } else {
        println!("{} {} pending migration(s)", "!".yellow(), pending_count);
    }

    Ok(())
}
