//! CLI integration tests
//!
//! These tests run the actual CLI binary and verify behavior.

use std::path::PathBuf;
use std::process::Command;
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

fn run_git(dir: &TempDir, args: &[&str]) -> (bool, String, String) {
    let output = Command::new("git")
        .current_dir(dir.path())
        .args(args)
        .output()
        .expect("Failed to execute git");

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
    let (success, stdout, _) = run_xas(
        &dir,
        &["plan", "require", "Must be fast", "--priority", "must"],
    );
    assert!(success);
    assert!(stdout.contains("Added requirement"));

    // Add decision (without --why, testing default)
    let (success, stdout, _) = run_xas(&dir, &["plan", "decided", "Use Rust"]);
    assert!(success);
    assert!(stdout.contains("Recorded decision"));

    // Add decision with --why
    let (success, stdout, _) = run_xas(
        &dir,
        &[
            "plan",
            "decided",
            "Use serde",
            "--why",
            "Best serialization",
        ],
    );
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
    let (success, _, _) = run_xas(
        &dir,
        &["debug", "hypothesis", "Memory leak", "--likelihood", "high"],
    );
    assert!(success);

    // Add tried (without --result, testing default)
    let (success, stdout, _) = run_xas(&dir, &["debug", "tried", "Restarted server"]);
    assert!(success);
    assert!(stdout.contains("Recorded attempt"));

    // Add suspect
    let (success, _, _) = run_xas(
        &dir,
        &["debug", "suspect", "src/cache.rs", "Unbounded cache"],
    );
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
    assert!(stdout.contains("archive"));
}

#[test]
fn test_cli_handoff_json_output() {
    let dir = TempDir::new().unwrap();
    run_xas(&dir, &["init"]);
    run_xas(&dir, &["whoami", "--set", "test-agent"]);

    let (success, stdout, _) = run_xas(
        &dir,
        &[
            "--json",
            "handoff",
            "--mode",
            "plan",
            "Machine readable event",
        ],
    );

    assert!(success);
    let payload: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(payload["event"], "handoff_created");
    assert_eq!(payload["mode"], "plan");
    assert_eq!(payload["summary"], "Machine readable event");
    assert!(payload["id"].as_str().unwrap().len() > 8);
    assert_eq!(payload["id_short"].as_str().unwrap().len(), 8);
}

#[test]
fn test_cli_no_auto_commit_does_not_advance_head() {
    let dir = TempDir::new().unwrap();
    let (ok, _, err) = run_git(&dir, &["init"]);
    assert!(ok, "{}", err);
    run_git(&dir, &["config", "user.email", "test@example.com"]);
    run_git(&dir, &["config", "user.name", "Test Agent"]);

    std::fs::write(dir.path().join("README.md"), "base\n").unwrap();
    run_git(&dir, &["add", "README.md"]);
    run_git(&dir, &["commit", "-m", "init"]);
    let (_, before_head, _) = run_git(&dir, &["rev-parse", "HEAD"]);

    run_xas(&dir, &["init"]);
    run_xas(&dir, &["whoami", "--set", "test-agent"]);

    std::fs::write(dir.path().join("unrelated.txt"), "keep staged\n").unwrap();
    run_git(&dir, &["add", "unrelated.txt"]);

    let (success, _, stderr) = run_xas(
        &dir,
        &[
            "--no-auto-commit",
            "handoff",
            "--mode",
            "plan",
            "No commit expected",
        ],
    );
    assert!(success, "{}", stderr);

    let (_, after_head, _) = run_git(&dir, &["rev-parse", "HEAD"]);
    assert_eq!(before_head.trim(), after_head.trim());
}

#[test]
fn test_cli_auto_commit_is_scoped_to_handoff_paths() {
    let dir = TempDir::new().unwrap();
    let (ok, _, err) = run_git(&dir, &["init"]);
    assert!(ok, "{}", err);
    run_git(&dir, &["config", "user.email", "test@example.com"]);
    run_git(&dir, &["config", "user.name", "Test Agent"]);

    std::fs::write(dir.path().join("README.md"), "base\n").unwrap();
    run_git(&dir, &["add", "README.md"]);
    run_git(&dir, &["commit", "-m", "init"]);
    let (_, before_head, _) = run_git(&dir, &["rev-parse", "HEAD"]);

    run_xas(&dir, &["init"]);
    run_xas(&dir, &["whoami", "--set", "test-agent"]);

    std::fs::write(dir.path().join("unrelated.txt"), "staged but unrelated\n").unwrap();
    run_git(&dir, &["add", "unrelated.txt"]);

    let (success, _, stderr) = run_xas(
        &dir,
        &["handoff", "--mode", "plan", "Scoped auto commit behavior"],
    );
    assert!(success, "{}", stderr);

    let (_, after_head, _) = run_git(&dir, &["rev-parse", "HEAD"]);
    assert_ne!(before_head.trim(), after_head.trim());

    let (_, commit_files, _) = run_git(&dir, &["show", "--name-only", "--pretty=format:", "HEAD"]);
    assert!(commit_files.contains("pending/"));
    assert!(!commit_files.contains("unrelated.txt"));

    let (_, status, _) = run_git(&dir, &["status", "--short"]);
    assert!(status.contains("A  unrelated.txt"));
}

#[test]
fn test_cli_archive_specific_handoff_with_json_output() {
    let dir = TempDir::new().unwrap();
    run_xas(&dir, &["init"]);
    run_xas(&dir, &["whoami", "--set", "test-agent"]);

    let (success, stdout, stderr) = run_xas(
        &dir,
        &["--json", "handoff", "--mode", "plan", "Archive me"],
    );
    assert!(success, "{}", stderr);
    let first: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let first_id_short = first["id_short"].as_str().unwrap().to_string();

    let (success, _, stderr) = run_xas(
        &dir,
        &["--json", "handoff", "--mode", "debug", "Keep me pending"],
    );
    assert!(success, "{}", stderr);

    let pending_before = std::fs::read_dir(dir.path().join("pending"))
        .unwrap()
        .filter_map(std::result::Result::ok)
        .count();
    let archive_before = std::fs::read_dir(dir.path().join("archive"))
        .unwrap()
        .filter_map(std::result::Result::ok)
        .count();
    assert_eq!(pending_before, 2);
    assert_eq!(archive_before, 0);

    let (success, stdout, stderr) = run_xas(&dir, &["--json", "archive", &first_id_short]);
    assert!(success, "{}", stderr);
    let archived: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(archived["event"], "handoff_archived");
    assert_eq!(archived["id_short"], first_id_short);
    assert_eq!(archived["summary"], "Archive me");
    assert!(archived["path"].as_str().unwrap().contains("archive/"));

    let pending_after = std::fs::read_dir(dir.path().join("pending"))
        .unwrap()
        .filter_map(std::result::Result::ok)
        .count();
    let archive_after = std::fs::read_dir(dir.path().join("archive"))
        .unwrap()
        .filter_map(std::result::Result::ok)
        .count();
    assert_eq!(pending_after, 1);
    assert_eq!(archive_after, 1);
}
