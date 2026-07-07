# Uppercut

**An open-source, native video editor built for AI agents — and the humans who finish the job.**

Uppercut is a from-scratch, cross-platform video editor aimed at closing the gap between
free and workflows currently locked behind paywalled editors. It has two goals that
reinforce each other:

1. Every single edit operation is driven through one command API, so an AI agent (Claude
   Code, or any other model via MCP) can do the entire ideate → clip → order → caption →
   voiceover pass on its own, with a human doing final polish at the end.
2. The feature set — effects, transitions, filters, stickers, captions styles — is built
   by its community as plugins and asset packs, not gated behind a subscription.

Not affiliated with, endorsed by, or a clone of any existing commercial editor.

## Status

**Early / pre-alpha.** The engine's project schema and command API are tested; the CLI can
build a timeline end to end and **export a cuts-only edit to MP4** (FFmpeg decode → wgpu
composite → H.264 encode). Auto-captions, audio mixing, MCP, and the GUI are not built yet
— see the roadmap below.

Read [PLAN.md](PLAN.md) for the full vision, tech stack rationale, and phased roadmap
before contributing or building on this.

## Why

Editing gameplay footage to a script in a paywalled editor works, but too many of the
features that make it fast — the ones that actually matter — sit behind a subscription.
Uppercut is an attempt to build the same caliber of editor as free, open, and automatable
software, so the tool decides nothing about what you're allowed to do.

## Architecture at a glance

```
uppercut-core/     headless Rust engine — project model, command API, media I/O, compositing.
                    No UI dependencies. The single source of truth every frontend calls into.
uppercut-cli/       thin CLI over the command API — scriptable, the simplest agent-drivable surface.
uppercut-mcp/       MCP server exposing the same commands + perception tools to AI agents. (Phase 1)
uppercut-app/       Tauri 2 desktop app; native GPU preview surface, webview for UI chrome only. (Phase 2)
```

One command API, one project schema, dispatched identically by the GUI, the CLI, and the
MCP server — see [docs/architecture.md](docs/architecture.md) for the full picture and
[docs/command-api.md](docs/command-api.md) / [docs/project-schema.md](docs/project-schema.md)
for the exact specs the code implements.

**Tech stack:** Rust engine (FFmpeg, wgpu, whisper.cpp for local captions, local/BYO-key TTS)
+ Tauri 2 UI, with the video preview rendered on a native GPU surface rather than through
the webview. Plugins are sandboxed WebAssembly; simple content ships as no-code asset packs.
Windows first, macOS/Linux to follow. AGPL-3.0.

## Roadmap

- **Phase 0 (current)** — workspace skeleton, project schema v0, command API, CLI, and the
  media spine (FFmpeg subprocess decode → wgpu composite → encode). Milestone reached for
  cuts-only video export; audio muxing and linked libav I/O come next.
- **Phase 1** — headless AI editing: auto-captions, AI voiceover, audio mixing, MCP server.
  Milestone: hand Claude Code a script and a folder of gameplay recordings and get a
  finished video back with no GUI involved.
- **Phase 2** — GUI MVP: timeline, native preview, direct manipulation, export dialog.
- **Phase 3** — effects, transitions, keyframes, the WASM plugin SDK and asset pack format.
- **Phase 4** — community-driven feature parity march (background removal, motion tracking,
  templates, and more), mostly landing as plugins rather than core code.

Full detail in [PLAN.md](PLAN.md) §4.

## Getting started

```sh
git clone <repo-url>
cd video-editor
cargo build --workspace
cargo test --workspace
```

Try the CLI:

```sh
cargo run -p uppercut-cli -- new-project demo.uppercut.json --name "my-edit"
cargo run -p uppercut-cli -- apply demo.uppercut.json '{"command":"AddTrack","kind":"audio","name":"A1"}'
cargo run -p uppercut-cli -- show demo.uppercut.json
cargo run -p uppercut-cli -- export demo.uppercut.json out.mp4 --preset tiktok
```

Export requires `ffmpeg` and `ffprobe` on PATH (used as subprocesses in Phase 0; linked
`ffmpeg-the-third` is planned once vcpkg/FFMPEG_DIR is wired for all environments).

## Contributing

Code or no-code, there's a way in — see [CONTRIBUTING.md](CONTRIBUTING.md). If you're an AI
agent working in this repo, read [AGENTS.md](AGENTS.md) first; it's the enforceable
contract behind everything summarized above.

## License

[AGPL-3.0](LICENSE) — forks and hosted services must stay open source.
