# Project schema — v5

Status: **current**. This is the source of truth for `uppercut-core`'s project
model. Implementation types in `uppercut-core/src/project/` must match this document; if
they diverge, fix whichever one is wrong and note it in the same PR. Schema changes bump
`schema_version` and are documented in the "Version history" section at the bottom.

The project file is a single JSON document, human-readable, git-diffable, on disk with
extension `.uppercut.json`.

## Top-level shape

```jsonc
{
  "schema_version": 5,
  "id": "b3f1c2a0-...-uuid",
  "name": "ultra-bruno-ep12",
  "settings": {
    "fps": 60.0,
    "width": 1080,
    "height": 1920,
    "sample_rate": 48000,
    "duck_db": -12.0
  },
  "media": [ /* MediaItem[] */ ],
  "tracks": [ /* Track[] */ ],
  "asset_pack_paths": [],   // optional; directories containing pack.json
  "wasm_plugin_paths": []   // optional; directories containing plugin.json
}
```

| Field | Type | Notes |
|---|---|---|
| `schema_version` | u32 | `5` for this spec. Loaders accept `1`..=`5`; saves write `5`. |
| `id` | string (UUIDv4) | Stable project identity, generated on creation. |
| `name` | string | Human-facing project name; not used for file paths. |
| `settings.*` | — | Same as v3 (`fps`, `width`, `height`, `sample_rate`, `duck_db`). |
| `media` | MediaItem[] | Pool of imported source files. |
| `tracks` | Track[] | Ordered list (video / audio / caption). |
| `asset_pack_paths` | path[] | Loaded asset-pack roots (see [asset-pack.md](asset-pack.md)). |
| `wasm_plugin_paths` | path[] | Loaded WASM plugin roots (see [plugin-api.md](plugin-api.md)). |

## MediaClip (selected fields)

| Field | Notes |
|---|---|
| `source_in_secs` / `source_out_secs` | Source media range. |
| `speed` | Base playback rate (default `1.0`, clamp `0.25`..`4.0`) when no Speed keyframes. |
| `keyframes` | May include `AnimProperty::Speed` keys (same clamp). Timeline duration and source time are the **integral of speed** over the clip (piecewise-linear with easing between keys). |
| `transform` / `effects` | Phase 3. |
| `outgoing_transition` | Optional `ClipTransition` (video tracks). |

Audio with Speed keys is segmented (~50 ms / key intervals) and chained via FFmpeg `atempo`.

### Builtin effects

| `effect_id` | Params |
|---|---|
| `builtin:color_adjust` | `exposure`, `contrast`, `saturation` |
| `builtin:blur` | `radius` |
| `builtin:lut_contrast` / `builtin:lut_warm` | `intensity` |
| `builtin:glitch` | `intensity`, `slice` |

Pack LUTs: `pack:<pack_id>:lut:<lut_id>`. WASM: `wasm:<plugin_id>` when loaded (frame and/or audio ABI).

### ClipTransition

`kind` is one of: `crossfade`, `fade_black`, `wipe_left`, `wipe_right`, `wipe_up`,
`wipe_down`, `slide_left`, `slide_right`, `iris`, `blur_dissolve`, plus `duration_secs`.
Renderer dual-decodes during `[cut − d, cut)` and blends via WGSL (`transition.wgsl`).

## What's intentionally not in v5 yet

Background removal, chroma, masks, tracking, stabilization, denoise, multi-cam, in-app
remote marketplace with payments, custom pack WGSL shaders (Phase 4).

## Version history

- **v0–v3**: see prior history (keyframes/effects in v2; `outgoing_transition` in v3).
- **v4** (Phase 3 close-out): `MediaClip.speed`; ten transition kinds; glitch; project
  `asset_pack_paths` / `wasm_plugin_paths`.
- **v5** (Phase 3 deferred): keyframed `AnimProperty::Speed` ramps (integral source time);
  stickers/SFX pack entries + commands; audio WASM `process_audio`.
