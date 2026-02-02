# AuthKit CLI

> A CLI tool for AuthKit database schema management with feature-based migrations.

[![Crates.io](https://img.shields.io/crates/v/authkit-cli.svg)](https://crates.io/crates/authkit-cli)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Overview

AuthKit CLI is a companion tool for the [AuthKit](https://github.com/Akshay2642005/authkit) authentication library. It provides a **feature-based** approach to database schema management, similar to better-auth:

- **Feature detection** - Configure which features you need in `authkit.toml`
- **Single migration per feature** - Clean, consolidated migrations instead of many small files
- **Migration generation** - Generate SQL migration files for SQLite and PostgreSQL
- **Migration execution** - Apply pending migrations to your database
- **Status checking** - View which migrations have been applied
- **Schema visualization** - Display the current schema in various formats

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

## Quick Start

```bash
# 1. Initialize configuration
authkit init --db postgres

# 2. Edit authkit.toml to enable features you need

# 3. Generate migrations
authkit generate

# 4. Apply migrations to your database
authkit migrate --db-url "postgres://user:pass@localhost/authkit"

# 5. Check migration status
authkit status --db-url "postgres://user:pass@localhost/authkit"
```

## Configuration

AuthKit CLI uses `authkit.toml` to define which features are enabled:

```toml
[database]
type = "postgres"  # or "sqlite"

[features]
email_password = true       # Base feature (always enabled)
email_verification = true   # Adds email verification support
```

### Available Features

| Feature | Description | Tables/Changes |
|---------|-------------|----------------|
| `email_password` | Base authentication (required) | `users`, `accounts`, `sessions`, `verification` |
| `email_verification` | Email verification support | Adds `email_verified`, `email_verified_at` to `users` |

More features coming soon: OAuth, magic links, two-factor authentication, etc.

## Commands

### `authkit init`

Initialize a new `authkit.toml` configuration file.

```bash
authkit init [--db <sqlite|postgres>] [--output <PATH>] [--force]
```

**Options:**
- `--db <TYPE>` - Target database: `sqlite` or `postgres` (default: `postgres`)
- `--output <PATH>` - Config file path (default: `./authkit.toml`)
- `--force` - Overwrite existing config file

**Example:**
```bash
authkit init --db postgres
```

### `authkit generate`

Generate migration SQL files based on enabled features.

```bash
authkit generate [--config <PATH>] [--output <DIR>] [--force]
```

**Options:**
- `--config <PATH>` - Path to authkit.toml (default: `./authkit.toml`)
- `--output <DIR>` - Output directory (default: `./migrations`)
- `--force` - Overwrite existing files

**Example:**
```bash
authkit generate --output ./db/migrations

# Output:
# Generating postgres migrations to ./db/migrations
#
# Enabled features:
#   ✓ Email/Password Authentication
#   ✓ Email Verification
#
#   Created 001_base.up.sql
#   Created 001_base.down.sql
#   Created 002_email_verification.up.sql
#   Created 002_email_verification.down.sql
#
# ✓ Generated 4 migration files (2 features)
```

### `authkit migrate`

Apply pending migrations to the database.

```bash
authkit migrate --db-url <DATABASE_URL> [--config <PATH>] [--dry-run]
```

**Options:**
- `--db-url <URL>` - Database connection URL (required, or set `AUTHKIT_DATABASE_URL`)
- `--config <PATH>` - Path to authkit.toml (default: `./authkit.toml`)
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
authkit status --db-url <DATABASE_URL> [--config <PATH>]
```

**Example Output:**
```
Configuration: ./authkit.toml

Enabled features:
  ✓ Email/Password Authentication
  ✓ Email Verification

Database: postgres://localhost/authkit (PostgreSQL)
Config Database Type: postgres
Schema Version: 2

┌─────┬────────────────────┬─────────────────────┬──────────┐
│ #   │ Feature            │ Applied At          │ Status   │
├─────┼────────────────────┼─────────────────────┼──────────┤
│ 001 │ base               │ 2025-01-19 10:30:00 │ Applied  │
│ 002 │ email_verification │ 2025-01-19 10:30:01 │ Applied  │
└─────┴────────────────────┴─────────────────────┴──────────┘

✓ Database is up to date
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
authkit schema [--config <PATH>] [--db <TYPE>] [--format <FORMAT>] [--db-url <URL>]
```

**Options:**
- `--config <PATH>` - Path to authkit.toml (optional)
- `--db <TYPE>` - Override database type: `sqlite` or `postgres`
- `--format <FMT>` - Output format: `sql`, `json`, or `table` (default: `sql`)
- `--db-url <URL>` - Show actual schema from database

**Examples:**
```bash
# Show schema based on config
authkit schema

# Show schema as JSON
authkit schema --format json

# Show actual schema from database
authkit schema --db-url "postgres://localhost/authkit"
```

## Database Schema

### Base Schema (email_password feature)

The base schema provides:

| Table | Description |
|-------|-------------|
| `users` | Core user data (id, email, name, timestamps) |
| `accounts` | Links authentication providers to users |
| `sessions` | Active user sessions with metadata |
| `verification` | Tokens for password reset, magic links, etc. |

### Email Verification Feature

Adds to the `users` table:
- `email_verified` (BOOLEAN)
- `email_verified_at` (BIGINT/INTEGER)

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
| 0.2.x       | 0.2.x+            | Feature-based migrations |
| 0.1.x       | 0.1.x             | Legacy table-based migrations |

## Development

### Building

```bash
cargo build --release
```

### Running Tests

```bash
# Run unit tests
cargo test --bin authkit
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

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Related Projects

- [AuthKit](https://github.com/Akshay2642005/authkit) - The core authentication library for Rust