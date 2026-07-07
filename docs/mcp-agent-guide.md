# MCP agent guide

Status: **placeholder — Phase 1** (PLAN.md §4). `uppercut-mcp` does not exist as a working
server yet; see `uppercut-mcp/src/main.rs`. Until then, drive Uppercut through
`uppercut-cli` (`docs/command-api.md` covers the same commands the MCP tools will wrap).

This document will become the instructions an AI agent needs to make full use of the MCP
server, per PLAN.md §2's "AI needs eyes" principle and §3's tool surface sketch:

- **Tool → command mapping.** Every MCP edit tool wraps exactly one `uppercut-core` command
  from `docs/command-api.md` — no tool may implement editing logic itself.
- **Perception tools.** Render-frame-at-time, transcript, scene/silence detection, waveform
  peaks — read-only, letting an agent see the state of an edit before/after acting on it.
- **A worked example**: taking a script + a folder of gameplay recordings to a finished,
  captioned, voiced-over export using only MCP tool calls (the Phase 1 milestone demo from
  PLAN.md §4).

Fill this in when `uppercut-mcp` is implemented — don't let it drift out of sync with the
actual tool schema, the way `docs/command-api.md` must stay in sync with `Command`.
