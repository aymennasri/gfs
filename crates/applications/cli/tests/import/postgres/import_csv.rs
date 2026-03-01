//! Integration tests for `gfs import` with PostgreSQL CSV format.
//!
//! Run with: `cargo test -p gfs-cli import_postgres_csv -- --test-threads=1`

#![cfg(target_os = "macos")]

use std::fs;

#[path = "../../common/mod.rs"]
mod common;

use common::postgres::*;
use serial_test::serial;

/// Import a PostgreSQL CSV file and verify the data is loaded.
#[test]
#[serial]
fn import_postgres_csv() {
    with_fresh_repo(|repo_path| {
        let import_dir = repo_path.join("import_data");
        fs::create_dir_all(&import_dir).expect("create import dir");
        let csv_path = import_dir.join("sample_data.csv");
        fs::write(&csv_path, "id,name\n1,alice\n2,bob\n3,charlie").expect("write CSV");

        let (ok, stdout, stderr) = gfs_import(repo_path, &csv_path, Some("csv"));
        assert!(ok, "gfs import csv should succeed; stderr: {stderr}");
        if !stdout.is_empty() {
            assert!(
                stdout.contains("Imported from"),
                "stdout should mention import; got: {stdout}"
            );
        }

        let container_id = get_container_id(repo_path);
        let result = run_psql_select(&container_id, "SELECT id, name FROM csv_import ORDER BY id");
        assert!(
            result.contains("1|alice"),
            "csv_import should contain (1,alice); got: {result}"
        );
        assert!(
            result.contains("2|bob"),
            "csv_import should contain (2,bob); got: {result}"
        );
        assert!(
            result.contains("3|charlie"),
            "csv_import should contain (3,charlie); got: {result}"
        );
    });
}
