//! Placeholder. The MCP server (stdio, exposing uppercut-core's command API plus perception
//! tools) is Phase 1 work — see PLAN.md §4 and AGENTS.md "Current phase". Until this is
//! wired up, drive projects through `uppercut-cli` instead.
//!
//! When implemented, this crate must not reimplement editing logic: every tool handler
//! dispatches to `uppercut_core::apply_command` or a read-only `uppercut_core` query, same
//! as `uppercut-cli` does (see AGENTS.md §0.1).

fn main() {
    eprintln!("uppercut-mcp: not yet implemented (Phase 1). Use uppercut-cli for now.");
    std::process::exit(1);
}
