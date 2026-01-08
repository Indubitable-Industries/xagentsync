//! Deploy mode context - focused on shipping code

use serde::{Deserialize, Serialize};

/// Context for deployment handoffs
///
/// Optimizes for: what to ship, how to verify, how to rollback
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeployContext {
    /// What's ready to ship (files, features, changes)
    pub what_to_ship: Vec<ShipItem>,

    /// Steps to verify the deployment works
    pub verification_steps: Vec<String>,

    /// How to rollback if things go wrong
    pub rollback_plan: Option<String>,

    /// Environment-specific concerns
    pub env_concerns: Vec<EnvConcern>,

    /// Dependencies that must be in place
    pub dependencies: Vec<Dependency>,

    /// Breaking changes to be aware of
    pub breaking_changes: Vec<BreakingChange>,

    /// Pre-deployment checklist items
    pub checklist: Vec<ChecklistItem>,

    /// Post-deployment monitoring notes
    pub monitoring_notes: Option<String>,
}

/// Something ready to ship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipItem {
    /// What it is (file, feature, fix)
    pub item: String,
    /// Brief description
    pub description: String,
    /// Confidence level (high, medium, low)
    pub confidence: Confidence,
}

/// Confidence level
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    #[default]
    Medium,
    Low,
}

/// Environment-specific concern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConcern {
    /// Which environment (prod, staging, dev)
    pub environment: String,
    /// The concern
    pub concern: String,
    /// Mitigation if any
    pub mitigation: Option<String>,
}

/// A dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// What the dependency is
    pub name: String,
    /// Why it's needed
    pub reason: String,
    /// Is it already in place?
    pub in_place: bool,
}

/// A breaking change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    /// What breaks
    pub what: String,
    /// Who/what is affected
    pub affects: String,
    /// Migration path
    pub migration: Option<String>,
}

/// Checklist item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    /// The item
    pub item: String,
    /// Is it done?
    pub done: bool,
}

impl DeployContext {
    /// Add something to ship
    pub fn ship(mut self, item: impl Into<String>, description: impl Into<String>) -> Self {
        self.what_to_ship.push(ShipItem {
            item: item.into(),
            description: description.into(),
            confidence: Confidence::Medium,
        });
        self
    }

    /// Add a verification step
    pub fn verify(mut self, step: impl Into<String>) -> Self {
        self.verification_steps.push(step.into());
        self
    }

    /// Set the rollback plan
    pub fn rollback(mut self, plan: impl Into<String>) -> Self {
        self.rollback_plan = Some(plan.into());
        self
    }

    /// Add an environment concern
    pub fn env_concern(mut self, env: impl Into<String>, concern: impl Into<String>) -> Self {
        self.env_concerns.push(EnvConcern {
            environment: env.into(),
            concern: concern.into(),
            mitigation: None,
        });
        self
    }

    /// Add a breaking change
    pub fn breaking(mut self, what: impl Into<String>, affects: impl Into<String>) -> Self {
        self.breaking_changes.push(BreakingChange {
            what: what.into(),
            affects: affects.into(),
            migration: None,
        });
        self
    }

    /// Add a checklist item
    pub fn checklist(mut self, item: impl Into<String>, done: bool) -> Self {
        self.checklist.push(ChecklistItem {
            item: item.into(),
            done,
        });
        self
    }

    /// Compile this context into a prompt section
    pub fn compile(&self) -> String {
        let mut out = String::new();

        out.push_str("## Deployment Context\n\n");

        // What to ship
        if !self.what_to_ship.is_empty() {
            out.push_str("### Ready to Ship\n\n");
            for item in &self.what_to_ship {
                out.push_str(&format!(
                    "- **{}** ({:?}): {}\n",
                    item.item, item.confidence, item.description
                ));
            }
            out.push('\n');
        }

        // Verification
        if !self.verification_steps.is_empty() {
            out.push_str("### Verification Steps\n\n");
            for (i, step) in self.verification_steps.iter().enumerate() {
                out.push_str(&format!("{}. {}\n", i + 1, step));
            }
            out.push('\n');
        }

        // Rollback
        if let Some(ref rollback) = self.rollback_plan {
            out.push_str("### Rollback Plan\n\n");
            out.push_str(rollback);
            out.push_str("\n\n");
        }

        // Breaking changes
        if !self.breaking_changes.is_empty() {
            out.push_str("### Breaking Changes\n\n");
            for bc in &self.breaking_changes {
                out.push_str(&format!("- **{}** affects {}\n", bc.what, bc.affects));
                if let Some(ref migration) = bc.migration {
                    out.push_str(&format!("  Migration: {}\n", migration));
                }
            }
            out.push('\n');
        }

        // Env concerns
        if !self.env_concerns.is_empty() {
            out.push_str("### Environment Concerns\n\n");
            for ec in &self.env_concerns {
                out.push_str(&format!("- **{}**: {}\n", ec.environment, ec.concern));
            }
            out.push('\n');
        }

        // Checklist
        if !self.checklist.is_empty() {
            out.push_str("### Checklist\n\n");
            for item in &self.checklist {
                let mark = if item.done { "x" } else { " " };
                out.push_str(&format!("- [{}] {}\n", mark, item.item));
            }
            out.push('\n');
        }

        out
    }
}
