use assert_cmd::Command;
use predicates::prelude::*;

fn get_test_postgres_url() -> Option<String> {
    std::env::var("TEST_POSTGRES_URL").ok()
}

#[test]
#[ignore] // Run with: cargo test --test integration_postgres -- --ignored
fn test_migrate_postgres() {
    let db_url = match get_test_postgres_url() {
        Some(url) => url,
        None => {
            eprintln!("Skipping: TEST_POSTGRES_URL not set");
            return;
        }
    };

    // First destroy any existing tables
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["destroy", "--db-url", &db_url, "--force"])
        .assert()
        .success();

    // Run migrate
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));
}

#[test]
#[ignore]
fn test_status_postgres() {
    let db_url = match get_test_postgres_url() {
        Some(url) => url,
        None => {
            eprintln!("Skipping: TEST_POSTGRES_URL not set");
            return;
        }
    };

    Command::cargo_bin("authkit")
        .unwrap()
        .args(["status", "--db-url", &db_url])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_migrate_dry_run_postgres() {
    let db_url = match get_test_postgres_url() {
        Some(url) => url,
        None => {
            eprintln!("Skipping: TEST_POSTGRES_URL not set");
            return;
        }
    };

    // First destroy any existing tables
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["destroy", "--db-url", &db_url, "--force"])
        .assert()
        .success();

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
#[ignore]
fn test_destroy_postgres() {
    let db_url = match get_test_postgres_url() {
        Some(url) => url,
        None => {
            eprintln!("Skipping: TEST_POSTGRES_URL not set");
            return;
        }
    };

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
#[ignore]
fn test_idempotent_migrate_postgres() {
    let db_url = match get_test_postgres_url() {
        Some(url) => url,
        None => {
            eprintln!("Skipping: TEST_POSTGRES_URL not set");
            return;
        }
    };

    // First destroy to start fresh
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["destroy", "--db-url", &db_url, "--force"])
        .assert()
        .success();

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
#[ignore]
fn test_full_workflow_postgres() {
    let db_url = match get_test_postgres_url() {
        Some(url) => url,
        None => {
            eprintln!("Skipping: TEST_POSTGRES_URL not set");
            return;
        }
    };

    // 1. Start fresh by destroying existing tables
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["destroy", "--db-url", &db_url, "--force"])
        .assert()
        .success();

    // 2. Check initial status (should show pending)
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["status", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("pending"));

    // 3. Run migrations
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["migrate", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));

    // 4. Check status after migration
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["status", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));

    // 5. Destroy all tables
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["destroy", "--db-url", &db_url, "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("destroyed"));

    // 6. Check status after destroy (should show pending again)
    Command::cargo_bin("authkit")
        .unwrap()
        .args(["status", "--db-url", &db_url])
        .assert()
        .success()
        .stdout(predicate::str::contains("pending"));
}

#[test]
#[ignore]
fn test_schema_from_postgres_database() {
    let db_url = match get_test_postgres_url() {
        Some(url) => url,
        None => {
            eprintln!("Skipping: TEST_POSTGRES_URL not set");
            return;
        }
    };

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
