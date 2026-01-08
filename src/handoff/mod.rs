//! Handoff - The core unit of async agent collaboration
//!
//! A handoff is a structured package that enables one agent to efficiently
//! transfer work context to another agent, minimizing cold-start penalty.

mod mode;
pub mod deploy;
pub mod debug;
pub mod plan;

pub use mode::HandoffMode;
pub use deploy::DeployContext;
pub use debug::DebugContext;
pub use plan::PlanContext;

use crate::context::SessionState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A handoff package for async agent collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handoff {
    /// Unique identifier
    pub id: Uuid,

    /// The mode determines what context is prioritized
    pub mode: HandoffMode,

    /// Who created this handoff
    pub created_by: String,

    /// When this handoff was created
    pub created_at: DateTime<Utc>,

    /// Brief summary (the "subject line")
    pub summary: String,

    /// Session state - what the creating agent did
    pub session: SessionState,

    /// Compiled warm-up sequence for receiving agent
    pub warm_up: WarmUpSequence,

    /// Git reference (commit, branch, PR) this relates to
    pub git_ref: Option<GitRef>,

    /// Tags for filtering/organization
    pub tags: Vec<String>,
}

/// Reference to a git object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRef {
    /// Type of reference
    pub ref_type: GitRefType,
    /// The reference value (SHA, branch name, PR number)
    pub value: String,
    /// Optional remote URL
    pub remote: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitRefType {
    Commit,
    Branch,
    PullRequest,
    Tag,
}

/// Warm-up sequence to bootstrap the receiving agent
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WarmUpSequence {
    /// Files to read first, in priority order
    pub priority_files: Vec<PriorityFile>,

    /// TL;DR - the essential context in minimal tokens
    pub tldr: String,

    /// Key things the receiving agent must know
    pub must_know: Vec<String>,

    /// Suggested first action
    pub suggested_start: Option<String>,

    /// Estimated context tokens needed for full understanding
    pub estimated_tokens: Option<u32>,
}

/// A file with priority information for warm-up
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityFile {
    /// Path to the file
    pub path: String,

    /// Why this file matters
    pub reason: String,

    /// Specific lines/sections to focus on (optional)
    pub focus: Option<String>,

    /// Priority rank (1 = highest)
    pub rank: u8,
}

impl Handoff {
    /// Create a new handoff
    pub fn new(
        mode: HandoffMode,
        summary: impl Into<String>,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            mode,
            created_by: created_by.into(),
            created_at: Utc::now(),
            summary: summary.into(),
            session: SessionState::default(),
            warm_up: WarmUpSequence::default(),
            git_ref: None,
            tags: Vec::new(),
        }
    }

    /// Set the session state
    pub fn with_session(mut self, session: SessionState) -> Self {
        self.session = session;
        self
    }

    /// Set the warm-up sequence
    pub fn with_warm_up(mut self, warm_up: WarmUpSequence) -> Self {
        self.warm_up = warm_up;
        self
    }

    /// Attach a git reference
    pub fn with_git_ref(mut self, git_ref: GitRef) -> Self {
        self.git_ref = Some(git_ref);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Compile the handoff into a prompt for the receiving agent
    pub fn compile_prompt(&self) -> String {
        let mut prompt = String::new();

        // Header
        prompt.push_str(&format!("# Handoff: {}\n\n", self.summary));
        prompt.push_str(&format!("**Mode**: {:?}\n", self.mode.kind()));
        prompt.push_str(&format!("**From**: {}\n", self.created_by));
        prompt.push_str(&format!("**Created**: {}\n\n", self.created_at.format("%Y-%m-%d %H:%M UTC")));

        // TL;DR
        if !self.warm_up.tldr.is_empty() {
            prompt.push_str("## TL;DR\n\n");
            prompt.push_str(&self.warm_up.tldr);
            prompt.push_str("\n\n");
        }

        // Mode-specific context
        prompt.push_str(&self.mode.compile_section());

        // Must know
        if !self.warm_up.must_know.is_empty() {
            prompt.push_str("## Must Know\n\n");
            for item in &self.warm_up.must_know {
                prompt.push_str(&format!("- {}\n", item));
            }
            prompt.push_str("\n");
        }

        // Priority files
        if !self.warm_up.priority_files.is_empty() {
            prompt.push_str("## Start Here (Priority Files)\n\n");
            for pf in &self.warm_up.priority_files {
                prompt.push_str(&format!("{}. `{}` - {}\n", pf.rank, pf.path, pf.reason));
                if let Some(ref focus) = pf.focus {
                    prompt.push_str(&format!("   Focus: {}\n", focus));
                }
            }
            prompt.push_str("\n");
        }

        // Suggested start
        if let Some(ref start) = self.warm_up.suggested_start {
            prompt.push_str("## Suggested First Action\n\n");
            prompt.push_str(start);
            prompt.push_str("\n\n");
        }

        // Session summary
        if !self.session.files_read.is_empty() || !self.session.files_modified.is_empty() {
            prompt.push_str("## Previous Session Activity\n\n");
            if !self.session.files_modified.is_empty() {
                prompt.push_str("**Modified**:\n");
                for f in &self.session.files_modified {
                    prompt.push_str(&format!("- `{}`", f.path));
                    if let Some(ref note) = f.change_summary {
                        prompt.push_str(&format!(" - {}", note));
                    }
                    prompt.push_str("\n");
                }
            }
            prompt.push_str("\n");
        }

        // Git ref
        if let Some(ref git) = self.git_ref {
            prompt.push_str(&format!("**Git {:?}**: `{}`\n", git.ref_type, git.value));
        }

        prompt
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl WarmUpSequence {
    /// Create a new warm-up sequence
    pub fn new(tldr: impl Into<String>) -> Self {
        Self {
            tldr: tldr.into(),
            ..Default::default()
        }
    }

    /// Add a priority file
    pub fn with_file(mut self, path: impl Into<String>, reason: impl Into<String>, rank: u8) -> Self {
        self.priority_files.push(PriorityFile {
            path: path.into(),
            reason: reason.into(),
            focus: None,
            rank,
        });
        self
    }

    /// Add a must-know item
    pub fn must_know(mut self, item: impl Into<String>) -> Self {
        self.must_know.push(item.into());
        self
    }

    /// Set suggested start action
    pub fn suggest_start(mut self, action: impl Into<String>) -> Self {
        self.suggested_start = Some(action.into());
        self
    }
}

impl GitRef {
    pub fn commit(sha: impl Into<String>) -> Self {
        Self {
            ref_type: GitRefType::Commit,
            value: sha.into(),
            remote: None,
        }
    }

    pub fn branch(name: impl Into<String>) -> Self {
        Self {
            ref_type: GitRefType::Branch,
            value: name.into(),
            remote: None,
        }
    }

    pub fn pull_request(number: impl Into<String>) -> Self {
        Self {
            ref_type: GitRefType::PullRequest,
            value: number.into(),
            remote: None,
        }
    }
}
