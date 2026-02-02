use chrono::Utc;
use colored::Colorize;

use crate::cli::{DatabaseType, OutputFormat, SchemaArgs};
use crate::database::Database;
use crate::error::CliResult;
use crate::migrations::get_migrations;

pub async fn run(args: SchemaArgs) -> CliResult<()> {
    // If db_url is provided, show actual schema from database
    if let Some(db_url) = &args.db_url {
        return show_actual_schema(db_url, args.format).await;
    }

    // Otherwise show template schema
    let db_type = args.db.unwrap_or(DatabaseType::Sqlite);
    show_template_schema(db_type, args.format)
}

fn show_template_schema(db_type: DatabaseType, format: OutputFormat) -> CliResult<()> {
    let migrations = get_migrations(db_type);
    let db_name = match db_type {
        DatabaseType::Sqlite => "SQLite",
        DatabaseType::Postgres => "PostgreSQL",
    };

    match format {
        OutputFormat::Sql => {
            println!("-- AuthKit Schema for {}", db_name);
            println!("-- Generated: {}", Utc::now().format("%Y-%m-%d"));
            println!();

            for migration in &migrations {
                println!("-- Migration: {:03}_{}", migration.version, migration.name);
                println!("{}", migration.up_sql);
                println!();
            }
        }
        OutputFormat::Json => {
            let schema: Vec<_> = migrations
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "version": m.version,
                        "name": m.name,
                        "up_sql": m.up_sql,
                        "down_sql": m.down_sql,
                        "checksum": m.checksum,
                    })
                })
                .collect();

            println!(
                "{}",
                serde_json::to_string_pretty(&schema).unwrap_or_default()
            );
        }
        OutputFormat::Table => {
            println!("Schema for {}", db_name.green());
            println!();

            for migration in &migrations {
                println!(
                    "{} {:03}_{}",
                    "Migration".cyan(),
                    migration.version,
                    migration.name
                );
                println!("{}", "-".repeat(60));
                println!("{}", migration.up_sql.trim());
                println!();
            }
        }
    }

    Ok(())
}

async fn show_actual_schema(db_url: &str, format: OutputFormat) -> CliResult<()> {
    let db = Database::connect(db_url).await?;

    let query = match db.db_type {
        DatabaseType::Sqlite => {
            "SELECT sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        }
        DatabaseType::Postgres => {
            r#"
            SELECT
                'CREATE TABLE ' || tablename || ' (...);' as sql
            FROM pg_tables
            WHERE schemaname = 'public'
            ORDER BY tablename
            "#
        }
    };

    let rows: Vec<(String,)> = sqlx::query_as(query).fetch_all(&db.pool).await?;

    match format {
        OutputFormat::Sql => {
            println!("-- Actual schema from database");
            println!("-- URL: {}", db_url);
            println!();

            for (sql,) in rows {
                println!("{};", sql);
                println!();
            }
        }
        OutputFormat::Json => {
            let tables: Vec<_> = rows.iter().map(|(sql,)| sql).collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&tables).unwrap_or_default()
            );
        }
        OutputFormat::Table => {
            println!("Actual schema from: {}", db_url.green());
            println!();

            for (sql,) in rows {
                println!("{}", sql);
                println!();
            }
        }
    }

    Ok(())
}
