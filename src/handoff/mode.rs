//! Handoff modes - deploy, debug, plan

use super::{DeployContext, DebugContext, PlanContext};
use serde::{Deserialize, Serialize};

/// The three modes of handoff, each optimizing for different continuations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "context")]
pub enum HandoffMode {
    /// Deployment mode - focused on shipping
    /// Prioritizes: what to ship, verification, rollback, env concerns
    Deploy(DeployContext),

    /// Debug/Troubleshooting mode - focused on fixing
    /// Prioritizes: problem statement, hypotheses, evidence, what was tried
    Debug(DebugContext),

    /// Planning mode - focused on designing
    /// Prioritizes: requirements, decisions, rejected options, open questions
    Plan(PlanContext),
}

impl HandoffMode {
    /// Get the mode kind as a string
    pub fn kind(&self) -> &'static str {
        match self {
            HandoffMode::Deploy(_) => "deploy",
            HandoffMode::Debug(_) => "debug",
            HandoffMode::Plan(_) => "plan",
        }
    }

    /// Create a deploy mode handoff
    pub fn deploy() -> Self {
        HandoffMode::Deploy(DeployContext::default())
    }

    /// Create a debug mode handoff
    pub fn debug(problem: impl Into<String>) -> Self {
        HandoffMode::Debug(DebugContext::new(problem))
    }

    /// Create a plan mode handoff
    pub fn plan(goal: impl Into<String>) -> Self {
        HandoffMode::Plan(PlanContext::new(goal))
    }

    /// Compile mode-specific section for the prompt
    pub fn compile_section(&self) -> String {
        match self {
            HandoffMode::Deploy(ctx) => ctx.compile(),
            HandoffMode::Debug(ctx) => ctx.compile(),
            HandoffMode::Plan(ctx) => ctx.compile(),
        }
    }

    /// Get deploy context if this is deploy mode
    pub fn as_deploy(&self) -> Option<&DeployContext> {
        match self {
            HandoffMode::Deploy(ctx) => Some(ctx),
            _ => None,
        }
    }

    /// Get deploy context mutably
    pub fn as_deploy_mut(&mut self) -> Option<&mut DeployContext> {
        match self {
            HandoffMode::Deploy(ctx) => Some(ctx),
            _ => None,
        }
    }

    /// Get debug context if this is debug mode
    pub fn as_debug(&self) -> Option<&DebugContext> {
        match self {
            HandoffMode::Debug(ctx) => Some(ctx),
            _ => None,
        }
    }

    /// Get debug context mutably
    pub fn as_debug_mut(&mut self) -> Option<&mut DebugContext> {
        match self {
            HandoffMode::Debug(ctx) => Some(ctx),
            _ => None,
        }
    }

    /// Get plan context if this is plan mode
    pub fn as_plan(&self) -> Option<&PlanContext> {
        match self {
            HandoffMode::Plan(ctx) => Some(ctx),
            _ => None,
        }
    }

    /// Get plan context mutably
    pub fn as_plan_mut(&mut self) -> Option<&mut PlanContext> {
        match self {
            HandoffMode::Plan(ctx) => Some(ctx),
            _ => None,
        }
    }
}

impl std::fmt::Display for HandoffMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandoffMode::Deploy(_) => write!(f, "deploy"),
            HandoffMode::Debug(_) => write!(f, "debug"),
            HandoffMode::Plan(_) => write!(f, "plan"),
        }
    }
}

impl std::str::FromStr for HandoffMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "deploy" | "deployment" | "ship" => Ok(HandoffMode::deploy()),
            "debug" | "troubleshoot" | "fix" => Ok(HandoffMode::debug("(problem not specified)")),
            "plan" | "planning" | "design" => Ok(HandoffMode::plan("(goal not specified)")),
            _ => Err(format!("Unknown mode: {}. Use deploy, debug, or plan.", s)),
        }
    }
}
