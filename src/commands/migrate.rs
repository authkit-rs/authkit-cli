use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

use crate::cli::MigrateArgs;
use crate::database::Database;
use crate::error::CliResult;
use crate::migrations::{get_migrations, runner::MigrationRunner};

pub async fn run(args: MigrateArgs) -> CliResult<()> {
    println!("Connecting to database...");

    let db = Database::connect(&args.db_url).await?;
    let runner = MigrationRunner::new(&db.pool, db.db_type);

    // Ensure migrations table exists
    runner.ensure_migrations_table().await?;

    // Get migration status
    let available = get_migrations(db.db_type);
    let applied = runner.get_applied_migrations().await?;
    let pending = runner.get_pending_migrations(&available, &applied);

    if pending.is_empty() {
        println!();
        println!("{} Database is already up to date", "✓".green());
        return Ok(());
    }

    println!("Found {} pending migration(s)", pending.len());
    println!();

    if args.dry_run {
        println!("{}", "Dry run - no changes will be made".yellow());
        println!();
        for migration in &pending {
            println!("  Would apply: {:03}_{}", migration.version, migration.name);
        }
        return Ok(());
    }

    // Apply migrations with progress
    let pb = ProgressBar::new(pending.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("#>-"),
    );

    for migration in &pending {
        let migration_name = format!("{:03}_{}", migration.version, migration.name);
        pb.set_message(migration_name.clone());

        let start = Instant::now();
        runner.apply_migration(migration).await?;
        let elapsed = start.elapsed();

        pb.println(format!(
            "  {} {} ({}ms)",
            "Applied".green(),
            migration_name,
            elapsed.as_millis()
        ));
        pb.inc(1);
    }

    pb.finish_and_clear();

    println!();
    println!(
        "{} Applied {} migration(s) successfully",
        "✓".green(),
        pending.len()
    );

    Ok(())
}
