//! Integration tests for handoff creation and compilation

use xagentsync::{
    context::SessionState,
    handoff::{
        debug::{AttemptOutcome, DebugContext, Hypothesis, Likelihood},
        deploy::{Confidence, DeployContext, ShipItem},
        plan::{Decision, OpenQuestion, PlanContext, Priority, RejectedOption, Requirement},
    },
    GitRef, Handoff, HandoffMode, WarmUpSequence,
};

#[test]
fn test_deploy_handoff_creation() {
    let mut deploy = DeployContext::default();
    deploy.what_to_ship.push(ShipItem {
        item: "src/auth/*".to_string(),
        description: "OAuth2 implementation".to_string(),
        confidence: Confidence::High,
    });
    deploy.verification_steps.push("Run auth tests".to_string());
    deploy.rollback_plan = Some("git revert HEAD".to_string());

    let handoff = Handoff::new(
        HandoffMode::Deploy(deploy),
        "Ship OAuth feature",
        "test-agent",
    );

    assert_eq!(handoff.summary, "Ship OAuth feature");
    assert_eq!(handoff.created_by, "test-agent");
    assert!(handoff.mode.as_deploy().is_some());
}

#[test]
fn test_debug_handoff_creation() {
    let mut debug = DebugContext::new("Login failing after token refresh");
    debug.symptoms.push("500 error on callback".to_string());
    debug.hypotheses.push(Hypothesis {
        theory: "Race condition in refresh".to_string(),
        support: vec!["Timing dependent".to_string()],
        against: vec![],
        likelihood: Likelihood::High,
    });

    let handoff = Handoff::new(
        HandoffMode::Debug(debug),
        "Login failing after token refresh",
        "test-agent",
    );

    let ctx = handoff.mode.as_debug().unwrap();
    assert_eq!(ctx.problem_statement, "Login failing after token refresh");
    assert_eq!(ctx.symptoms.len(), 1);
    assert_eq!(ctx.hypotheses.len(), 1);
    assert_eq!(ctx.hypotheses[0].likelihood, Likelihood::High);
}

#[test]
fn test_plan_handoff_creation() {
    let mut plan = PlanContext::new("Design caching layer");
    plan.requirements.push(Requirement {
        description: "Sub-100ms p99".to_string(),
        priority: Priority::Must,
        source: None,
        confirmed: false,
    });
    plan.decisions.push(Decision {
        decision: "Use Redis".to_string(),
        rationale: "Team expertise".to_string(),
        context: None,
        reversible: true,
    });
    plan.rejected_options.push(RejectedOption {
        option: "Memcached".to_string(),
        reason: "No persistence".to_string(),
        reconsiderable: true,
    });
    plan.open_questions.push(OpenQuestion {
        question: "Cluster vs single?".to_string(),
        importance: "high".to_string(),
        ask_who: None,
        blocking: false,
    });

    let handoff = Handoff::new(
        HandoffMode::Plan(plan),
        "Design caching layer",
        "test-agent",
    );

    let ctx = handoff.mode.as_plan().unwrap();
    assert_eq!(ctx.goal, "Design caching layer");
    assert_eq!(ctx.requirements.len(), 1);
    assert_eq!(ctx.requirements[0].priority, Priority::Must);
    assert_eq!(ctx.decisions.len(), 1);
    assert_eq!(ctx.rejected_options.len(), 1);
    assert_eq!(ctx.open_questions.len(), 1);
}

#[test]
fn test_handoff_serialization_roundtrip() {
    let handoff = Handoff::new(
        HandoffMode::debug("Test problem"),
        "Test problem",
        "test-agent",
    );

    let json = handoff.to_json().expect("serialization should work");
    let restored = Handoff::from_json(&json).expect("deserialization should work");

    assert_eq!(handoff.id, restored.id);
    assert_eq!(handoff.summary, restored.summary);
    assert_eq!(handoff.created_by, restored.created_by);
}

#[test]
fn test_warm_up_sequence() {
    let warm_up = WarmUpSequence::new("Quick context")
        .with_file("src/main.rs", "Entry point", 1)
        .with_file("src/lib.rs", "Core types", 2)
        .must_know("Uses async/await throughout")
        .must_know("Redis connection is lazy")
        .suggest_start("Read the main handler first");

    assert_eq!(warm_up.tldr, "Quick context");
    assert_eq!(warm_up.priority_files.len(), 2);
    assert_eq!(warm_up.priority_files[0].rank, 1);
    assert_eq!(warm_up.must_know.len(), 2);
    assert!(warm_up.suggested_start.is_some());
}

#[test]
fn test_git_ref_types() {
    let commit = GitRef::commit("abc123");
    assert_eq!(commit.value, "abc123");

    let branch = GitRef::branch("feature/auth");
    assert_eq!(branch.value, "feature/auth");

    let pr = GitRef::pull_request("42");
    assert_eq!(pr.value, "42");
}

#[test]
fn test_session_state_builder() {
    let session = SessionState::new()
        .read_file("src/main.rs")
        .read_file_for("src/auth.rs", "Understanding auth flow")
        .modified_file("src/config.rs", "Added Redis settings")
        .created_file("src/cache/mod.rs")
        .ran_command("cargo test", true)
        .ran_command("cargo build", true)
        .gotcha("Redis connection must be established before auth middleware")
        .decided("Use connection pooling", "Performance under load")
        .dead_end("Tried sync Redis client", "Blocked async runtime");

    assert_eq!(session.files_read.len(), 2);
    assert_eq!(session.files_modified.len(), 1);
    assert_eq!(session.files_created.len(), 1);
    assert_eq!(session.commands_run.len(), 2);
    assert_eq!(session.observations.len(), 1);
    assert_eq!(session.decisions.len(), 1);
    assert_eq!(session.dead_ends.len(), 1);
}

#[test]
fn test_compile_prompt_deploy() {
    let mut deploy = DeployContext::default();
    deploy.what_to_ship.push(ShipItem {
        item: "auth module".to_string(),
        description: "New OAuth2 flow".to_string(),
        confidence: Confidence::High,
    });
    deploy.verification_steps.push("Run cargo test".to_string());

    let handoff = Handoff::new(
        HandoffMode::Deploy(deploy),
        "Ship auth",
        "claude",
    );

    let prompt = handoff.compile_prompt();

    assert!(prompt.contains("Ship auth"));
    assert!(prompt.contains("deploy"));
    assert!(prompt.contains("auth module"));
    assert!(prompt.contains("cargo test"));
}

#[test]
fn test_compile_prompt_debug() {
    let mut debug = DebugContext::new("API errors");
    debug.symptoms.push("500 on POST".to_string());
    debug.hypotheses.push(Hypothesis {
        theory: "Validation bug".to_string(),
        support: vec![],
        against: vec![],
        likelihood: Likelihood::High,
    });

    let handoff = Handoff::new(
        HandoffMode::Debug(debug),
        "API errors",
        "claude",
    );

    let prompt = handoff.compile_prompt();

    assert!(prompt.contains("API errors"));
    assert!(prompt.contains("debug"));
    assert!(prompt.contains("500 on POST"));
    assert!(prompt.contains("Validation bug"));
    assert!(prompt.contains("High"));
}

#[test]
fn test_compile_prompt_plan() {
    let mut plan = PlanContext::new("New feature");
    plan.requirements.push(Requirement {
        description: "Fast".to_string(),
        priority: Priority::Must,
        source: None,
        confirmed: false,
    });
    plan.decisions.push(Decision {
        decision: "Use Rust".to_string(),
        rationale: "Performance".to_string(),
        context: None,
        reversible: true,
    });

    let handoff = Handoff::new(
        HandoffMode::Plan(plan),
        "New feature",
        "claude",
    );

    let prompt = handoff.compile_prompt();

    assert!(prompt.contains("New feature"));
    assert!(prompt.contains("plan"));
    assert!(prompt.contains("Must"));
    assert!(prompt.contains("Fast"));
    assert!(prompt.contains("Use Rust"));
    assert!(prompt.contains("Performance"));
}

#[test]
fn test_attempt_outcomes() {
    let outcomes = vec![
        AttemptOutcome::Fixed,
        AttemptOutcome::Helped,
        AttemptOutcome::NoEffect,
        AttemptOutcome::MadeWorse,
    ];

    // Just ensure they can be created and compared
    assert_ne!(AttemptOutcome::Fixed, AttemptOutcome::MadeWorse);
    assert_eq!(outcomes.len(), 4);
}

#[test]
fn test_priority_ordering() {
    // Must > Should > Could > Wont
    let priorities = vec![
        Priority::Must,
        Priority::Should,
        Priority::Could,
        Priority::Wont,
    ];

    assert_eq!(priorities.len(), 4);
    // Ensure they're distinct
    assert_ne!(Priority::Must, Priority::Wont);
}

#[test]
fn test_handoff_with_full_context() {
    // Create a realistic handoff with all the bells and whistles
    let session = SessionState::new()
        .read_file("src/main.rs")
        .modified_file("src/auth.rs", "Added token refresh")
        .gotcha("Token refresh is async")
        .decided("Use JWT", "Standard, well-supported");

    let warm_up = WarmUpSequence::new("Auth system changes")
        .with_file("src/auth.rs", "Main changes here", 1)
        .must_know("Uses async refresh now")
        .suggest_start("Review the token_refresh function");

    let handoff = Handoff::new(
        HandoffMode::debug("Token refresh race condition"),
        "Token refresh race condition",
        "claude-opus",
    )
    .with_session(session)
    .with_warm_up(warm_up)
    .with_git_ref(GitRef::branch("fix/token-refresh"))
    .with_tag("auth")
    .with_tag("urgent");

    assert_eq!(handoff.tags.len(), 2);
    assert!(handoff.git_ref.is_some());
    assert!(!handoff.session.files_read.is_empty());
    assert!(!handoff.warm_up.priority_files.is_empty());

    // Ensure it serializes
    let json = handoff.to_json().unwrap();
    assert!(json.contains("token-refresh"));
    assert!(json.contains("urgent"));
}
