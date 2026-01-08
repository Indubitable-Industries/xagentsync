//! Context capture - tracking what an agent did during a session
//!
//! This module captures session state that helps the receiving agent
//! understand what happened and bootstrap efficiently.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session state - what the agent did during their work session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionState {
    /// When the session started
    pub started_at: Option<DateTime<Utc>>,

    /// When the session ended
    pub ended_at: Option<DateTime<Utc>>,

    /// Files that were read during the session
    pub files_read: Vec<FileRead>,

    /// Files that were modified
    pub files_modified: Vec<FileModified>,

    /// Files that were created
    pub files_created: Vec<String>,

    /// Commands/tools that were run
    pub commands_run: Vec<CommandRun>,

    /// Key observations made during the session
    pub observations: Vec<Observation>,

    /// Decisions made during the session
    pub decisions: Vec<SessionDecision>,

    /// Things that didn't work (negative knowledge)
    pub dead_ends: Vec<DeadEnd>,
}

/// A file that was read
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRead {
    /// Path to the file
    pub path: String,
    /// Why it was read (purpose)
    pub purpose: Option<String>,
    /// Key takeaways from reading it
    pub takeaways: Vec<String>,
    /// Order in which it was read (for warm-up sequencing)
    pub read_order: Option<u32>,
}

/// A file that was modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModified {
    /// Path to the file
    pub path: String,
    /// Brief summary of changes
    pub change_summary: Option<String>,
    /// Lines changed (approximate)
    pub lines_changed: Option<u32>,
}

/// A command or tool that was run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRun {
    /// The command
    pub command: String,
    /// What it was for
    pub purpose: Option<String>,
    /// Did it succeed?
    pub success: bool,
    /// Notable output (if any)
    pub notable_output: Option<String>,
}

/// An observation made during the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// The observation
    pub note: String,
    /// Category
    pub category: ObservationCategory,
    /// Importance (1-5)
    pub importance: u8,
}

/// Category of observation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObservationCategory {
    #[default]
    General,
    Pattern,
    Gotcha,
    Insight,
    Question,
    Risk,
}

/// A decision made during the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDecision {
    /// What was decided
    pub decision: String,
    /// Why (brief rationale)
    pub why: String,
    /// Alternatives considered
    pub alternatives: Vec<String>,
}

/// Something that was tried but didn't work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadEnd {
    /// What was tried
    pub approach: String,
    /// Why it didn't work
    pub reason: String,
    /// Should this be revisited?
    pub revisit: bool,
}

impl SessionState {
    /// Create a new session state
    pub fn new() -> Self {
        Self {
            started_at: Some(Utc::now()),
            ..Default::default()
        }
    }

    /// Record a file read
    pub fn read_file(mut self, path: impl Into<String>) -> Self {
        let order = self.files_read.len() as u32 + 1;
        self.files_read.push(FileRead {
            path: path.into(),
            purpose: None,
            takeaways: Vec::new(),
            read_order: Some(order),
        });
        self
    }

    /// Record a file read with purpose
    pub fn read_file_for(mut self, path: impl Into<String>, purpose: impl Into<String>) -> Self {
        let order = self.files_read.len() as u32 + 1;
        self.files_read.push(FileRead {
            path: path.into(),
            purpose: Some(purpose.into()),
            takeaways: Vec::new(),
            read_order: Some(order),
        });
        self
    }

    /// Record a file modification
    pub fn modified_file(mut self, path: impl Into<String>, summary: impl Into<String>) -> Self {
        self.files_modified.push(FileModified {
            path: path.into(),
            change_summary: Some(summary.into()),
            lines_changed: None,
        });
        self
    }

    /// Record a created file
    pub fn created_file(mut self, path: impl Into<String>) -> Self {
        self.files_created.push(path.into());
        self
    }

    /// Record a command run
    pub fn ran_command(mut self, command: impl Into<String>, success: bool) -> Self {
        self.commands_run.push(CommandRun {
            command: command.into(),
            purpose: None,
            success,
            notable_output: None,
        });
        self
    }

    /// Record an observation
    pub fn observed(mut self, note: impl Into<String>, category: ObservationCategory, importance: u8) -> Self {
        self.observations.push(Observation {
            note: note.into(),
            category,
            importance: importance.min(5),
        });
        self
    }

    /// Record a gotcha (important observation about something tricky)
    pub fn gotcha(mut self, note: impl Into<String>) -> Self {
        self.observations.push(Observation {
            note: note.into(),
            category: ObservationCategory::Gotcha,
            importance: 4,
        });
        self
    }

    /// Record a decision
    pub fn decided(mut self, decision: impl Into<String>, why: impl Into<String>) -> Self {
        self.decisions.push(SessionDecision {
            decision: decision.into(),
            why: why.into(),
            alternatives: Vec::new(),
        });
        self
    }

    /// Record a dead end
    pub fn dead_end(mut self, approach: impl Into<String>, reason: impl Into<String>) -> Self {
        self.dead_ends.push(DeadEnd {
            approach: approach.into(),
            reason: reason.into(),
            revisit: false,
        });
        self
    }

    /// End the session
    pub fn end(mut self) -> Self {
        self.ended_at = Some(Utc::now());
        self
    }

    /// Get files in read order for warm-up sequencing
    pub fn files_by_read_order(&self) -> Vec<&FileRead> {
        let mut files: Vec<_> = self.files_read.iter().collect();
        files.sort_by_key(|f| f.read_order.unwrap_or(999));
        files
    }

    /// Get high-importance observations
    pub fn important_observations(&self) -> Vec<&Observation> {
        self.observations
            .iter()
            .filter(|o| o.importance >= 3)
            .collect()
    }

    /// Generate a summary of the session
    pub fn summarize(&self) -> String {
        let mut summary = String::new();

        if !self.files_modified.is_empty() {
            summary.push_str(&format!("Modified {} files. ", self.files_modified.len()));
        }
        if !self.files_created.is_empty() {
            summary.push_str(&format!("Created {} files. ", self.files_created.len()));
        }
        if !self.decisions.is_empty() {
            summary.push_str(&format!("Made {} decisions. ", self.decisions.len()));
        }
        if !self.dead_ends.is_empty() {
            summary.push_str(&format!("{} dead ends noted. ", self.dead_ends.len()));
        }

        if summary.is_empty() {
            summary = "Exploratory session.".to_string();
        }

        summary.trim().to_string()
    }
}
