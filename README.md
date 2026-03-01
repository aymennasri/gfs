![Guepard](/resources/guepard-cover.png)

<div align="center">
    <h1>Git For database Systems</h1>
    <br />  
    <p align="center">
    <a href="https://youtu.be/WlOkLnoY2h8?si=hb6-7kLhlOvVL1u6">
        <img src="https://img.shields.io/badge/Watch-YouTube-%23ffcb51?logo=youtube&logoColor=black" alt="Watch on YouTube" />
    </a>
    <a href="https://discord.gg/SEdZuJbc5V">
        <img src="https://img.shields.io/badge/Join-Community-%23ffcb51?logo=discord&logoColor=black" alt="Join our Community" />
    </a>
    <a href="https://github.com/Guepard-Corp/gfs/actions/workflows/main.yml" target="_blank">
        <img src="https://img.shields.io/github/actions/workflow/status/Guepard-Corp/gfs/main.yml?branch=main" alt="Build">
    </a>
    <a href="https://github.com/Guepard-Corp/gfs/blob/main/LICENCE" target="_blank">
        <img src="https://img.shields.io/badge/license-ELv2-blue.svg" alt="License" />
    </a>
    <a href="https://github.com/Guepard-Corp/gfs/pulls" target="_blank">
        <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg" alt="PRs Welcome" />
    </a>
    </p>
    <img src="resources/GFSShowcase.gif" alt="GFS Showcase" />
</div>

## Table of Contents

- [Important Notice](#important-notice)
- [What is GFS?](#what-is-gfs)
- [Supported Databases](#supported-databases)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
- [MCP Server](#mcp-server)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)
- [Development](#development)
- [Contributing](#contributing)
- [Community](#community)
- [Roadmap](#roadmap)
- [License](#license)

## Important Notice

🚧 This project is under active development and not yet suitable for production use. Expect breaking changes, incomplete features, and evolving APIs.

## What is GFS?

GFS (Git For database Systems) brings Git-like version control to your databases. It enables you to:

- **Commit** database states with meaningful messages
- **Branch** and **merge** database schemas and data
- **Time travel** through your database history
- **Collaborate** on database changes with confidence
- **Rollback** to any previous state instantly

GFS uses Docker to manage isolated database environments, making it easy to work with different versions of your database without conflicts.

## Supported Databases

- **PostgreSQL** (versions 13-18)
- **MySQL** (versions 8.0-8.1)

Run `gfs providers` to see all available providers and their supported versions.

## Features

- ✅ Initialize database repositories
- ✅ Commit database changes
- ✅ View commit history
- ✅ Checkout previous commits
- ✅ Create and switch branches
- ✅ Check database status
- ✅ Query database directly from CLI (SQL execution and interactive mode)
- ✅ Schema extraction, show, and diff between commits
- ✅ Export and import data (SQL, custom, CSV)
- ✅ Compute container management (start, stop, logs)
- ✅ Repository config (user.name, user.email)

## Installation

```bash
curl -fsSL https://gfs.guepard.run/install | bash
```

## Quick Start

### 1. Check available database providers

```bash
gfs providers
```

This shows all supported database providers and their versions.

### 2. Create a new project directory

```bash
mkdir my_project
cd my_project
```

### 3. Initialize the repository

```bash
gfs init --database-provider postgres --database-version 17
```

This creates a `.gfs` directory and starts a PostgreSQL database in a Docker container.

### 4. Check status

```bash
gfs status
```

This shows the current state of your storage and compute resources.

### 5. Query your database

```bash
# Execute a SQL query directly
gfs query "SELECT 1"

# Or open an interactive terminal session
gfs query
```

### 6. Make changes and commit

[Example] Create table users

```bash
gfs query "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT NOT NULL);"
```

[Example] Add data to users

```bash
gfs query "INSERT INTO users (name) VALUES ('Alice'), ('Bob');"
```

After modifying your database schema or data:

```bash
gfs commit -m "my first commit"
```

Output: `[main] 88b0ff8  my first commit`

### 7. View commit history

```bash
gfs log
```

### 8. Update the schema

[Example] Add `transactions` table with foreign key to `users`:

```bash
gfs query "CREATE TABLE transactions (id SERIAL PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id), amount NUMERIC NOT NULL, created_at TIMESTAMP DEFAULT NOW());"
```

[Example] Add data to `transactions`:

```bash
gfs query "INSERT INTO transactions (user_id, amount) VALUES (1, 100.50), (2, 25.00);"
```

```bash
gfs commit -m "my second commit"
```

### 8. Make more changes

Use `gfs query` to run SQL or open an interactive session, then commit:

```bash
gfs query "ALTER TABLE users ADD COLUMN email TEXT;"
gfs commit -m "my third commit"
```

### 9. Time travel through history

View the log again:

```bash
gfs log
```

Checkout a previous commit:

```bash
gfs checkout <commit_hash>
```

Your database will be restored to that exact state!

### 10. Work with branches

Create a new branch:

```bash
gfs checkout -b release
```

Switch back to main:

```bash
gfs checkout main
```

## Command Reference

### Revision References

GFS supports Git-style revision notation for referencing commits in commands like `checkout`, `schema show`, and `schema diff`:

- `HEAD` - Current commit
- `main` - Branch tip (any branch name)
- `abc123...` - Full commit hash (64 characters)
- `HEAD~1` - Parent of HEAD (previous commit)
- `HEAD~5` - 5th ancestor of HEAD
- `main~3` - 3 commits before main branch tip

Examples:
```bash
gfs checkout HEAD~1                    # Checkout previous commit
gfs schema diff HEAD~5 HEAD           # Compare schema with 5 commits ago
gfs schema show main~3                # View schema from 3 commits back
```

### `gfs providers`

List available database providers and their supported versions.

```bash
# List all providers
gfs providers

# Show details for a specific provider
gfs providers postgres
```

**Example output:**

```
  database_provider    | version                        | features
  ---------------------+--------------------------------+---------------------------------------------------
  mysql                | 8.0, 8.1                       | tls, schema, masking, backup, import
  postgres             | 13, 14, 15, 16, 17, 18         | tls, schema, masking, auto-scaling, performance...

  Images are pulled from Docker Hub by default.
```

For provider details (e.g. `gfs providers postgres`):

```
  Provider: postgres

  Supported versions: 13, 14, 15, 16, 17, 18

  Features
  ─────────────────────────────────────────────────────────────────────────────────────
  feature                   | description
  --------------------------+--------------------------------------------------------
  tls                       | TLS/SSL encryption for connections.
  schema                    | Schema and DDL management.
  ...
```

Use this command to check available providers before initializing a repository.

### `gfs init`

Initialize a new GFS repository.

```bash
gfs init --database-provider <provider> --database-version <version>
```

**Options:**
- `--database-provider`: Database type (supports `postgres`, `mysql`)
- `--database-version`: Database version (e.g., `17` for postgres, `8.0` for mysql)

### `gfs status`

Show the current state of storage and compute resources.

```bash
gfs status
# Or JSON output:
gfs status --output json
```

**Example output:**

```
  Repository
  ────────────────────────────────────────
  Branch               main
  Active workspace     .gfs/workspaces/main/0/data

  Compute
  ────────────────────────────────────────
  Provider             postgres
  Version              17
  Status               ● running
  Container ID         37f65464d421…
  Container data dir   .gfs/workspaces/main/0/data
  Connection           postgresql://postgres:postgres@localhost:55251/postgres
```

### `gfs commit`

Commit the current database state.

```bash
gfs commit -m "commit message"
```

**Example output:**

```
[main] 88b0ff8  Add users table
```

**Options:**
- `-m, --message`: Commit message describing the changes

### `gfs log`

Show the commit history.

```bash
gfs log
gfs log -n 10                    # Limit to 10 commits
gfs log --full-hash              # Show full 64-char hashes
```

**Example output:**

```
commit 88b0ff8 (HEAD -> main, main)
Author: user
Date:   Sun Mar  1 12:56:43 2026 +0000

    Add users table
```

### `gfs checkout`

Switch to a different commit or branch.

```bash
# Checkout a specific commit
gfs checkout <commit_hash>

# Create and checkout a new branch
gfs checkout -b <branch_name>

# Checkout an existing branch
gfs checkout <branch_name>
```

**Example output:**

```
Switched to new branch 'feature-test' (88b0ff8)
Switched to main (88b0ff8)
```

**Options:**
- `-b`: Create a new branch

### `gfs query`

Execute SQL queries or open an interactive database terminal.

```bash
# Execute a SQL query
gfs query "SELECT * FROM users WHERE id = 1"

# Open interactive terminal (omit the query)
gfs query
```

**Example output (for `gfs query "SELECT 1"`):**

```
 ?column?
----------
        1
(1 row)
```

**Options:**
- `--database`: Override the default database name
- `--path`: Path to the GFS repository root

### `gfs schema`

Database schema operations: extract, show, and diff.

```bash
# Extract schema from the running database
gfs schema extract [--output <file>] [--compact]

# Show schema from a specific commit
gfs schema show <commit> [--metadata-only] [--ddl-only]

# Compare schemas between two commits
gfs schema diff <commit1> <commit2> [--pretty] [--json]
```

**Options:**
- `extract`: Schema from the running database (JSON output)
- `show`: Schema from a historical commit
- `diff`: Compare two commits (agentic format by default; `--pretty` for human-readable; `--json` for structured output)

### `gfs export`

Export data from the running database to a file.

```bash
gfs export --output-dir <dir> --format <fmt>
```

**Options:**
- `--output-dir`: Directory where the export file will be written
- `--format`: Export format (`sql` for plain-text SQL, `custom` for PostgreSQL binary dump)

### `gfs import`

Import data from a file into the running database.

```bash
gfs import --file <path> [--format <fmt>]
```

**Options:**
- `--file`: Path to the dump file (`.sql`, `.dump`, or `.csv`)
- `--format`: Import format (inferred from file extension when omitted)

### `gfs config`

Read or write repository config (e.g. `user.name`, `user.email`).

```bash
# Read a config value
gfs config user.name

# Set a config value
gfs config user.name "John Doe"
```

### `gfs compute`

Manage the database container (start, stop, status, logs).

```bash
gfs compute start    # Start the container
gfs compute stop     # Stop the container
gfs compute status   # Show container status
gfs compute logs     # View container logs (--tail, --since options)
```

## MCP Server

GFS includes a Model Context Protocol (MCP) server that allows AI assistants and other tools to interact with your GFS repositories programmatically. The MCP server provides a standardized interface for database version control operations.

### Starting the MCP Server

**Stdio mode (default):**

```bash
# Start MCP server with stdio transport (for direct client use)
gfs mcp
# or explicitly
gfs mcp stdio
```

This mode is designed for direct integration with MCP-compatible clients like Claude Desktop, Cline, or other AI tools.

**HTTP mode (daemon):**

```bash
# Start as a background daemon with HTTP transport
gfs mcp start

# Check daemon status
gfs mcp status

# Stop the daemon
gfs mcp stop

# Restart the daemon
gfs mcp restart
```

**HTTP mode (foreground):**

```bash
# Start with HTTP transport in foreground (default port: 3000)
gfs mcp web

# Specify a custom port
gfs mcp web --port 8080
```

### Specifying a Repository Path

```bash
# Use a specific repository
gfs mcp --path /path/to/repo

# Start daemon for a specific repository
gfs mcp --path /path/to/repo start
```

### MCP Version

```bash
gfs mcp version
```

### Use Cases

- **AI Assistants**: Integrate with Claude Desktop or other AI tools for automated database version control
- **CI/CD Integration**: Use the HTTP API in automated pipelines
- **Custom Tools**: Build custom tooling on top of the MCP interface
- **Remote Management**: Control GFS repositories from remote systems

### Claude Desktop Integration

To use GFS with Claude Desktop, add the following to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "gfs": {
      "command": "gfs",
      "args": ["mcp", "--path", "/path/to/your/repo"]
    }
  }
}
```

Then restart Claude Desktop and GFS operations will be available as tools.

## Configuration

GFS uses Docker to manage database containers. Make sure Docker is installed and running before using GFS.

### Requirements

- Docker (latest version recommended)
- Bash/Zsh shell
- `curl` for installation
- `tar` for extracting releases

## Troubleshooting

### Docker not running

If you get an error about Docker not being available:

```bash
# Start Docker Desktop or Docker daemon
# On macOS/Windows: Start Docker Desktop
# On Linux: sudo systemctl start docker
```

### Port conflicts

If the default port (5432 for PostgreSQL) is already in use:

```bash
# Stop the conflicting service or configure GFS to use a different port
# (Port configuration coming in future releases)
```

### Connection issues

If you can't connect to the database:

1. Check that the container is running: `docker ps`
2. Verify the connection details with: `gfs status`
3. Ensure Docker has network access

## Development

### Prerequisites

- Rust (latest stable version)
- Docker
- Cargo

### Running locally

Clone the repository:

```bash
git clone https://github.com/Guepard-Corp/gfs.git
cd gfs
```

Build the project:

```bash
cargo build
```

Run commands using cargo:

```bash
# Initialize a repository
cargo run --bin gfs init --database-provider postgres --database-version 17

# Commit changes
cargo run --bin gfs commit -m "v1"

# View history
cargo run --bin gfs log

# Check status
cargo run --bin gfs status
```

### Testing

Run all tests:

```bash
cargo test
```

On macOS, the full suite (including E2E checkout tests) requires sequential execution. Use either:

```bash
cargo test-all
# or
RUST_TEST_THREADS=1 cargo test

#or generate coverage report
cargo cov
```

Run specific tests:

```bash
cargo test <test_name>
```

Run tests with output:

```bash
cargo test -- --nocapture
```

**Optional: Better test reports and code coverage**

- [cargo-nextest](https://nexte.st/): Faster, clearer test output. Install with `cargo install cargo-nextest`, then run `cargo nextest run` or `cargo nt`.
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov): Code coverage. Install with `cargo install cargo-llvm-cov` (requires `rustup component add llvm-tools-preview`). Run `cargo llvm-cov --html --open` for an HTML report.

### Building for release

```bash
cargo build --release
```

The binary will be available at `target/release/gfs`.

## Contributing

We welcome contributions! Whether you're fixing bugs, adding features, or improving documentation, your help is appreciated.

Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines on:
- How to submit contributions
- Code contribution workflow
- Good first issues to get started
- Development best practices

For quick questions, join our [Discord community](https://discord.gg/SEdZuJbc5V).

## Community

- **Discord**: [Join our community](https://discord.gg/SEdZuJbc5V)
- **YouTube**: [Watch the demo](https://youtu.be/WlOkLnoY2h8?si=hb6-7kLhlOvVL1u6)
- **Issues**: [Report bugs or request features](https://github.com/Guepard-Corp/gfs/issues)

## Roadmap

Check [Roadmap](ROADMAP.md)

## License

This project is licensed under the Elastic License v2 (ELv2). See the [LICENSE](LICENCE) file for details.

## Acknowledgments

GFS is inspired by Git's approach to version control and extends these concepts to database management. Special thanks to all contributors and the open-source community.

---

<div align="center">
Made with ❤️ by the Guepard team
</div>
