use std::fs;
use std::path::Path;

use colored::Colorize;

use crate::cli::GenerateArgs;
use crate::error::{CliError, CliResult};
use crate::migrations::get_migrations;

pub async fn run(args: GenerateArgs) -> CliResult<()> {
    let migrations = get_migrations(args.db);
    let db_name = match args.db {
        crate::cli::DatabaseType::Sqlite => "sqlite",
        crate::cli::DatabaseType::Postgres => "postgres",
    };

    let output_dir = Path::new(&args.output).join(db_name);

    // Create output directory
    fs::create_dir_all(&output_dir)?;

    println!(
        "Generating {} migrations to {}",
        db_name,
        output_dir.display()
    );
    println!();

    for migration in &migrations {
        let up_filename = format!("{:03}_{}.up.sql", migration.version, migration.name);
        let down_filename = format!("{:03}_{}.down.sql", migration.version, migration.name);

        let up_path = output_dir.join(&up_filename);
        let down_path = output_dir.join(&down_filename);

        // Check if files exist
        if !args.force {
            if up_path.exists() {
                return Err(CliError::FileExists(up_path.display().to_string()));
            }
            if down_path.exists() {
                return Err(CliError::FileExists(down_path.display().to_string()));
            }
        }

        // Write files
        fs::write(&up_path, migration.up_sql)?;
        fs::write(&down_path, migration.down_sql)?;

        println!("  {} {}", "Created".green(), up_filename);
        println!("  {} {}", "Created".green(), down_filename);
    }

    println!();
    println!(
        "{} Generated {} migration files",
        "✓".green(),
        migrations.len() * 2
    );

    Ok(())
}
