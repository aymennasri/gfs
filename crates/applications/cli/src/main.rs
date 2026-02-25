//! `gfs` – Guepard data-plane CLI.
//!
//! # Usage
//!
//! ```text
//! gfs commit -m <message> [--path <dir>] [--author <name>] [--author-email <email>]
//!
//! gfs storage mount   --id <volume-id> --mount-point <path>
//! gfs storage unmount --id <volume-id>
//! gfs storage snapshot --id <volume-id> [--label <label>]
//! gfs storage clone   --source <volume-id> --target <volume-id> [--from-snapshot <snap-id>]
//! gfs storage status  --id <volume-id>
//! gfs storage quota   --id <volume-id>
//!
//! gfs status [--path <dir>] [--output table|json]
//!
//! gfs providers
//!
//! gfs compute [--path <dir>] status   [--id <container>]
//! gfs compute [--path <dir>] start    [--id <container>]
//! gfs compute [--path <dir>] stop     [--id <container>]
//! gfs compute [--path <dir>] restart  [--id <container>]
//! gfs compute [--path <dir>] pause    [--id <container>]
//! gfs compute [--path <dir>] unpause  [--id <container>]
//! gfs compute [--path <dir>] logs     [--id <container>] [--tail N] [--since RFC3339] [--no-stdout] [--no-stderr]
//!
//! gfs mcp [--path <dir>]              (defaults to stdio)
//! gfs mcp [--path <dir>] stdio
//! gfs mcp [--path <dir>] web [--port <port>]
//! gfs mcp [--path <dir>] start | stop | restart | status
//! gfs mcp version
//!
//! gfs version
//! ```

mod cli_utils;
mod commands;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use gfs_domain::ports::storage::{CloneOptions, SnapshotId, SnapshotOptions, VolumeId};

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "gfs", about = "Guepard CLI", version, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: TopLevel,
}

#[derive(Subcommand)]
enum TopLevel {
    /// Initialize a new Guepard environment at the given path
    Init {
        /// Path where to initialize the .gfs repo (default: current directory)
        path: Option<PathBuf>,

        /// Database provider to deploy (e.g. postgres). If set, the repo is initialized and the database is provisioned and started. Requires --database-version.
        #[arg(long)]
        database_provider: Option<String>,

        /// Database version (e.g. 17 for postgres). Required when --database-provider is set.
        #[arg(long)]
        database_version: Option<String>,
    },

    /// Record a commit of the current repository state
    Commit {
        /// Commit message (required)
        #[arg(short = 'm', long)]
        message: String,

        /// Path to the GFS repository root (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Override the author name (falls back to user.name in repo config)
        #[arg(long)]
        author: Option<String>,

        /// Override the author e-mail (falls back to user.email in repo config)
        #[arg(long)]
        author_email: Option<String>,
    },

    /// Read or write repository config (e.g. user.name, user.email)
    Config {
        /// Path to the GFS repository root (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Config key (e.g. user.name, user.email)
        key: String,

        /// Value to set; omit to read
        value: Option<String>,
    },

    /// Switch branch or checkout a commit (detached HEAD). Use -b to create a new branch.
    Checkout {
        /// Path to the GFS repository root (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Create a new branch and switch to it (optional start revision defaults to HEAD)
        #[arg(short = 'b', long = "branch")]
        create_branch: Option<String>,

        /// Branch name or full 64-char commit hash; or start revision when using -b
        revision: Option<String>,
    },

    /// List database providers and their supported versions. Pass a provider name for details.
    Providers {
        /// Provider name to show details for (e.g. postgres). Omit to list all providers.
        #[arg()]
        provider: Option<String>,
    },

    /// Display commit history
    Log {
        /// Path to the GFS repository root (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Limit the number of commits to display
        #[arg(short = 'n', long)]
        max_count: Option<usize>,

        /// Start traversal at this revision (branch name or full hash)
        #[arg(long)]
        from: Option<String>,

        /// Stop before this revision (exclusive)
        #[arg(long)]
        until: Option<String>,
    },

    /// Show repository and compute status (current branch, container state, connection string)
    Status {
        /// Path to the GFS repository root (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Output format: table (default) or json
        #[arg(long, default_value = "table", value_parser = ["table", "json"])]
        output: String,
    },

    /// Storage operations (mount, unmount, snapshot, clone, status, quota)
    Storage {
        #[command(subcommand)]
        action: StorageAction,
    },

    /// Compute instance management (Docker containers)
    Compute {
        /// Path to the GFS repository root (default: current directory). When set, --id may be omitted and the container name is read from .gfs/config.toml (runtime.container_name).
        #[arg(long)]
        path: Option<PathBuf>,

        #[command(subcommand)]
        action: ComputeAction,
    },

    /// MCP server (start stdio, daemon, or utilities). Defaults to stdio if no subcommand given.
    Mcp {
        /// Path to the GFS repository root (default: current directory). The daemon will use this repo.
        #[arg(long)]
        path: Option<PathBuf>,

        #[command(subcommand)]
        action: Option<McpAction>,
    },

    /// Print the CLI version
    Version,
}

// ---------------------------------------------------------------------------
// Storage subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
enum StorageAction {
    /// Mount a volume at the given path
    Mount {
        /// Volume identifier (e.g. disk3s1 or APFS UUID)
        #[arg(long)]
        id: String,

        /// Filesystem path where the volume should be mounted
        #[arg(long)]
        mount_point: PathBuf,
    },

    /// Unmount a volume
    Unmount {
        /// Volume identifier
        #[arg(long)]
        id: String,
    },

    /// Create a point-in-time snapshot of a volume
    Snapshot {
        /// Volume identifier
        #[arg(long)]
        id: String,

        /// Human-readable label for the snapshot
        #[arg(long)]
        label: Option<String>,
    },

    /// Clone a volume (optionally from a snapshot)
    Clone {
        /// Source volume identifier
        #[arg(long)]
        source: String,

        /// Target volume identifier (must not already exist)
        #[arg(long)]
        target: String,

        /// Snapshot to clone from; clones the live volume when omitted
        #[arg(long)]
        from_snapshot: Option<String>,
    },

    /// Show the current status of a volume
    Status {
        /// Volume identifier
        #[arg(long)]
        id: String,
    },

    /// Show disk-usage quota for a volume
    Quota {
        /// Volume identifier
        #[arg(long)]
        id: String,
    },
}

// ---------------------------------------------------------------------------
// Compute subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum ComputeAction {
    /// Show the current status of a compute instance
    Status {
        /// Container name or id (defaults to repo runtime.container_name when --path is set)
        #[arg(long)]
        id: Option<String>,
    },

    /// Start a stopped compute instance
    Start {
        /// Container name or id (defaults to repo runtime.container_name when --path is set)
        #[arg(long)]
        id: Option<String>,
    },

    /// Gracefully stop a running compute instance
    Stop {
        /// Container name or id (defaults to repo runtime.container_name when --path is set)
        #[arg(long)]
        id: Option<String>,
    },

    /// Stop then start a compute instance
    Restart {
        /// Container name or id (defaults to repo runtime.container_name when --path is set)
        #[arg(long)]
        id: Option<String>,
    },

    /// Suspend a running compute instance (cgroups freezer)
    Pause {
        /// Container name or id (defaults to repo runtime.container_name when --path is set)
        #[arg(long)]
        id: Option<String>,
    },

    /// Resume a paused compute instance
    Unpause {
        /// Container name or id (defaults to repo runtime.container_name when --path is set)
        #[arg(long)]
        id: Option<String>,
    },

    /// Fetch log output from a compute instance
    Logs {
        /// Container name or id (defaults to repo runtime.container_name when --path is set)
        #[arg(long)]
        id: Option<String>,

        /// Return at most N most-recent lines (default: all)
        #[arg(long)]
        tail: Option<usize>,

        /// Only return entries after this RFC 3339 timestamp
        #[arg(long)]
        since: Option<String>,

        /// Include stdout (default: true; pass --no-stdout to disable)
        #[arg(long, default_missing_value = "true", num_args = 0..=1)]
        stdout: Option<bool>,

        /// Include stderr (default: true; pass --no-stderr to disable)
        #[arg(long, default_missing_value = "true", num_args = 0..=1)]
        stderr: Option<bool>,
    },
}

// ---------------------------------------------------------------------------
// MCP daemon subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum McpAction {
    /// Start the MCP server as a daemon (background process with HTTP)
    Start,

    /// Stop the MCP server daemon
    Stop,

    /// Restart the MCP server daemon
    Restart,

    /// Show MCP server daemon status (running or stopped, PID)
    Status,

    /// Start the MCP server with stdio transport (for direct client use)
    Stdio,

    /// Start the MCP server with HTTP transport (equivalent to start, but runs in foreground)
    Web {
        /// HTTP port to listen on (default: 3000)
        #[arg(long, default_value = "3000")]
        port: u16,
    },

    /// Show the version of the MCP crate
    Version,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        TopLevel::Init {
            path,
            database_provider,
            database_version,
        } => commands::cmd_init::init(path, database_provider, database_version)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e)),
        TopLevel::Commit {
            message,
            path,
            author,
            author_email,
        } => commands::cmd_commit::commit(path, message, author, author_email).await,
        TopLevel::Config { path, key, value } => {
            commands::cmd_config::run(path, key, value).map_err(|e| anyhow::anyhow!("{}", e))
        }
        TopLevel::Checkout {
            path,
            create_branch,
            revision,
        } => commands::cmd_checkout::checkout(path, revision, create_branch).await,
        TopLevel::Providers { provider } => {
            commands::cmd_providers::run(provider).map_err(|e| anyhow::anyhow!("{}", e))
        }
        TopLevel::Log {
            path,
            max_count,
            from,
            until,
        } => commands::cmd_log::log(path, max_count, from, until).await,
        TopLevel::Status { path, output } => commands::cmd_status::run(path, output).await,
        TopLevel::Storage { action } => run_storage(action).await,
        TopLevel::Compute { path, action } => run_compute(path, action).await,
        TopLevel::Mcp { path, action } => {
            let action = action.unwrap_or(McpAction::Stdio);
            commands::cmd_mcp::run(path, action).await
        }

        TopLevel::Version => {
            commands::cmd_version::run();
            Ok(())
        }
    };

    if let Err(err) = result {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Storage dispatch
// ---------------------------------------------------------------------------

async fn run_storage(action: StorageAction) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use gfs_storage_apfs::ApfsStorage;

        let storage = ApfsStorage::new();
        dispatch_storage(&storage, action).await
    }

    #[cfg(not(target_os = "macos"))]
    {
        use gfs_storage_file::FileStorage;

        let storage = FileStorage::new();
        dispatch_storage(&storage, action).await
    }
}

async fn dispatch_storage(
    storage: &impl gfs_domain::ports::storage::StoragePort,
    action: StorageAction,
) -> Result<()> {
    match action {
        StorageAction::Mount { id, mount_point } => {
            storage
                .mount(&VolumeId(id), &mount_point)
                .await
                .map_err(anyhow::Error::from)?;
            println!("mounted");
        }

        StorageAction::Unmount { id } => {
            storage
                .unmount(&VolumeId(id))
                .await
                .map_err(anyhow::Error::from)?;
            println!("unmounted");
        }

        StorageAction::Snapshot { id, label } => {
            let snap = storage
                .snapshot(&VolumeId(id), SnapshotOptions { label })
                .await
                .map_err(anyhow::Error::from)?;
            println!("snapshot id  : {}", snap.id);
            println!("volume       : {}", snap.volume_id);
            println!("created_at   : {}", snap.created_at);
            if let Some(lbl) = &snap.label {
                println!("label        : {lbl}");
            }
        }

        StorageAction::Clone {
            source,
            target,
            from_snapshot,
        } => {
            let opts = CloneOptions {
                from_snapshot: from_snapshot.map(SnapshotId),
            };
            let status = storage
                .clone(&VolumeId(source), VolumeId(target), opts)
                .await
                .map_err(anyhow::Error::from)?;
            print_volume_status(&status);
        }

        StorageAction::Status { id } => {
            let status = storage
                .status(&VolumeId(id))
                .await
                .map_err(anyhow::Error::from)?;
            print_volume_status(&status);
        }

        StorageAction::Quota { id } => {
            let quota = storage
                .quota(&VolumeId(id))
                .await
                .map_err(anyhow::Error::from)?;
            println!("volume      : {}", quota.volume_id);
            println!("limit_bytes : {}", quota.limit_bytes);
            println!("used_bytes  : {}", quota.used_bytes);
            println!("free_bytes  : {}", quota.free_bytes);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Compute dispatch
// ---------------------------------------------------------------------------

async fn run_compute(path: Option<PathBuf>, action: ComputeAction) -> Result<()> {
    commands::cmd_compute::run(path, action).await
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

fn print_volume_status(s: &gfs_domain::ports::storage::VolumeStatus) {
    println!("id          : {}", s.id);
    println!(
        "mount_point : {}",
        s.mount_point
            .as_deref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "-".to_owned())
    );
    println!("status      : {:?}", s.status);
    println!("size_bytes  : {}", s.size_bytes);
    println!("used_bytes  : {}", s.used_bytes);
}
