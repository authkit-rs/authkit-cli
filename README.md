# AuthKit CLI

> A CLI tool for AuthKit database schema management, migrations, and visualization.

[![Crates.io](https://img.shields.io/crates/v/authkit-cli.svg)](https://crates.io/crates/authkit-cli)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Overview

AuthKit CLI is a companion tool for the [AuthKit](https://github.com/Akshay2642005/authkit) authentication library. It provides database schema management capabilities including:

- **Migration generation** - Generate SQL migration files for SQLite and PostgreSQL
- **Migration execution** - Apply pending migrations to your database
- **Status checking** - View which migrations have been applied
- **Schema visualization** - Display the current schema in various formats
- **Clean teardown** - Safely destroy all AuthKit tables

## Installation

### From crates.io

```bash
cargo install authkit-cli
```

### From source

```bash
git clone https://github.com/Akshay2642005/authkit-cli
cd authkit-cli
cargo install --path .
```

### Pre-built binaries

Download pre-built binaries from the [GitHub Releases](https://github.com/Akshay2642005/authkit-cli/releases) page.

## Quick Start

```bash
# Apply migrations to a SQLite database
authkit migrate --db-url "sqlite:auth.db"

# Check migration status
authkit status --db-url "sqlite:auth.db"

# Generate migration files
authkit generate --db sqlite --output ./migrations

# View the schema
authkit schema --db sqlite
```

## Commands

### `authkit migrate`

Apply pending migrations to the database.

```bash
authkit migrate --db-url <DATABASE_URL> [--dry-run]
```

**Options:**
- `--db-url <URL>` - Database connection URL (required, or set `AUTHKIT_DATABASE_URL`)
- `--dry-run` - Show what would be executed without applying

**Examples:**
```bash
# SQLite
authkit migrate --db-url "sqlite:auth.db"

# PostgreSQL
authkit migrate --db-url "postgres://user:pass@localhost/authkit"

# Dry run
authkit migrate --db-url "$DATABASE_URL" --dry-run
```

### `authkit status`

Show current migration status.

```bash
authkit status --db-url <DATABASE_URL>
```

**Output:**
```
Database: sqlite:auth.db (SQLite)
Schema Version: 4

┌─────┬─────────────────────────────────┬─────────────────────┬──────────┐
│ #   │ Migration                       │ Applied At          │ Status   │
├─────┼─────────────────────────────────┼─────────────────────┼──────────┤
│ 001 │ create_users_table              │ 2025-01-19 10:30:00 │ Applied  │
│ 002 │ create_sessions_table           │ 2025-01-19 10:30:00 │ Applied  │
│ 003 │ create_tokens_table             │ 2025-01-19 10:30:01 │ Applied  │
│ 004 │ create_indexes                  │ 2025-01-19 10:30:01 │ Applied  │
└─────┴─────────────────────────────────┴─────────────────────┴──────────┘

✓ Database is up to date
```

### `authkit generate`

Generate migration SQL files for a specific database.

```bash
authkit generate --db <sqlite|postgres> [--output <DIR>] [--force]
```

**Options:**
- `--db <TYPE>` - Target database: `sqlite` or `postgres` (required)
- `--output <DIR>` - Output directory (default: `./migrations`)
- `--force` - Overwrite existing files

**Examples:**
```bash
# Generate SQLite migrations
authkit generate --db sqlite

# Generate PostgreSQL migrations to custom directory
authkit generate --db postgres --output ./db/migrations
```

### `authkit destroy`

Drop all AuthKit tables (destructive operation).

```bash
authkit destroy --db-url <DATABASE_URL> [--force]
```

**Options:**
- `--db-url <URL>` - Database connection URL (required)
- `--force` - Skip confirmation prompt

> ⚠️ **Warning:** This command permanently deletes all AuthKit tables and data!

### `authkit schema`

Display the current schema or generate SQL.

```bash
authkit schema [--db <TYPE>] [--format <FORMAT>] [--db-url <URL>]
```

**Options:**
- `--db <TYPE>` - Show schema for: `sqlite` or `postgres`
- `--format <FMT>` - Output format: `sql`, `json`, or `table` (default: `sql`)
- `--db-url <URL>` - Show actual schema from database

**Examples:**
```bash
# Show SQLite schema as SQL
authkit schema --db sqlite

# Show PostgreSQL schema as JSON
authkit schema --db postgres --format json

# Show actual schema from database
authkit schema --db-url "sqlite:auth.db"
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `AUTHKIT_DATABASE_URL` | Default database connection URL |

## Database URL Formats

### SQLite

```
sqlite:path/to/database.db
sqlite::memory:
```

### PostgreSQL

```
postgres://username:password@host:port/database
postgresql://username:password@host/database
```

## Compatibility

| authkit-cli | authkit (library) | Notes |
|-------------|-------------------|-------|
| 0.1.x       | 0.2.x+            | Initial release |

## Development

### Building

```bash
cargo build --release
```

### Running Tests

```bash
# Run SQLite integration tests
cargo test --test integration_sqlite

# Run PostgreSQL integration tests (requires TEST_POSTGRES_URL)
TEST_POSTGRES_URL="postgres://user:pass@localhost/test" cargo test --test integration_postgres -- --ignored
```

### Running Clippy

```bash
cargo clippy -- -D warnings
```

## Safety

This project follows Rust's safety principles:

- **No `unsafe` code** - All code uses safe Rust
- **Proper error handling** - Uses `Result` and `thiserror` for error propagation
- **No panics in production paths** - Uses `?` operator throughout
- **Secure defaults** - Requires confirmation for destructive operations
- **Checksum verification** - Ensures migrations haven't been tampered with

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Related Projects

- [AuthKit](https://github.com/Akshay2642005/authkit) - The core authentication library for Rust