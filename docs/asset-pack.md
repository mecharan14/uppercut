# Asset packs (Phase 3)

Declarative, no-code extensions. A pack is a directory containing `pack.json` plus assets.

## Layout

```
my-pack/
  pack.json
  luts/
    film.cube
```

## `pack.json` (v1)

```jsonc
{
  "id": "my-pack",          // unique slug; used in effect ids
  "name": "My Pack",
  "version": "1",
  "caption_styles": [ /* optional */ ],
  "luts": [
    { "id": "film", "label": "Film", "path": "luts/film.cube" }
  ],
  "transitions": [
    // Aliases of builtin TransitionKind only — no custom shaders in v1
    { "id": "quick", "label": "Quick wipe", "kind": "wipe_left", "default_duration_secs": 0.35 }
  ]
}
```

### Caption styles

Fields: `id`, `label`, `font_scale` (relative), `fill_rgba`, optional `stroke_rgba` /
`shadow_rgba` / `box_rgba`, `anchor` (`top`|`center`|`bottom`). Resolved after builtins
when burning captions.

### LUTs

IRIDAS `.cube` (`LUT_3D_SIZE`). Applied on the CPU before GPU upload. Effect id:

`pack:<pack_id>:lut:<lut_id>`

Params: `intensity` (0..1, default 1).

## Commands

- `LoadAssetPack { path }` — validate + record path on the project
- `UnloadAssetPack { pack_id }`

See example: [`examples/packs/starter`](../examples/packs/starter).
Registry seed: [`examples/registry/README.md`](../examples/registry/README.md).
