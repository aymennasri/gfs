//! Integration tests for the gfs CLI.
//!
//! Uses simple temp directories only (no diskutil or storage backends).
//! Runs CLI in-process via gfs_cli::run() for coverage capture.

mod common;

use std::path::PathBuf;

use common::cli_runner;
use gfs_domain::repo_utils::repo_layout::validate_repo_layout;
use tempfile::tempdir;

/// Checks that `gfs init <path>` creates a valid `.gfs` directory layout.
#[test]
fn gfs_init_creates_valid_repo_layout() {
    let temp_dir = tempdir().expect("create temp dir");
    let work_dir = temp_dir.path().to_path_buf();

    let ok = cli_runner::gfs_init(&work_dir);
    assert!(ok, "gfs init should succeed");

    let gfs_path: PathBuf = work_dir.join(".gfs");
    assert!(gfs_path.exists(), ".gfs directory should exist");
    assert!(gfs_path.is_dir(), ".gfs should be a directory");

    assert!(
        validate_repo_layout(&gfs_path).is_ok(),
        ".gfs layout should be valid"
    );
}
