//! Debug mode context - focused on troubleshooting

use serde::{Deserialize, Serialize};

/// Context for debug/troubleshooting handoffs
///
/// Optimizes for: what's broken, hypotheses, evidence, what was tried
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugContext {
    /// Clear statement of the problem
    pub problem_statement: String,

    /// How the problem manifests
    pub symptoms: Vec<String>,

    /// Current hypotheses about the cause
    pub hypotheses: Vec<Hypothesis>,

    /// What has been tried already
    pub attempted: Vec<Attempt>,

    /// Evidence gathered (logs, errors, observations)
    pub evidence: Vec<Evidence>,

    /// Files suspected to be involved
    pub suspected_files: Vec<SuspectedFile>,

    /// Steps to reproduce the issue
    pub reproduction_steps: Option<String>,

    /// Current best theory
    pub working_theory: Option<String>,

    /// What the previous agent was about to try
    pub next_to_try: Option<String>,
}

/// A hypothesis about what might be wrong
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    /// The hypothesis
    pub theory: String,
    /// Supporting evidence
    pub support: Vec<String>,
    /// Contradicting evidence
    pub against: Vec<String>,
    /// Likelihood assessment
    pub likelihood: Likelihood,
}

/// Likelihood of a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Likelihood {
    High,
    #[default]
    Medium,
    Low,
    Eliminated,
}

/// Something that was attempted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    /// What was tried
    pub what: String,
    /// What happened
    pub result: String,
    /// Did it help/hurt/nothing?
    pub outcome: AttemptOutcome,
}

/// Outcome of an attempt
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttemptOutcome {
    Fixed,
    Helped,
    #[default]
    NoEffect,
    MadeWorse,
    Inconclusive,
}

/// A piece of evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Type of evidence
    pub kind: EvidenceKind,
    /// The evidence content
    pub content: String,
    /// Where it came from
    pub source: Option<String>,
    /// When it was observed
    pub timestamp: Option<String>,
}

/// Kind of evidence
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    #[default]
    Observation,
    LogEntry,
    ErrorMessage,
    StackTrace,
    Metric,
    UserReport,
    Screenshot,
}

/// A file suspected to be involved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspectedFile {
    /// Path to the file
    pub path: String,
    /// Why it's suspected
    pub reason: String,
    /// Specific lines if known
    pub lines: Option<String>,
    /// Confidence
    pub confidence: Likelihood,
}

impl DebugContext {
    /// Create a new debug context with a problem statement
    pub fn new(problem: impl Into<String>) -> Self {
        Self {
            problem_statement: problem.into(),
            symptoms: Vec::new(),
            hypotheses: Vec::new(),
            attempted: Vec::new(),
            evidence: Vec::new(),
            suspected_files: Vec::new(),
            reproduction_steps: None,
            working_theory: None,
            next_to_try: None,
        }
    }

    /// Add a symptom
    pub fn symptom(mut self, symptom: impl Into<String>) -> Self {
        self.symptoms.push(symptom.into());
        self
    }

    /// Add a hypothesis
    pub fn hypothesis(mut self, theory: impl Into<String>, likelihood: Likelihood) -> Self {
        self.hypotheses.push(Hypothesis {
            theory: theory.into(),
            support: Vec::new(),
            against: Vec::new(),
            likelihood,
        });
        self
    }

    /// Record an attempt
    pub fn tried(mut self, what: impl Into<String>, result: impl Into<String>, outcome: AttemptOutcome) -> Self {
        self.attempted.push(Attempt {
            what: what.into(),
            result: result.into(),
            outcome,
        });
        self
    }

    /// Add evidence
    pub fn evidence(mut self, kind: EvidenceKind, content: impl Into<String>) -> Self {
        self.evidence.push(Evidence {
            kind,
            content: content.into(),
            source: None,
            timestamp: None,
        });
        self
    }

    /// Add a suspected file
    pub fn suspect_file(mut self, path: impl Into<String>, reason: impl Into<String>) -> Self {
        self.suspected_files.push(SuspectedFile {
            path: path.into(),
            reason: reason.into(),
            lines: None,
            confidence: Likelihood::Medium,
        });
        self
    }

    /// Set reproduction steps
    pub fn repro(mut self, steps: impl Into<String>) -> Self {
        self.reproduction_steps = Some(steps.into());
        self
    }

    /// Set working theory
    pub fn theory(mut self, theory: impl Into<String>) -> Self {
        self.working_theory = Some(theory.into());
        self
    }

    /// Set what to try next
    pub fn try_next(mut self, next: impl Into<String>) -> Self {
        self.next_to_try = Some(next.into());
        self
    }

    /// Compile this context into a prompt section
    pub fn compile(&self) -> String {
        let mut out = String::new();

        out.push_str("## Troubleshooting Context\n\n");

        // Problem statement
        out.push_str("### Problem\n\n");
        out.push_str(&self.problem_statement);
        out.push_str("\n\n");

        // Symptoms
        if !self.symptoms.is_empty() {
            out.push_str("### Symptoms\n\n");
            for symptom in &self.symptoms {
                out.push_str(&format!("- {}\n", symptom));
            }
            out.push('\n');
        }

        // Reproduction
        if let Some(ref repro) = self.reproduction_steps {
            out.push_str("### How to Reproduce\n\n");
            out.push_str(repro);
            out.push_str("\n\n");
        }

        // Working theory
        if let Some(ref theory) = self.working_theory {
            out.push_str("### Current Working Theory\n\n");
            out.push_str(theory);
            out.push_str("\n\n");
        }

        // Hypotheses
        if !self.hypotheses.is_empty() {
            out.push_str("### Hypotheses\n\n");
            for h in &self.hypotheses {
                out.push_str(&format!("- **{:?}**: {}\n", h.likelihood, h.theory));
                for s in &h.support {
                    out.push_str(&format!("  - Supports: {}\n", s));
                }
                for a in &h.against {
                    out.push_str(&format!("  - Against: {}\n", a));
                }
            }
            out.push('\n');
        }

        // What was tried
        if !self.attempted.is_empty() {
            out.push_str("### Already Tried\n\n");
            for attempt in &self.attempted {
                out.push_str(&format!(
                    "- **{}** → {} ({:?})\n",
                    attempt.what, attempt.result, attempt.outcome
                ));
            }
            out.push('\n');
        }

        // Evidence
        if !self.evidence.is_empty() {
            out.push_str("### Evidence\n\n");
            for e in &self.evidence {
                out.push_str(&format!("**{:?}**", e.kind));
                if let Some(ref src) = e.source {
                    out.push_str(&format!(" (from {})", src));
                }
                out.push_str(":\n```\n");
                out.push_str(&e.content);
                out.push_str("\n```\n\n");
            }
        }

        // Suspected files
        if !self.suspected_files.is_empty() {
            out.push_str("### Suspected Files\n\n");
            for sf in &self.suspected_files {
                out.push_str(&format!("- `{}` ({:?}): {}\n", sf.path, sf.confidence, sf.reason));
                if let Some(ref lines) = sf.lines {
                    out.push_str(&format!("  Lines: {}\n", lines));
                }
            }
            out.push('\n');
        }

        // What to try next
        if let Some(ref next) = self.next_to_try {
            out.push_str("### Suggested Next Step\n\n");
            out.push_str(next);
            out.push_str("\n\n");
        }

        out
    }
}

impl Default for DebugContext {
    fn default() -> Self {
        Self::new("(problem not specified)")
    }
}
