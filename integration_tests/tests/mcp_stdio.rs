//! Integration test for the MCP server (gfs-mcp) over stdio.
//!
//! Spawns the gfs-mcp binary, sends the MCP handshake (initialize + initialized notification),
//! then calls tools/list and asserts the expected tools are returned.
//!
//! Requires the gfs-mcp binary to be built: `cargo build -p gfs-mcp --bin gfs-mcp`

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Workspace root (parent of integration_tests).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[test]
fn mcp_stdio_handshake_and_list_tools() {
    let bin = workspace_root()
        .join("target/debug")
        .join(if cfg!(windows) {
            "gfs-mcp.exe"
        } else {
            "gfs-mcp"
        });
    if !bin.exists() {
        eprintln!("Skipping MCP test: gfs-mcp not found at {}", bin.display());
        eprintln!("Run: cargo build -p gfs-mcp --bin gfs-mcp");
        return;
    }

    let mut child = Command::new(&bin)
        .env("GFS_REPO_PATH", workspace_root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn gfs-mcp");

    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");

    // 1. Send initialize request (newline-delimited JSON)
    let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
    writeln!(stdin, "{}", init_req).expect("write init");
    stdin.flush().expect("flush");

    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    // 2. Read initialize response
    let init_line = lines.next().expect("read init response").expect("line");
    let init_resp: serde_json::Value =
        serde_json::from_str(&init_line).expect("parse init response");
    assert_eq!(init_resp.get("id"), Some(&serde_json::json!(1)));
    let result = init_resp.get("result").expect("result");
    assert_eq!(
        result
            .get("serverInfo")
            .and_then(|s| s.get("name"))
            .and_then(|v| v.as_str()),
        Some("gfs-mcp")
    );

    // 3. Send initialized notification (no id)
    let notif = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    writeln!(stdin, "{}", notif).expect("write initialized");
    stdin.flush().expect("flush");

    // 4. Send tools/list request (keep stdin open so server does not see EOF before responding)
    let tools_req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    writeln!(stdin, "{}", tools_req).expect("write tools/list");
    stdin.flush().expect("flush");

    // 5. Read tools/list response
    let tools_line = lines
        .next()
        .expect("read tools/list response")
        .expect("line");
    let tools_resp: serde_json::Value =
        serde_json::from_str(&tools_line).expect("parse tools response");
    assert_eq!(tools_resp.get("id"), Some(&serde_json::json!(2)));
    let tools_result = tools_resp.get("result").expect("result");
    let tool_list = tools_result
        .get("tools")
        .and_then(|t| t.as_array())
        .expect("tools array");
    let names: Vec<&str> = tool_list
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        names.contains(&"list_providers"),
        "expected list_providers in {:?}",
        names
    );
    assert!(names.contains(&"status"), "expected status in {:?}", names);
    assert!(names.contains(&"commit"), "expected commit in {:?}", names);

    drop(stdin);
    let status = child.wait().expect("wait");
    assert!(status.success(), "gfs-mcp should exit successfully");
}
