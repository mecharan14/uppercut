//! Headless engine for Uppercut. No UI dependencies — see docs/architecture.md.
//! `project` and `commands` are the contract described in docs/project-schema.md and
//! docs/command-api.md; keep them in sync with those documents.

pub mod commands;
pub mod media;
pub mod project;

pub use commands::{apply_command, Command, CommandError, CommandOutcome};
pub use project::Project;
