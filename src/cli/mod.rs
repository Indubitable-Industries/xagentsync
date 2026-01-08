//! CLI commands and argument parsing

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// XAgentSync - Async handoff protocol for LLM code assistants
#[derive(Parser, Debug)]
#[command(name = "xas")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to sync directory
    #[arg(short, long, default_value = ".")]
    pub sync_dir: PathBuf,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Subcommand to run
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new sync directory
    Init {
        /// Path to initialize (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Create a handoff for the next agent
    Handoff {
        /// The handoff mode
        #[arg(short, long, value_enum)]
        mode: HandoffModeArg,

        /// Summary of the handoff (the "subject line")
        summary: String,

        /// Add a priority file to read first
        #[arg(long = "file", short = 'f')]
        priority_files: Vec<String>,

        /// Add a must-know item
        #[arg(long = "know", short = 'k')]
        must_know: Vec<String>,

        /// Suggested first action for receiving agent
        #[arg(long)]
        suggest_start: Option<String>,

        /// Attach to a git commit
        #[arg(long)]
        commit: Option<String>,

        /// Attach to a git branch
        #[arg(long)]
        branch: Option<String>,

        /// Attach to a PR number
        #[arg(long)]
        pr: Option<String>,

        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Open editor to fill in details interactively
        #[arg(long, short = 'i')]
        interactive: bool,
    },

    /// Receive and display pending handoffs
    Receive {
        /// Show the compiled prompt (ready to paste to receiving agent)
        #[arg(long, short = 'p')]
        prompt: bool,

        /// Filter by mode
        #[arg(long, short = 'm')]
        mode: Option<HandoffModeArg>,

        /// Show full details
        #[arg(long, short = 'f')]
        full: bool,

        /// Archive handoff after viewing
        #[arg(long)]
        archive: bool,
    },

    /// Set your agent identity
    Whoami {
        /// Set the current agent ID
        #[arg(long)]
        set: Option<String>,
    },

    /// Show sync status
    Status,

    /// Deploy mode helpers
    Deploy {
        #[command(subcommand)]
        action: DeployAction,
    },

    /// Debug mode helpers
    Debug {
        #[command(subcommand)]
        action: DebugAction,
    },

    /// Plan mode helpers
    Plan {
        #[command(subcommand)]
        action: PlanAction,
    },

    /// Sync with remote (git pull/push)
    Sync {
        /// Only pull, don't push
        #[arg(long)]
        pull_only: bool,
    },
}

/// Handoff mode argument
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HandoffModeArg {
    /// Deployment - focused on shipping
    Deploy,
    /// Debug/Troubleshoot - focused on fixing
    Debug,
    /// Planning - focused on designing
    Plan,
}

/// Deploy mode subcommands
#[derive(Subcommand, Debug)]
pub enum DeployAction {
    /// Start a new deploy handoff interactively
    New {
        /// Summary
        summary: String,
    },

    /// Add something to ship
    Ship {
        /// What to ship
        item: String,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Add a verification step
    Verify {
        /// Verification step
        step: String,
    },

    /// Set the rollback plan
    Rollback {
        /// Rollback plan
        plan: String,
    },

    /// Add an environment concern
    EnvConcern {
        /// Environment (prod, staging, etc)
        env: String,
        /// The concern
        concern: String,
    },

    /// Add a breaking change warning
    Breaking {
        /// What breaks
        what: String,
        /// What it affects
        affects: String,
    },

    /// Finalize and create the handoff
    Done,
}

/// Debug mode subcommands
#[derive(Subcommand, Debug)]
pub enum DebugAction {
    /// Start a new debug handoff with problem statement
    New {
        /// The problem statement
        problem: String,
    },

    /// Add a symptom
    Symptom {
        /// The symptom
        symptom: String,
    },

    /// Add a hypothesis
    Hypothesis {
        /// The theory
        theory: String,
        /// Likelihood (high, medium, low)
        #[arg(short, long, default_value = "medium")]
        likelihood: String,
    },

    /// Record something that was tried
    Tried {
        /// What was tried
        what: String,
        /// What happened
        #[arg(short, long, default_value = "No result captured")]
        result: String,
        /// Outcome (fixed, helped, nothing, worse)
        #[arg(short, long, default_value = "nothing")]
        outcome: String,
    },

    /// Add evidence
    Evidence {
        /// The evidence content
        content: String,
        /// Type (log, error, observation, etc)
        #[arg(short, long, default_value = "observation")]
        kind: String,
    },

    /// Add a suspected file
    Suspect {
        /// File path
        path: String,
        /// Why it's suspected
        reason: String,
    },

    /// Set reproduction steps
    Repro {
        /// Steps to reproduce
        steps: String,
    },

    /// Set what to try next
    TryNext {
        /// What the next agent should try
        next: String,
    },

    /// Finalize and create the handoff
    Done,
}

/// Plan mode subcommands
#[derive(Subcommand, Debug)]
pub enum PlanAction {
    /// Start a new plan handoff with goal
    New {
        /// The goal
        goal: String,
    },

    /// Add a requirement
    Require {
        /// The requirement
        requirement: String,
        /// Priority (must, should, could, wont)
        #[arg(short, long, default_value = "should")]
        priority: String,
    },

    /// Record a decision
    Decided {
        /// What was decided
        decision: String,
        /// Why (optional, can be added later)
        #[arg(short, long, default_value = "")]
        why: String,
    },

    /// Record a rejected option
    Rejected {
        /// The option that was rejected
        option: String,
        /// Why it was rejected
        reason: String,
    },

    /// Add an open question
    Question {
        /// The question
        question: String,
        /// Why it matters (high, medium, low)
        #[arg(short, long, default_value = "medium")]
        importance: String,
        /// Is it blocking?
        #[arg(long)]
        blocking: bool,
    },

    /// Add a constraint
    Constraint {
        /// The constraint
        constraint: String,
    },

    /// Add a next step
    NextStep {
        /// The step
        step: String,
    },

    /// Finalize and create the handoff
    Done,
}

impl Cli {
    /// Parse CLI arguments
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

impl std::fmt::Display for HandoffModeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandoffModeArg::Deploy => write!(f, "deploy"),
            HandoffModeArg::Debug => write!(f, "debug"),
            HandoffModeArg::Plan => write!(f, "plan"),
        }
    }
}
