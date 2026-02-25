![Guepard](/resources/guepard-cover.png)

<div align="center">
    <h1>Git For database Systems</h1>
    <br />  
    <p align="center">
    <a href="https://youtu.be/WlOkLnoY2h8?si=hb6-7kLhlOvVL1u6">
        <img src="https://img.shields.io/badge/Watch-YouTube-%23ffcb51?logo=youtube&logoColor=black" alt="Watch on YouTube" />
    </a>
    <a href="https://discord.gg/nCXAsUd3hm">
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
- More database providers coming soon

Run `gfs providers` to see all available providers and their supported versions.

## Features

- ✅ Initialize database repositories
- ✅ Commit database changes
- ✅ View commit history
- ✅ Checkout previous commits
- ✅ Create and switch branches
- ✅ Check database status
- 🚧 Merge branches (coming soon)
- 🚧 Remote repositories (coming soon)
- 🚧 Conflict resolution (coming soon)

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

### 5. Connect to your database

```bash
# GFS will output the connection details, typically:
psql -h localhost -p 5432 -U postgres -d postgres
```

Or use any SQL client with the connection details shown by `gfs status`.

### 6. Make changes and commit

After modifying your database schema or data:

```bash
gfs commit -m "my first commit"
```

### 7. View commit history

```bash
gfs log
```

### 8. Make more changes

Connect to your database and make additional changes:

```bash
psql -h localhost -p 5432 -U postgres -d postgres
# Make your changes...
```

Then commit again:

```bash
gfs commit -m "my second commit"
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
```

### `gfs commit`

Commit the current database state.

```bash
gfs commit -m "commit message"
```

**Options:**
- `-m, --message`: Commit message describing the changes

### `gfs log`

Show the commit history.

```bash
gfs log
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

**Options:**
- `-b`: Create a new branch

### `gfs branch`

List, create, or delete branches (coming soon).

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

Run specific tests:

```bash
cargo test <test_name>
```

Run tests with output:

```bash
cargo test -- --nocapture
```

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

For quick questions, join our [Discord community](https://discord.gg/nCXAsUd3hm).

## Community

- **Discord**: [Join our community](https://discord.gg/nCXAsUd3hm)
- **YouTube**: [Watch the demo](https://youtu.be/WlOkLnoY2h8?si=hb6-7kLhlOvVL1u6)
- **Issues**: [Report bugs or request features](https://github.com/Guepard-Corp/gfs/issues)

## Roadmap

- [ ] Merge branch functionality
- [ ] Remote repository support (push/pull)
- [ ] Conflict resolution tools
- [ ] Support for MongoDB
- [ ] Support for SQLite
- [ ] Support for MariaDB
- [ ] Web UI for visualization
- [ ] CI/CD integrations
- [ ] Diff visualization tools
- [ ] Branch comparison and diff

## License

This project is licensed under the Elastic License v2 (ELv2). See the [LICENSE](LICENCE) file for details.

## Acknowledgments

GFS is inspired by Git's approach to version control and extends these concepts to database management. Special thanks to all contributors and the open-source community.

---

<div align="center">
Made with ❤️ by the Guepard team
</div>
