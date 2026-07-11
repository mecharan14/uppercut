# Uppercut extension registry (seed)

Curated index of example asset packs and WASM plugins that ship with this repository.
Community contributions: open a PR adding a row that points at a pack/plugin directory
with a valid `pack.json` / `plugin.json`. CI should validate manifests once packs land
in a dedicated `uppercut-registry` repo — until then, this file is the seed index.

## Asset packs

| Id | Path | Notes |
|----|------|-------|
| `starter` | [`../packs/starter`](../packs/starter) | Neon caption style + punch `.cube` LUT |

Load in a project via command `LoadAssetPack` with the pack directory path. LUT effect ids
look like `pack:starter:lut:punch`.

## WASM plugins

| Id | Path | Notes |
|----|------|-------|
| `invert` | [`../plugins/invert`](../plugins/invert) | Invert RGB; template for “30-line” effects |

Generate `invert.wasm` with `compile_invert_wasm()` (see `docs/plugin-api.md`) or `rustc`
as noted in `invert.rs`. Load via `LoadWasmPlugin`; effect id `wasm:invert`.

## Specs

- [Asset packs](../../docs/asset-pack.md)
- [Plugin API](../../docs/plugin-api.md)
