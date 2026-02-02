use std::fs;
use std::path::Path;

use colored::Colorize;

use crate::cli::GenerateArgs;
use crate::config::AuthKitConfig;
use crate::error::{CliError, CliResult};
use crate::migrations::get_migrations_from_config;

pub async fn run(args: GenerateArgs) -> CliResult<()> {
    // Load configuration
    let config = AuthKitConfig::load(&args.config)?;
    let db_type = config.database_type()?;

    let db_name = db_type.to_string();
    let migrations = get_migrations_from_config(&config);

    if migrations.is_empty() {
        println!("{} No features enabled. Nothing to generate.", "!".yellow());
        return Ok(());
    }

    let output_dir = Path::new(&args.output);

    // Create output directory
    fs::create_dir_all(output_dir)?;

    println!(
        "Generating {} migrations to {}",
        db_name,
        output_dir.display()
    );
    println!();

    // Show enabled features
    println!("Enabled features:");
    for feature in config.enabled_features() {
        println!("  {} {}", "✓".green(), feature.display_name());
    }
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
        "{} Generated {} migration files ({} features)",
        "✓".green(),
        migrations.len() * 2,
        migrations.len()
    );
    println!();
    println!("Next steps:");
    println!(
        "  Run {} to apply migrations",
        "authkit migrate --db-url <URL>".cyan()
    );

    Ok(())
}
