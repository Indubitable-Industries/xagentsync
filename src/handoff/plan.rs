//! Plan mode context - focused on design and planning

use serde::{Deserialize, Serialize};

/// Context for planning handoffs
///
/// Optimizes for: requirements, decisions made, options rejected, open questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanContext {
    /// The goal we're working toward
    pub goal: String,

    /// Requirements gathered
    pub requirements: Vec<Requirement>,

    /// Decisions that have been made
    pub decisions: Vec<Decision>,

    /// Options that were considered but rejected
    pub rejected_options: Vec<RejectedOption>,

    /// Questions that still need answers
    pub open_questions: Vec<OpenQuestion>,

    /// Suggested next steps
    pub next_steps: Vec<String>,

    /// Constraints and limitations
    pub constraints: Vec<Constraint>,

    /// Key stakeholders or considerations
    pub stakeholders: Vec<String>,

    /// Current phase of planning
    pub phase: PlanPhase,

    /// Rough progress estimate (0-100)
    pub progress_pct: Option<u8>,
}

/// A requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// The requirement
    pub description: String,
    /// Priority
    pub priority: Priority,
    /// Source of this requirement
    pub source: Option<String>,
    /// Is it validated/confirmed?
    pub confirmed: bool,
}

/// Priority level
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Must,
    #[default]
    Should,
    Could,
    Wont,
}

/// A decision that was made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// What was decided
    pub decision: String,
    /// Why this was chosen
    pub rationale: String,
    /// When it was made (or by whom)
    pub context: Option<String>,
    /// Is it reversible?
    pub reversible: bool,
}

/// An option that was rejected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedOption {
    /// What was the option
    pub option: String,
    /// Why it was rejected
    pub reason: String,
    /// Could it be reconsidered?
    pub reconsiderable: bool,
}

/// A question that needs answering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenQuestion {
    /// The question
    pub question: String,
    /// Why it matters
    pub importance: String,
    /// Who might know the answer
    pub ask_who: Option<String>,
    /// Is it blocking progress?
    pub blocking: bool,
}

/// A constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// The constraint
    pub constraint: String,
    /// Why it exists
    pub reason: Option<String>,
    /// Is it negotiable?
    pub negotiable: bool,
}

/// Phase of planning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlanPhase {
    #[default]
    Discovery,
    Requirements,
    Design,
    Review,
    Ready,
}

impl PlanContext {
    /// Create a new plan context with a goal
    pub fn new(goal: impl Into<String>) -> Self {
        Self {
            goal: goal.into(),
            requirements: Vec::new(),
            decisions: Vec::new(),
            rejected_options: Vec::new(),
            open_questions: Vec::new(),
            next_steps: Vec::new(),
            constraints: Vec::new(),
            stakeholders: Vec::new(),
            phase: PlanPhase::Discovery,
            progress_pct: None,
        }
    }

    /// Add a requirement
    pub fn requirement(mut self, desc: impl Into<String>, priority: Priority) -> Self {
        self.requirements.push(Requirement {
            description: desc.into(),
            priority,
            source: None,
            confirmed: false,
        });
        self
    }

    /// Record a decision
    pub fn decided(mut self, decision: impl Into<String>, rationale: impl Into<String>) -> Self {
        self.decisions.push(Decision {
            decision: decision.into(),
            rationale: rationale.into(),
            context: None,
            reversible: true,
        });
        self
    }

    /// Record a rejected option
    pub fn rejected(mut self, option: impl Into<String>, reason: impl Into<String>) -> Self {
        self.rejected_options.push(RejectedOption {
            option: option.into(),
            reason: reason.into(),
            reconsiderable: true,
        });
        self
    }

    /// Add an open question
    pub fn question(mut self, question: impl Into<String>, importance: impl Into<String>) -> Self {
        self.open_questions.push(OpenQuestion {
            question: question.into(),
            importance: importance.into(),
            ask_who: None,
            blocking: false,
        });
        self
    }

    /// Add a blocking question
    pub fn blocking_question(mut self, question: impl Into<String>, importance: impl Into<String>) -> Self {
        self.open_questions.push(OpenQuestion {
            question: question.into(),
            importance: importance.into(),
            ask_who: None,
            blocking: true,
        });
        self
    }

    /// Add a next step
    pub fn next_step(mut self, step: impl Into<String>) -> Self {
        self.next_steps.push(step.into());
        self
    }

    /// Add a constraint
    pub fn constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(Constraint {
            constraint: constraint.into(),
            reason: None,
            negotiable: false,
        });
        self
    }

    /// Set the phase
    pub fn phase(mut self, phase: PlanPhase) -> Self {
        self.phase = phase;
        self
    }

    /// Set progress
    pub fn progress(mut self, pct: u8) -> Self {
        self.progress_pct = Some(pct.min(100));
        self
    }

    /// Compile this context into a prompt section
    pub fn compile(&self) -> String {
        let mut out = String::new();

        out.push_str("## Planning Context\n\n");

        // Goal
        out.push_str("### Goal\n\n");
        out.push_str(&self.goal);
        out.push_str("\n\n");

        // Phase and progress
        out.push_str(&format!("**Phase**: {:?}", self.phase));
        if let Some(pct) = self.progress_pct {
            out.push_str(&format!(" ({}% complete)", pct));
        }
        out.push_str("\n\n");

        // Requirements
        if !self.requirements.is_empty() {
            out.push_str("### Requirements\n\n");
            for req in &self.requirements {
                let confirmed = if req.confirmed { " ✓" } else { "" };
                out.push_str(&format!(
                    "- **{:?}**{}: {}\n",
                    req.priority, confirmed, req.description
                ));
            }
            out.push('\n');
        }

        // Decisions made
        if !self.decisions.is_empty() {
            out.push_str("### Decisions Made\n\n");
            for d in &self.decisions {
                out.push_str(&format!("- **{}**\n", d.decision));
                out.push_str(&format!("  Rationale: {}\n", d.rationale));
            }
            out.push('\n');
        }

        // Rejected options
        if !self.rejected_options.is_empty() {
            out.push_str("### Options Rejected\n\n");
            for r in &self.rejected_options {
                let reconsider = if r.reconsiderable { " (could reconsider)" } else { "" };
                out.push_str(&format!("- ~~{}~~{}: {}\n", r.option, reconsider, r.reason));
            }
            out.push('\n');
        }

        // Open questions
        if !self.open_questions.is_empty() {
            out.push_str("### Open Questions\n\n");
            for q in &self.open_questions {
                let blocking = if q.blocking { " **[BLOCKING]**" } else { "" };
                out.push_str(&format!("- {}{}\n", q.question, blocking));
                out.push_str(&format!("  Why it matters: {}\n", q.importance));
            }
            out.push('\n');
        }

        // Constraints
        if !self.constraints.is_empty() {
            out.push_str("### Constraints\n\n");
            for c in &self.constraints {
                out.push_str(&format!("- {}\n", c.constraint));
            }
            out.push('\n');
        }

        // Next steps
        if !self.next_steps.is_empty() {
            out.push_str("### Suggested Next Steps\n\n");
            for (i, step) in self.next_steps.iter().enumerate() {
                out.push_str(&format!("{}. {}\n", i + 1, step));
            }
            out.push('\n');
        }

        out
    }
}

impl Default for PlanContext {
    fn default() -> Self {
        Self::new("(goal not specified)")
    }
}
