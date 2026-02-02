use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_help_command() {
    Command::cargo_bin("authkit")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "AuthKit database schema management CLI",
        ));
}

#[test]
fn test_version_command() {
    Command::cargo_bin("authkit")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn test_migrate_sqlite() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // Run migrate
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));
}

#[test]
fn test_migrate_dry_run() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // Run migrate with dry-run
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", &db_url, "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"))
        .stdout(predicate::str::contains("Would apply"));
}

#[test]
fn test_status_after_migrate() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // First migrate
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", &db_url])
        .assert()
        .success();

    // Then check status
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["status", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));
}

#[test]
fn test_status_shows_pending() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // Check status on empty database (will create migrations table)
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["status", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("pending"));
}

#[test]
fn test_destroy_with_force() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // First migrate
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", &db_url])
        .assert()
        .success();

    // Then destroy
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["destroy", "--db-url", &db_url, "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("destroyed"));
}

#[test]
fn test_destroy_nothing_to_destroy() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // Try to destroy on empty database
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["destroy", "--db-url", &db_url, "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing to destroy"));
}

#[test]
fn test_generate_sqlite() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("migrations");

    Command::cargo_bin("authkit")
        .unwrap()
        .args([
            "generate",
            "--db",
            "sqlite",
            "--output",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));

    // Verify files were created
    assert!(output_dir
        .join("sqlite")
        .join("001_create_users_table.up.sql")
        .exists());
    assert!(output_dir
        .join("sqlite")
        .join("001_create_users_table.down.sql")
        .exists());
    assert!(output_dir
        .join("sqlite")
        .join("004_create_indexes.up.sql")
        .exists());
}

#[test]
fn test_generate_postgres() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("migrations");

    Command::cargo_bin("authkit")
        .unwrap()
        .args([
            "generate",
            "--db",
            "postgres",
            "--output",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));

    // Verify files were created (PostgreSQL has more files due to individual indexes)
    assert!(output_dir
        .join("postgres")
        .join("001_create_users_table.up.sql")
        .exists());
    assert!(output_dir
        .join("postgres")
        .join("010_create_tokens_type_index.up.sql")
        .exists());
}

#[test]
fn test_generate_file_exists_error() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("migrations");

    // First generate
    Command::cargo_bin("authkit")
        .unwrap()
        .args([
            "generate",
            "--db",
            "sqlite",
            "--output",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Second generate should fail without --force
    Command::cargo_bin("authkit")
        .unwrap()
        .args([
            "generate",
            "--db",
            "sqlite",
            "--output",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("FileExists"));
}

#[test]
fn test_generate_force_overwrites() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("migrations");

    // First generate
    Command::cargo_bin("authkit")
        .unwrap()
        .args([
            "generate",
            "--db",
            "sqlite",
            "--output",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Second generate with --force should succeed
    Command::cargo_bin("authkit")
        .unwrap()
        .args([
            "generate",
            "--db",
            "sqlite",
            "--output",
            output_dir.to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();
}

#[test]
fn test_idempotent_migrate() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // Run migrate twice
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", &db_url])
        .assert()
        .success();

    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));
}

#[test]
fn test_schema_sql_output() {
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["schema", "--db", "sqlite", "--format", "sql"])
        .assert()
        .success()
        .stdout(predicate::str::contains("AuthKit Schema for SQLite"))
        .stdout(predicate::str::contains("CREATE TABLE"));
}

#[test]
fn test_schema_json_output() {
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["schema", "--db", "sqlite", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"version\""))
        .stdout(predicate::str::contains("\"name\""));
}

#[test]
fn test_schema_table_output() {
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["schema", "--db", "sqlite", "--format", "table"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Migration"));
}

#[test]
fn test_schema_from_database() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // First migrate to create tables
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", &db_url])
        .assert()
        .success();

    // Then get schema from database
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["schema", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("Actual schema from database"));
}

#[test]
fn test_invalid_database_url() {
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", "invalid://something"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("UnknownDatabase"));
}

#[test]
fn test_full_workflow() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("workflow_test.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // 1. Check initial status (should show pending)
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["status", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("pending"));

    // 2. Run migrations
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));

    // 3. Check status after migration
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["status", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));

    // 4. Destroy all tables
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["destroy", "--db-url", &db_url, "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("destroyed"));

    // 5. Check status after destroy (should show pending again)
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["status", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("pending"));
}
