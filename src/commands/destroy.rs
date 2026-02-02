use colored::Colorize;
use dialoguer::Confirm;

use crate::cli::DestroyArgs;
use crate::database::Database;
use crate::error::{CliError, CliResult};

const AUTHKIT_TABLES: &[&str] = &["tokens", "sessions", "users", "_authkit_migrations"];

pub async fn run(args: DestroyArgs) -> CliResult<()> {
    let db = Database::connect(&args.db_url).await?;

    println!();
    println!(
        "{}",
        "⚠️  WARNING: This will permanently delete all AuthKit tables and data!"
            .red()
            .bold()
    );
    println!();

    // Show tables and row counts
    println!("Tables to be dropped:");
    let mut tables_to_drop = Vec::new();

    for table in AUTHKIT_TABLES {
        if db.table_exists(table).await? {
            let count = db.count_rows(table).await.unwrap_or(0);
            println!("  - {} ({} rows)", table, count);
            tables_to_drop.push(*table);
        }
    }

    if tables_to_drop.is_empty() {
        println!("  (no AuthKit tables found)");
        println!();
        println!("{} Nothing to destroy", "✓".green());
        return Ok(());
    }

    println!();

    // Confirm unless --force
    if !args.force {
        let confirmed = Confirm::new()
            .with_prompt("Are you sure you want to destroy all tables?")
            .default(false)
            .interact()
            .map_err(|_| CliError::Cancelled)?;

        if !confirmed {
            println!();
            println!("Operation cancelled");
            return Ok(());
        }
    }

    println!();

    // Drop tables in order (respecting foreign keys)
    for table in &tables_to_drop {
        print!("Dropping {}... ", table);
        db.drop_table(table).await?;
        println!("{}", "done".green());
    }

    println!();
    println!("{} All AuthKit tables destroyed", "✓".green());

    Ok(())
}
