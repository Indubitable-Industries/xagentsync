//! CLI integration tests
//!
//! These tests run the actual CLI binary and verify behavior.

use std::process::Command;
use std::path::PathBuf;
use tempfile::TempDir;

fn xas_binary() -> PathBuf {
    // Find the built binary
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("xas");
    path
}

fn run_xas(dir: &TempDir, args: &[&str]) -> (bool, String, String) {
    let output = Command::new(xas_binary())
        .current_dir(dir.path())
        .args(args)
        .output()
        .expect("Failed to execute xas");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (output.status.success(), stdout, stderr)
}

#[test]
fn test_cli_init() {
    let dir = TempDir::new().unwrap();

    let (success, stdout, _) = run_xas(&dir, &["init"]);

    assert!(success, "init should succeed");
    assert!(stdout.contains("Initialized XAgentSync"));
    assert!(dir.path().join("pending").exists());
    assert!(dir.path().join(".xas").exists());
}

#[test]
fn test_cli_whoami() {
    let dir = TempDir::new().unwrap();

    // Init first
    run_xas(&dir, &["init"]);

    // Set identity
    let (success, stdout, _) = run_xas(&dir, &["whoami", "--set", "test-agent"]);
    assert!(success);
    assert!(stdout.contains("test-agent"));

    // Read identity back
    let (success, stdout, _) = run_xas(&dir, &["whoami"]);
    assert!(success);
    assert!(stdout.contains("test-agent"));
}

#[test]
fn test_cli_status_empty() {
    let dir = TempDir::new().unwrap();
    run_xas(&dir, &["init"]);
    run_xas(&dir, &["whoami", "--set", "test-agent"]);

    let (success, stdout, _) = run_xas(&dir, &["status"]);

    assert!(success);
    assert!(stdout.contains("test-agent"));
    assert!(stdout.contains("No pending handoffs"));
}

#[test]
fn test_cli_plan_workflow() {
    let dir = TempDir::new().unwrap();
    run_xas(&dir, &["init"]);
    run_xas(&dir, &["whoami", "--set", "test-agent"]);

    // Start plan
    let (success, stdout, _) = run_xas(&dir, &["plan", "new", "Test planning"]);
    assert!(success);
    assert!(stdout.contains("Started plan handoff"));

    // Add requirement
    let (success, stdout, _) = run_xas(&dir, &["plan", "require", "Must be fast", "--priority", "must"]);
    assert!(success);
    assert!(stdout.contains("Added requirement"));

    // Add decision (without --why, testing default)
    let (success, stdout, _) = run_xas(&dir, &["plan", "decided", "Use Rust"]);
    assert!(success);
    assert!(stdout.contains("Recorded decision"));

    // Add decision with --why
    let (success, stdout, _) = run_xas(&dir, &["plan", "decided", "Use serde", "--why", "Best serialization"]);
    assert!(success);
    assert!(stdout.contains("Recorded decision"));

    // Add rejected option
    let (success, stdout, _) = run_xas(&dir, &["plan", "rejected", "Use Python", "Too slow"]);
    assert!(success);
    assert!(stdout.contains("Recorded rejected"));

    // Add question (without --importance, testing default)
    let (success, stdout, _) = run_xas(&dir, &["plan", "question", "What about Go?"]);
    assert!(success);
    assert!(stdout.contains("Added question"));

    // Status should show WIP
    let (_, stdout, _) = run_xas(&dir, &["status"]);
    assert!(stdout.contains("Work in progress"));
    assert!(stdout.contains("Test planning"));
}

#[test]
fn test_cli_debug_workflow() {
    let dir = TempDir::new().unwrap();
    run_xas(&dir, &["init"]);
    run_xas(&dir, &["whoami", "--set", "test-agent"]);

    // Start debug
    let (success, _, _) = run_xas(&dir, &["debug", "new", "Server crashing"]);
    assert!(success);

    // Add symptom
    let (success, stdout, _) = run_xas(&dir, &["debug", "symptom", "OOM errors in logs"]);
    assert!(success);
    assert!(stdout.contains("Added symptom"));

    // Add hypothesis
    let (success, _, _) = run_xas(&dir, &["debug", "hypothesis", "Memory leak", "--likelihood", "high"]);
    assert!(success);

    // Add tried (without --result, testing default)
    let (success, stdout, _) = run_xas(&dir, &["debug", "tried", "Restarted server"]);
    assert!(success);
    assert!(stdout.contains("Recorded attempt"));

    // Add suspect
    let (success, _, _) = run_xas(&dir, &["debug", "suspect", "src/cache.rs", "Unbounded cache"]);
    assert!(success);

    // Status should show WIP
    let (_, stdout, _) = run_xas(&dir, &["status"]);
    assert!(stdout.contains("Server crashing"));
}

#[test]
fn test_cli_deploy_workflow() {
    let dir = TempDir::new().unwrap();
    run_xas(&dir, &["init"]);
    run_xas(&dir, &["whoami", "--set", "test-agent"]);

    // Start deploy
    let (success, _, _) = run_xas(&dir, &["deploy", "new", "Ship v1.0"]);
    assert!(success);

    // Add ship item
    let (success, _, _) = run_xas(&dir, &["deploy", "ship", "src/*"]);
    assert!(success);

    // Add verification
    let (success, _, _) = run_xas(&dir, &["deploy", "verify", "Run tests"]);
    assert!(success);

    // Set rollback
    let (success, _, _) = run_xas(&dir, &["deploy", "rollback", "git revert HEAD"]);
    assert!(success);

    // Status should show WIP
    let (_, stdout, _) = run_xas(&dir, &["status"]);
    assert!(stdout.contains("Ship v1.0"));
}

#[test]
fn test_cli_receive_empty() {
    let dir = TempDir::new().unwrap();
    run_xas(&dir, &["init"]);

    let (success, stdout, _) = run_xas(&dir, &["receive"]);

    assert!(success);
    assert!(stdout.contains("No pending handoffs"));
}

#[test]
fn test_cli_no_active_handoff_error() {
    let dir = TempDir::new().unwrap();
    run_xas(&dir, &["init"]);
    run_xas(&dir, &["whoami", "--set", "test-agent"]);

    // Try to add to non-existent WIP
    let (success, _, stderr) = run_xas(&dir, &["plan", "require", "Something"]);

    assert!(!success);
    assert!(stderr.contains("No active handoff") || stderr.contains("NoActiveHandoff"));
}

#[test]
fn test_cli_help() {
    let dir = TempDir::new().unwrap();

    let (success, stdout, _) = run_xas(&dir, &["--help"]);

    assert!(success);
    assert!(stdout.contains("LLM-to-LLM") || stdout.contains("async"));
    assert!(stdout.contains("deploy"));
    assert!(stdout.contains("debug"));
    assert!(stdout.contains("plan"));
}
