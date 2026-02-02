use std::path::Path;

use colored::Colorize;

use crate::cli::InitArgs;
use crate::config::AuthKitConfig;
use crate::error::{CliError, CliResult};

pub async fn run(args: InitArgs) -> CliResult<()> {
    let config_path = Path::new(&args.output);

    // Check if file already exists
    if config_path.exists() && !args.force {
        return Err(CliError::FileExists(format!(
            "{}. Use --force to overwrite.",
            config_path.display()
        )));
    }

    // Create default config
    let config = AuthKitConfig::default_config(args.db);

    // Create parent directories if needed
    if let Some(parent) = config_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Save config to file
    config.save(config_path)?;

    println!();
    println!("{} Created {}", "✓".green(), config_path.display());
    println!();
    println!("Configuration file created with:");
    println!("  Database: {}", args.db.to_string().cyan());
    println!("  Features:");
    println!("    - {} (base)", "email_password".green());
    println!();
    println!("To enable additional features, edit the config file:");
    println!();
    println!("  [features]");
    println!("  email_password = true");
    println!("  email_verification = true  # Enable this for email verification");
    println!();
    println!("Next steps:");
    println!(
        "  1. Edit {} to enable features",
        config_path.display().to_string().cyan()
    );
    println!(
        "  2. Run {} to generate migrations",
        "authkit generate".cyan()
    );
    println!(
        "  3. Run {} to apply migrations",
        "authkit migrate --db-url <URL>".cyan()
    );

    Ok(())
}
