# WASM plugin API v1 (frame effects)

Sandboxed plugins via [wasmtime](https://wasmtime.dev/) (Apache-2.0 / MIT — AGPL-compatible
when linked). Plugins cannot access the filesystem, network, or host APIs beyond the
exported guest ABI below.

## Layout

```
my-plugin/
  plugin.json
  effect.wasm
```

### `plugin.json`

```json
{
  "id": "invert",
  "name": "Invert Colors",
  "version": "1",
  "wasm": "invert.wasm"
}
```

Effect id when loaded: `wasm:<id>`.

## Guest ABI

Exports (required):

| Export | Signature | Notes |
|--------|-----------|--------|
| `memory` | linear memory | Host writes tightly packed RGBA8 into offset 0 |
| `process` | `(ptr: i32, len: i32, width: i32, height: i32) -> ()` | Mutate buffer in place |

No host imports in v1. The module must not require WASI.

## Host flow

1. `LoadWasmPlugin { path }` validates the manifest, compiles the module, stores the path
   on the project.
2. During preview/export, after decode and before GPU upload, enabled `wasm:*` effects on
   a layer run through `PluginHost::apply_effects`.
3. Builtin GPU effects still run after upload.

## Template

[`examples/plugins/invert`](../examples/plugins/invert) — regenerate WASM bytes with:

```rust
let bytes = uppercut_core::plugins::compile_invert_wasm()?;
std::fs::write("examples/plugins/invert/invert.wasm", bytes)?;
```

Or build the Rust `cdylib` sources in that directory for `wasm32-unknown-unknown`.

## Commands

- `LoadWasmPlugin { path }`
- `UnloadWasmPlugin { plugin_id }`
- `SetClipEffects` accepts `wasm:<id>` when the plugin is loaded
