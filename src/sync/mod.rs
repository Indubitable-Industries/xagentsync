//! Sync - Git-based synchronization for handoffs
//!
//! Handles syncing handoffs through shared git repositories.

use crate::{Handoff, Result};
use git2::Repository;
use std::path::PathBuf;
use tracing::{debug, info};

/// Configuration for sync operations
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Path to the sync directory (usually a git repo)
    pub sync_dir: PathBuf,

    /// Subdirectory for pending handoffs (shared by all agents)
    pub pending: PathBuf,

    /// Subdirectory for state files (agent identity, work in progress)
    pub state: PathBuf,

    /// Subdirectory for archived handoffs
    pub archive: PathBuf,

    /// Whether to auto-commit changes
    pub auto_commit: bool,

    /// Whether to auto-push after commit
    pub auto_push: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_dir: PathBuf::from("."),
            pending: PathBuf::from("pending"),
            state: PathBuf::from(".xas"),
            archive: PathBuf::from("archive"),
            auto_commit: true,
            auto_push: false,
        }
    }
}

impl SyncConfig {
    /// Create config with a specific sync directory
    pub fn with_sync_dir(sync_dir: impl Into<PathBuf>) -> Self {
        let sync_dir = sync_dir.into();
        Self {
            pending: sync_dir.join("pending"),
            state: sync_dir.join(".xas"),
            archive: sync_dir.join("archive"),
            sync_dir,
            ..Default::default()
        }
    }
}

/// Sync manager for Git-based synchronization
pub struct SyncManager {
    config: SyncConfig,
    repo: Option<Repository>,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(config: SyncConfig) -> Result<Self> {
        let repo = if config.sync_dir.join(".git").exists() {
            Some(Repository::open(&config.sync_dir)?)
        } else {
            None
        };

        Ok(Self { config, repo })
    }

    /// Initialize the sync directory structure
    pub fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config.pending)?;
        std::fs::create_dir_all(&self.config.state)?;
        std::fs::create_dir_all(&self.config.archive)?;

        // Create .gitignore for state directory (local only)
        let gitignore = self.config.state.join(".gitignore");
        if !gitignore.exists() {
            std::fs::write(&gitignore, "wip.json\ncurrent_agent.json\n")?;
        }

        info!(
            "Initialized XAgentSync directory structure at {:?}",
            self.config.sync_dir
        );
        Ok(())
    }

    /// Write a handoff to the pending directory
    pub fn send_handoff(&self, handoff: &Handoff) -> Result<PathBuf> {
        let filename = format!(
            "{}_{}.json",
            handoff.created_at.format("%Y%m%d_%H%M%S"),
            &handoff.id.to_string()[..8]
        );
        let path = self.config.pending.join(&filename);

        let json = handoff.to_json()?;
        std::fs::write(&path, json)?;

        debug!("Wrote handoff {} to {:?}", handoff.id, path);

        if self.config.auto_commit {
            self.commit_changes(&format!(
                "XAS handoff [{}]: {}",
                handoff.mode.kind(),
                handoff.summary
            ))?;
        }

        Ok(path)
    }

    /// Read handoffs from pending directory
    pub fn receive_handoffs(&self) -> Result<Vec<Handoff>> {
        let mut handoffs = Vec::new();

        if !self.config.pending.exists() {
            return Ok(handoffs);
        }

        for entry in std::fs::read_dir(&self.config.pending)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "json") {
                let content = std::fs::read_to_string(&path)?;
                match Handoff::from_json(&content) {
                    Ok(handoff) => {
                        debug!("Read handoff {} from {:?}", handoff.id, path);
                        handoffs.push(handoff);
                    }
                    Err(e) => {
                        debug!("Failed to parse {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by creation time, newest first
        handoffs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(handoffs)
    }

    /// Archive a processed handoff
    pub fn archive_handoff(&self, handoff_id: &str) -> Result<()> {
        // Find the handoff file in pending
        for entry in std::fs::read_dir(&self.config.pending)? {
            let entry = entry?;
            let path = entry.path();

            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().contains(handoff_id))
            {
                let archive_path = self.config.archive.join(path.file_name().unwrap());
                std::fs::rename(&path, &archive_path)?;
                debug!("Archived handoff to {:?}", archive_path);
                return Ok(());
            }
        }

        Err(crate::Error::HandoffNotFound(handoff_id.to_string()))
    }

    /// Save work-in-progress handoff state
    pub fn save_wip(&self, handoff: &Handoff) -> Result<()> {
        let path = self.config.state.join("wip.json");
        let json = handoff.to_json()?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Load work-in-progress handoff
    pub fn load_wip(&self) -> Result<Option<Handoff>> {
        let path = self.config.state.join("wip.json");
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let handoff = Handoff::from_json(&content)?;
        Ok(Some(handoff))
    }

    /// Clear work-in-progress
    pub fn clear_wip(&self) -> Result<()> {
        let path = self.config.state.join("wip.json");
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Commit pending changes
    pub fn commit_changes(&self, message: &str) -> Result<()> {
        let Some(repo) = &self.repo else {
            debug!("No git repository, skipping commit");
            return Ok(());
        };

        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let sig = repo.signature()?;
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

        let parents: Vec<&git2::Commit> = parent.iter().collect();

        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;

        info!("Committed: {}", message);
        Ok(())
    }

    /// Pull latest changes
    pub fn pull(&self) -> Result<()> {
        let Some(repo) = &self.repo else {
            debug!("No git repository, skipping pull");
            return Ok(());
        };

        let mut remote = repo.find_remote("origin")?;
        let branch = "main";

        remote.fetch(&[branch], None, None)?;

        info!("Pulled latest changes");
        Ok(())
    }

    /// Check if there are pending handoffs
    pub fn has_pending_handoffs(&self) -> Result<bool> {
        if !self.config.pending.exists() {
            return Ok(false);
        }

        for entry in std::fs::read_dir(&self.config.pending)? {
            let entry = entry?;
            if entry.path().extension().is_some_and(|e| e == "json") {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Read state from a file
    pub fn read_state<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let path = self.config.state.join(format!("{}.json", key));
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let state = serde_json::from_str(&content)?;
        Ok(Some(state))
    }

    /// Write state to a file
    pub fn write_state<T: serde::Serialize>(&self, key: &str, state: &T) -> Result<()> {
        let path = self.config.state.join(format!("{}.json", key));
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Get current git commit SHA
    pub fn current_commit(&self) -> Option<String> {
        self.repo.as_ref().and_then(|repo| {
            repo.head()
                .ok()
                .and_then(|h| h.peel_to_commit().ok())
                .map(|c| c.id().to_string())
        })
    }

    /// Get current git branch
    pub fn current_branch(&self) -> Option<String> {
        self.repo.as_ref().and_then(|repo| {
            repo.head().ok().and_then(|h| {
                h.shorthand().map(|s| s.to_string())
            })
        })
    }
}
