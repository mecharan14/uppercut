//! Headless engine for Uppercut. No UI dependencies — see docs/architecture.md.
//! `project` and `commands` are the contract described in docs/project-schema.md and
//! docs/command-api.md; keep them in sync with those documents.

pub mod commands;
pub mod compose;
pub mod export;
pub mod media;
pub mod project;

pub use commands::{apply_command, Command, CommandError, CommandOutcome};
pub use export::{export_project, ExportError, ExportSettings};
pub use project::Project;
