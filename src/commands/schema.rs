use chrono::Utc;
use colored::Colorize;

use crate::cli::{DatabaseType, OutputFormat, SchemaArgs};
use crate::config::AuthKitConfig;
use crate::database::Database;
use crate::error::CliResult;
use crate::migrations::get_migrations_from_config;

pub async fn run(args: SchemaArgs) -> CliResult<()> {
    // If db_url is provided, show actual schema from database
    if let Some(db_url) = &args.db_url {
        return show_actual_schema(db_url, args.format).await;
    }

    // Load configuration if available, otherwise use defaults
    let config = if let Some(ref config_path) = args.config {
        match AuthKitConfig::load(config_path) {
            Ok(config) => config,
            Err(_) => {
                // If config doesn't exist, use defaults with specified db type
                let db_type = args.db.unwrap_or(DatabaseType::Postgres);
                println!(
                    "{} Config not found, using defaults for {}",
                    "Note:".yellow(),
                    db_type.to_string()
                );
                AuthKitConfig::default_config(db_type)
            }
        }
    } else {
        let db_type = args.db.unwrap_or(DatabaseType::Postgres);
        AuthKitConfig::default_config(db_type)
    };

    // Override db type if specified in args
    let db_type = args
        .db
        .unwrap_or_else(|| config.database_type().unwrap_or(DatabaseType::Postgres));

    show_template_schema(&config, db_type, args.format)
}

fn show_template_schema(
    config: &AuthKitConfig,
    db_type: DatabaseType,
    format: OutputFormat,
) -> CliResult<()> {
    let migrations = get_migrations_from_config(config);
    let db_name = match db_type {
        DatabaseType::Sqlite => "SQLite",
        DatabaseType::Postgres => "PostgreSQL",
    };

    let features = config.enabled_features();

    match format {
        OutputFormat::Sql => {
            println!("-- AuthKit Schema for {}", db_name);
            println!("-- Generated: {}", Utc::now().format("%Y-%m-%d"));
            println!("--");
            println!("-- Enabled Features:");
            for feature in &features {
                println!("--   - {}", feature.display_name());
            }
            println!();

            for migration in &migrations {
                println!("-- ============================================================");
                println!(
                    "-- Feature: {} (Migration {:03}_{})",
                    migration.name, migration.version, migration.name
                );
                println!("-- ============================================================");
                println!("{}", migration.up_sql);
                println!();
            }
        }
        OutputFormat::Json => {
            let schema = serde_json::json!({
                "database": db_name,
                "generated_at": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                "features": features.iter().map(|f| {
                    serde_json::json!({
                        "name": f.migration_name(),
                        "display_name": f.display_name(),
                        "version": f.version(),
                    })
                }).collect::<Vec<_>>(),
                "migrations": migrations.iter().map(|m| {
                    serde_json::json!({
                        "version": m.version,
                        "name": m.name,
                        "up_sql": m.up_sql,
                        "down_sql": m.down_sql,
                        "checksum": m.checksum,
                    })
                }).collect::<Vec<_>>(),
            });

            println!(
                "{}",
                serde_json::to_string_pretty(&schema).unwrap_or_default()
            );
        }
        OutputFormat::Table => {
            println!("Schema for {}", db_name.green());
            println!();
            println!("Enabled Features:");
            for feature in &features {
                println!("  {} {}", "✓".green(), feature.display_name());
            }
            println!();

            for migration in &migrations {
                println!(
                    "{} {:03}_{} ({})",
                    "Feature".cyan(),
                    migration.version,
                    migration.name,
                    format!("checksum: {}...", &migration.checksum[..8]).dimmed()
                );
                println!("{}", "─".repeat(60));
                println!("{}", migration.up_sql.trim());
                println!();
            }
        }
    }

    Ok(())
}

async fn show_actual_schema(db_url: &str, format: OutputFormat) -> CliResult<()> {
    let db = Database::connect(db_url).await?;

    let db_type_name = match db.db_type {
        DatabaseType::Sqlite => "SQLite",
        DatabaseType::Postgres => "PostgreSQL",
    };

    // Get table list
    let tables = get_table_list(&db).await?;

    // Get migration status
    let migrations_applied = get_applied_migration_count(&db).await.unwrap_or(0);

    match format {
        OutputFormat::Sql => {
            println!("-- Actual schema from database");
            println!("-- URL: {}", db_url);
            println!("-- Type: {}", db_type_name);
            println!("-- Applied migrations: {}", migrations_applied);
            println!();

            for table in &tables {
                println!("-- Table: {}", table.name);
                if let Some(ref sql) = table.create_sql {
                    println!("{};", sql);
                } else {
                    println!("-- (schema not available)");
                }
                println!();
            }
        }
        OutputFormat::Json => {
            let schema = serde_json::json!({
                "database_url": db_url,
                "database_type": db_type_name,
                "applied_migrations": migrations_applied,
                "tables": tables.iter().map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "create_sql": t.create_sql,
                    })
                }).collect::<Vec<_>>(),
            });

            println!(
                "{}",
                serde_json::to_string_pretty(&schema).unwrap_or_default()
            );
        }
        OutputFormat::Table => {
            println!("Actual schema from: {}", db_url.green());
            println!("Database type: {}", db_type_name.cyan());
            println!("Applied migrations: {}", migrations_applied);
            println!();

            if tables.is_empty() {
                println!("{} No tables found", "!".yellow());
            } else {
                println!("Tables ({}):", tables.len());
                for table in &tables {
                    let is_authkit = table.name.starts_with("_authkit")
                        || ["users", "accounts", "sessions", "verification"]
                            .contains(&table.name.as_str());

                    if is_authkit {
                        println!("  {} {} (AuthKit)", "✓".green(), table.name);
                    } else {
                        println!("  {} {}", "○".dimmed(), table.name);
                    }
                }
            }
        }
    }

    Ok(())
}

struct TableInfo {
    name: String,
    create_sql: Option<String>,
}

async fn get_table_list(db: &Database) -> CliResult<Vec<TableInfo>> {
    let rows: Vec<(String, Option<String>)> = match db.db_type {
        DatabaseType::Sqlite => {
            let query =
                "SELECT name, sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name";
            sqlx::query_as(query).fetch_all(&db.pool).await?
        }
        DatabaseType::Postgres => {
            let query = r#"
                SELECT
                    tablename::text as name,
                    NULL::text as sql
                FROM pg_tables
                WHERE schemaname = 'public'
                ORDER BY tablename
            "#;
            sqlx::query_as(query).fetch_all(&db.pool).await?
        }
    };

    Ok(rows
        .into_iter()
        .map(|(name, create_sql)| TableInfo { name, create_sql })
        .collect())
}

async fn get_applied_migration_count(db: &Database) -> CliResult<i64> {
    // Check if migrations table exists first
    let exists = db.table_exists("_authkit_migrations").await?;
    if !exists {
        return Ok(0);
    }

    let count = db.count_rows("_authkit_migrations").await?;
    Ok(count)
}
