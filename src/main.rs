//! XAgentSync - Async handoff protocol for LLM code assistants
//!
//! CLI tool for creating structured handoffs between LLM agents
//! working asynchronously on shared codebases.

use xagentsync::{
    cli::{Cli, Commands, DeployAction, DebugAction, HandoffModeArg, PlanAction},
    handoff::{
        deploy::{Confidence, ShipItem},
        debug::{AttemptOutcome, EvidenceKind, Likelihood},
        plan::Priority,
    },
    GitRef, Handoff, HandoffMode, PriorityFile, Result, WarmUpSequence,
    sync::{SyncConfig, SyncManager},
};
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();

    // Set up logging
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    // Execute command
    match cli.command {
        Commands::Init { path } => cmd_init(path).await,
        Commands::Handoff {
            mode,
            summary,
            priority_files,
            must_know,
            suggest_start,
            commit,
            branch,
            pr,
            tags,
            interactive: _,
        } => {
            cmd_handoff(
                &cli.sync_dir,
                mode,
                summary,
                priority_files,
                must_know,
                suggest_start,
                commit,
                branch,
                pr,
                tags,
            )
            .await
        }
        Commands::Receive { prompt, mode, full, archive } => {
            cmd_receive(&cli.sync_dir, prompt, mode, full, archive).await
        }
        Commands::Whoami { set } => cmd_whoami(&cli.sync_dir, set).await,
        Commands::Status => cmd_status(&cli.sync_dir).await,
        Commands::Deploy { action } => cmd_deploy(&cli.sync_dir, action).await,
        Commands::Debug { action } => cmd_debug(&cli.sync_dir, action).await,
        Commands::Plan { action } => cmd_plan(&cli.sync_dir, action).await,
        Commands::Sync { pull_only } => cmd_sync(&cli.sync_dir, pull_only).await,
    }
}

async fn cmd_init(path: PathBuf) -> Result<()> {
    let config = SyncConfig::with_sync_dir(&path);
    let manager = SyncManager::new(config)?;
    manager.init()?;

    println!("Initialized XAgentSync at {:?}", path);
    println!("  pending/  - handoffs waiting to be processed");
    println!("  archive/  - processed handoffs");
    println!("  .xas/     - local state (gitignored)");
    println!();
    println!("Next: Set your identity with 'xas whoami --set <your-name>'");

    Ok(())
}

async fn cmd_handoff(
    sync_dir: &PathBuf,
    mode: HandoffModeArg,
    summary: String,
    priority_files: Vec<String>,
    must_know: Vec<String>,
    suggest_start: Option<String>,
    commit: Option<String>,
    branch: Option<String>,
    pr: Option<String>,
    tags: Option<String>,
) -> Result<()> {
    let config = SyncConfig::with_sync_dir(sync_dir);
    let manager = SyncManager::new(config)?;

    let creator = get_current_agent(sync_dir)?;

    // Build the mode
    let handoff_mode = match mode {
        HandoffModeArg::Deploy => HandoffMode::deploy(),
        HandoffModeArg::Debug => HandoffMode::debug(&summary),
        HandoffModeArg::Plan => HandoffMode::plan(&summary),
    };

    // Build warm-up sequence
    let mut warm_up = WarmUpSequence::new(&summary);
    for (i, file) in priority_files.iter().enumerate() {
        warm_up.priority_files.push(PriorityFile {
            path: file.clone(),
            reason: "Priority file".to_string(),
            focus: None,
            rank: (i + 1) as u8,
        });
    }
    warm_up.must_know = must_know;
    warm_up.suggested_start = suggest_start;

    // Build handoff
    let mut handoff = Handoff::new(handoff_mode, &summary, &creator).with_warm_up(warm_up);

    // Attach git ref
    if let Some(sha) = commit {
        handoff = handoff.with_git_ref(GitRef::commit(sha));
    } else if let Some(br) = branch {
        handoff = handoff.with_git_ref(GitRef::branch(br));
    } else if let Some(p) = pr {
        handoff = handoff.with_git_ref(GitRef::pull_request(p));
    } else if let Some(sha) = manager.current_commit() {
        handoff = handoff.with_git_ref(GitRef::commit(&sha[..8]));
    }

    // Add tags
    if let Some(tag_str) = tags {
        for tag in tag_str.split(',') {
            handoff = handoff.with_tag(tag.trim());
        }
    }

    // Send it
    let path = manager.send_handoff(&handoff)?;

    println!("Handoff created: {}", handoff.id);
    println!("  Mode: {}", handoff.mode);
    println!("  Summary: {}", handoff.summary);
    println!("  Written to: {:?}", path);

    Ok(())
}

async fn cmd_receive(
    sync_dir: &PathBuf,
    show_prompt: bool,
    mode_filter: Option<HandoffModeArg>,
    full: bool,
    archive: bool,
) -> Result<()> {
    let config = SyncConfig::with_sync_dir(sync_dir);
    let manager = SyncManager::new(config)?;

    let handoffs = manager.receive_handoffs()?;

    if handoffs.is_empty() {
        println!("No pending handoffs in inbox.");
        return Ok(());
    }

    // Filter by mode if requested
    let handoffs: Vec<_> = handoffs
        .into_iter()
        .filter(|h| {
            mode_filter
                .as_ref()
                .map_or(true, |m| h.mode.kind() == m.to_string())
        })
        .collect();

    println!("Found {} handoff(s):\n", handoffs.len());

    for handoff in &handoffs {
        if show_prompt {
            // Show the compiled prompt, ready to paste
            println!("═══════════════════════════════════════════════════════════════");
            println!("{}", handoff.compile_prompt());
            println!("═══════════════════════════════════════════════════════════════\n");
        } else {
            // Show summary
            println!(
                "[{}] {} - {}",
                handoff.mode.kind().to_uppercase(),
                &handoff.id.to_string()[..8],
                handoff.summary
            );
            println!("  From: {}", handoff.created_by);
            println!("  Created: {}", handoff.created_at.format("%Y-%m-%d %H:%M"));

            if let Some(ref git) = handoff.git_ref {
                println!("  Git: {:?} {}", git.ref_type, git.value);
            }

            if full {
                println!("  TL;DR: {}", handoff.warm_up.tldr);
                if !handoff.warm_up.must_know.is_empty() {
                    println!("  Must know:");
                    for item in &handoff.warm_up.must_know {
                        println!("    - {}", item);
                    }
                }
            }
            println!();
        }

        if archive {
            manager.archive_handoff(&handoff.id.to_string()[..8])?;
            println!("  (archived)");
        }
    }

    if !show_prompt && !handoffs.is_empty() {
        println!("Use --prompt to see the full compiled handoff prompt.");
    }

    Ok(())
}

async fn cmd_whoami(sync_dir: &PathBuf, set: Option<String>) -> Result<()> {
    let config = SyncConfig::with_sync_dir(sync_dir);
    let manager = SyncManager::new(config)?;

    if let Some(id) = set {
        manager.write_state("current_agent", &id)?;
        println!("Set identity to: {}", id);
    } else {
        match get_current_agent(sync_dir) {
            Ok(id) => println!("Current identity: {}", id),
            Err(_) => println!("No identity set. Use 'xas whoami --set <your-name>'"),
        }
    }

    Ok(())
}

async fn cmd_status(sync_dir: &PathBuf) -> Result<()> {
    let config = SyncConfig::with_sync_dir(sync_dir);
    let manager = SyncManager::new(config)?;

    // Identity
    match get_current_agent(sync_dir) {
        Ok(id) => println!("Identity: {}", id),
        Err(_) => println!("Identity: (not set)"),
    }

    // Git info
    if let Some(branch) = manager.current_branch() {
        print!("Branch: {}", branch);
        if let Some(commit) = manager.current_commit() {
            print!(" ({})", &commit[..8]);
        }
        println!();
    }

    // Pending handoffs
    let handoffs = manager.receive_handoffs()?;
    if !handoffs.is_empty() {
        println!("\nPending handoffs: {}", handoffs.len());
        for h in &handoffs {
            println!(
                "  [{}] {} - {}",
                h.mode.kind(),
                &h.id.to_string()[..8],
                h.summary
            );
        }
    } else {
        println!("\nNo pending handoffs.");
    }

    // WIP
    if let Ok(Some(wip)) = manager.load_wip() {
        println!("\nWork in progress: [{}] {}", wip.mode.kind(), wip.summary);
    }

    Ok(())
}

async fn cmd_deploy(sync_dir: &PathBuf, action: DeployAction) -> Result<()> {
    let config = SyncConfig::with_sync_dir(sync_dir);
    let manager = SyncManager::new(config)?;

    match action {
        DeployAction::New { summary } => {
            let creator = get_current_agent(sync_dir)?;
            let handoff = Handoff::new(HandoffMode::deploy(), &summary, &creator);
            manager.save_wip(&handoff)?;
            println!("Started deploy handoff: {}", summary);
            println!("Use 'xas deploy ship', 'xas deploy verify', etc. to add details.");
            println!("Use 'xas deploy done' to finalize.");
        }

        DeployAction::Ship { item, description } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_deploy_mut() {
                ctx.what_to_ship.push(ShipItem {
                    item: item.clone(),
                    description: description.unwrap_or_else(|| item.clone()),
                    confidence: Confidence::Medium,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Added to ship: {}", item);
        }

        DeployAction::Verify { step } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_deploy_mut() {
                ctx.verification_steps.push(step.clone());
            }
            manager.save_wip(&handoff)?;
            println!("Added verification step: {}", step);
        }

        DeployAction::Rollback { plan } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_deploy_mut() {
                ctx.rollback_plan = Some(plan.clone());
            }
            manager.save_wip(&handoff)?;
            println!("Set rollback plan.");
        }

        DeployAction::EnvConcern { env, concern } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_deploy_mut() {
                ctx.env_concerns.push(xagentsync::handoff::deploy::EnvConcern {
                    environment: env.clone(),
                    concern: concern.clone(),
                    mitigation: None,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Added {} concern: {}", env, concern);
        }

        DeployAction::Breaking { what, affects } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_deploy_mut() {
                ctx.breaking_changes.push(xagentsync::handoff::deploy::BreakingChange {
                    what: what.clone(),
                    affects: affects.clone(),
                    migration: None,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Added breaking change: {} affects {}", what, affects);
        }

        DeployAction::Done => {
            let handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            let path = manager.send_handoff(&handoff)?;
            manager.clear_wip()?;
            println!("Deploy handoff finalized: {:?}", path);
        }
    }

    Ok(())
}

async fn cmd_debug(sync_dir: &PathBuf, action: DebugAction) -> Result<()> {
    let config = SyncConfig::with_sync_dir(sync_dir);
    let manager = SyncManager::new(config)?;

    match action {
        DebugAction::New { problem } => {
            let creator = get_current_agent(sync_dir)?;
            let handoff = Handoff::new(HandoffMode::debug(&problem), &problem, &creator);
            manager.save_wip(&handoff)?;
            println!("Started debug handoff: {}", problem);
            println!("Use 'xas debug symptom', 'xas debug tried', etc. to add details.");
        }

        DebugAction::Symptom { symptom } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_debug_mut() {
                ctx.symptoms.push(symptom.clone());
            }
            manager.save_wip(&handoff)?;
            println!("Added symptom: {}", symptom);
        }

        DebugAction::Hypothesis { theory, likelihood } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            let lh = match likelihood.to_lowercase().as_str() {
                "high" => Likelihood::High,
                "low" => Likelihood::Low,
                _ => Likelihood::Medium,
            };
            if let Some(ctx) = handoff.mode.as_debug_mut() {
                ctx.hypotheses.push(xagentsync::handoff::debug::Hypothesis {
                    theory: theory.clone(),
                    support: Vec::new(),
                    against: Vec::new(),
                    likelihood: lh,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Added hypothesis: {}", theory);
        }

        DebugAction::Tried { what, result, outcome } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            let oc = match outcome.to_lowercase().as_str() {
                "fixed" => AttemptOutcome::Fixed,
                "helped" => AttemptOutcome::Helped,
                "worse" => AttemptOutcome::MadeWorse,
                _ => AttemptOutcome::NoEffect,
            };
            if let Some(ctx) = handoff.mode.as_debug_mut() {
                ctx.attempted.push(xagentsync::handoff::debug::Attempt {
                    what: what.clone(),
                    result: result.clone(),
                    outcome: oc,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Recorded attempt: {}", what);
        }

        DebugAction::Evidence { content, kind } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            let k = match kind.to_lowercase().as_str() {
                "log" => EvidenceKind::LogEntry,
                "error" => EvidenceKind::ErrorMessage,
                "stack" | "stacktrace" => EvidenceKind::StackTrace,
                _ => EvidenceKind::Observation,
            };
            if let Some(ctx) = handoff.mode.as_debug_mut() {
                ctx.evidence.push(xagentsync::handoff::debug::Evidence {
                    kind: k,
                    content: content.clone(),
                    source: None,
                    timestamp: None,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Added evidence.");
        }

        DebugAction::Suspect { path, reason } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_debug_mut() {
                ctx.suspected_files.push(xagentsync::handoff::debug::SuspectedFile {
                    path: path.clone(),
                    reason: reason.clone(),
                    lines: None,
                    confidence: Likelihood::Medium,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Added suspect file: {}", path);
        }

        DebugAction::Repro { steps } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_debug_mut() {
                ctx.reproduction_steps = Some(steps.clone());
            }
            manager.save_wip(&handoff)?;
            println!("Set reproduction steps.");
        }

        DebugAction::TryNext { next } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_debug_mut() {
                ctx.next_to_try = Some(next.clone());
            }
            manager.save_wip(&handoff)?;
            println!("Set next step: {}", next);
        }

        DebugAction::Done => {
            let handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            let path = manager.send_handoff(&handoff)?;
            manager.clear_wip()?;
            println!("Debug handoff finalized: {:?}", path);
        }
    }

    Ok(())
}

async fn cmd_plan(sync_dir: &PathBuf, action: PlanAction) -> Result<()> {
    let config = SyncConfig::with_sync_dir(sync_dir);
    let manager = SyncManager::new(config)?;

    match action {
        PlanAction::New { goal } => {
            let creator = get_current_agent(sync_dir)?;
            let handoff = Handoff::new(HandoffMode::plan(&goal), &goal, &creator);
            manager.save_wip(&handoff)?;
            println!("Started plan handoff: {}", goal);
            println!("Use 'xas plan require', 'xas plan decided', etc. to add details.");
        }

        PlanAction::Require { requirement, priority } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            let p = match priority.to_lowercase().as_str() {
                "must" => Priority::Must,
                "could" => Priority::Could,
                "wont" => Priority::Wont,
                _ => Priority::Should,
            };
            if let Some(ctx) = handoff.mode.as_plan_mut() {
                ctx.requirements.push(xagentsync::handoff::plan::Requirement {
                    description: requirement.clone(),
                    priority: p,
                    source: None,
                    confirmed: false,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Added requirement: {}", requirement);
        }

        PlanAction::Decided { decision, why } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_plan_mut() {
                ctx.decisions.push(xagentsync::handoff::plan::Decision {
                    decision: decision.clone(),
                    rationale: why.clone(),
                    context: None,
                    reversible: true,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Recorded decision: {}", decision);
        }

        PlanAction::Rejected { option, reason } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_plan_mut() {
                ctx.rejected_options.push(xagentsync::handoff::plan::RejectedOption {
                    option: option.clone(),
                    reason: reason.clone(),
                    reconsiderable: true,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Recorded rejected option: {}", option);
        }

        PlanAction::Question { question, importance, blocking } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_plan_mut() {
                ctx.open_questions.push(xagentsync::handoff::plan::OpenQuestion {
                    question: question.clone(),
                    importance: importance.clone(),
                    ask_who: None,
                    blocking,
                });
            }
            manager.save_wip(&handoff)?;
            let bl = if blocking { " (blocking)" } else { "" };
            println!("Added question{}: {}", bl, question);
        }

        PlanAction::Constraint { constraint } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_plan_mut() {
                ctx.constraints.push(xagentsync::handoff::plan::Constraint {
                    constraint: constraint.clone(),
                    reason: None,
                    negotiable: false,
                });
            }
            manager.save_wip(&handoff)?;
            println!("Added constraint: {}", constraint);
        }

        PlanAction::NextStep { step } => {
            let mut handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            if let Some(ctx) = handoff.mode.as_plan_mut() {
                ctx.next_steps.push(step.clone());
            }
            manager.save_wip(&handoff)?;
            println!("Added next step: {}", step);
        }

        PlanAction::Done => {
            let handoff = manager.load_wip()?.ok_or(xagentsync::Error::NoActiveHandoff)?;
            let path = manager.send_handoff(&handoff)?;
            manager.clear_wip()?;
            println!("Plan handoff finalized: {:?}", path);
        }
    }

    Ok(())
}

async fn cmd_sync(sync_dir: &PathBuf, pull_only: bool) -> Result<()> {
    let config = SyncConfig::with_sync_dir(sync_dir);
    let manager = SyncManager::new(config)?;

    println!("Pulling latest...");
    manager.pull()?;

    if !pull_only {
        println!("Committing local changes...");
        manager.commit_changes("XAgentSync sync")?;
    }

    println!("Done.");
    Ok(())
}

/// Get the current agent ID from state
fn get_current_agent(sync_dir: &PathBuf) -> Result<String> {
    let config = SyncConfig::with_sync_dir(sync_dir);
    let manager = SyncManager::new(config)?;

    manager
        .read_state::<String>("current_agent")?
        .ok_or_else(|| {
            xagentsync::Error::AgentNotRegistered(
                "No identity set. Use 'xas whoami --set <name>'".to_string(),
            )
        })
}
