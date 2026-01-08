//! XAgentSync - Async handoff protocol for LLM code assistants
//!
//! This library provides types and utilities for creating structured handoffs
//! between LLM agents working asynchronously on shared codebases.
//!
//! ## Core Concepts
//!
//! - **Handoff**: The unit of transfer between agents, containing context and warm-up info
//! - **Mode**: One of `deploy`, `debug`, or `plan` - determines what context is prioritized
//! - **Session State**: What the creating agent did, for receiving agent's awareness
//! - **Warm-up Sequence**: How to efficiently bootstrap the receiving agent
//!
//! ## Three Modes
//!
//! - **Deploy**: Focused on shipping - what to ship, verification, rollback
//! - **Debug**: Focused on fixing - problem, hypotheses, evidence, what was tried
//! - **Plan**: Focused on designing - requirements, decisions, rejected options, questions

pub mod cli;
pub mod context;
pub mod handoff;
pub mod sync;

pub use context::SessionState;
pub use handoff::{
    DeployContext, DebugContext, GitRef, Handoff, HandoffMode, PlanContext, PriorityFile,
    WarmUpSequence,
};

/// Result type for xagentsync operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in xagentsync operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Handoff not found: {0}")]
    HandoffNotFound(String),

    #[error("No active handoff in progress. Start one with 'deploy new', 'debug new', or 'plan new'")]
    NoActiveHandoff,

    #[error("Agent not registered: {0}")]
    AgentNotRegistered(String),

    #[error("Invalid mode: {0}")]
    InvalidMode(String),

    #[error("Validation error: {0}")]
    Validation(String),
}
