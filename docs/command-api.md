# Command API — v0

Status: **spec for Phase 0**. This is the source of truth for the `Command` enum and
`apply_command` in `uppercut-core`. GUI, CLI, and MCP must all dispatch through this exact
set (see AGENTS.md §0.1) — none of them may mutate `Project` state any other way.

## Shape

```rust
pub enum Command {
    ImportMedia { path: String },
    AddTrack { kind: TrackKind, name: String },
    AddClip { track_id: Id, media_id: Id, position_secs: f64, source_in_secs: f64, source_out_secs: f64 },
    SplitClip { track_id: Id, clip_id: Id, at_secs: f64 },
    TrimClip { track_id: Id, clip_id: Id, new_source_in_secs: Option<f64>, new_source_out_secs: Option<f64> },
    MoveClip { track_id: Id, clip_id: Id, new_position_secs: f64, new_track_id: Option<Id> },
    DeleteClip { track_id: Id, clip_id: Id, ripple: bool },
    AddCaption { track_id: Id, text: String, position_secs: f64, duration_secs: f64, style_id: String },
    SetAudioGain { track_id: Id, clip_id: Id, gain_db: f64 },
    Export { output_path: String, preset: ExportPreset },
}

pub fn apply_command(project: &mut Project, cmd: Command) -> Result<CommandOutcome, CommandError>;
```

`CommandOutcome` carries whatever the caller needs back (e.g. `ImportMedia` returns the new
`media_id`; edit commands return `Ok(CommandOutcome::Applied)`; `Export` returns a handle to
track render progress — exact shape TBD when the render pipeline exists, don't over-specify
ahead of Phase 0's spike).

Every command except `Export` is a pure mutation of `Project` and must be representable in
the project's command log for undo/redo (Phase 2+ feature; v0 just needs the enum to be
`Serialize`/`Deserialize`-clean so logging is possible later — don't build the log itself
yet).

## Commands

### `ImportMedia`
Probes the file at `path` (duration, dimensions, fps as applicable) and adds a `MediaItem`
to `project.media`. Returns the new `media_id`. Does not place anything on a track.

- Errors: file not found, unsupported/unprobeable format.

### `AddTrack`
Appends a new empty `Track` of the given `kind` (`video` | `audio` | `caption`) to
`project.tracks`. Returns the new `track_id`.

### `AddClip`
Adds a `VideoClip`/`AudioClip` (matching the target track's kind) referencing `media_id` to
`track_id`, placed at `position_secs`, sourced from `[source_in_secs, source_out_secs)` of
the media.

- Errors: `track_id`/`media_id` not found, track kind doesn't match media kind, resulting
  clip overlaps an existing clip on that track, `source_out_secs <= source_in_secs`, or the
  source range exceeds the media's probed duration (skipped if duration is unknown — see
  project-schema.md).

### `SplitClip`
Splits the clip at timeline position `at_secs` into two clips with adjusted
`source_in_secs`/`source_out_secs`, both keeping the same `media_id`. New clip on the right
gets a fresh id; the left retains the original id.

- Errors: `at_secs` not strictly inside the clip's timeline span.

### `TrimClip`
Adjusts `source_in_secs` and/or `source_out_secs` in place (ripple is not implied — this
only changes which part of the source plays, not other clips' positions). At least one of
the two optional fields must be `Some`.

- Errors: resulting `source_out_secs <= source_in_secs`, or new range exceeds media bounds.

### `MoveClip`
Changes a clip's `position_secs`, optionally moving it to a different track
(`new_track_id`). Used for reordering clips to match the script.

- Errors: destination track kind mismatch, resulting overlap with another clip on the
  destination track.

### `DeleteClip`
Removes a clip. If `ripple: true`, all subsequent clips on the same track shift left to
close the gap; if `false`, a gap is left in place.

- Errors: clip not found on track.

### `AddCaption`
Adds a `CaptionClip` with `text` at `position_secs` for `duration_secs`, tagged with
`style_id` (a built-in style name in v0; asset-pack-provided styles come in Phase 3). Target
track must be `kind: "caption"`.

- Errors: track kind mismatch, overlap with existing caption clip on that track.

### `SetAudioGain`
Sets `gain_db` on an audio-bearing clip (`AudioClip`, or a `VideoClip` with embedded audio
once that's modeled — v0 audio gain applies to `AudioClip` only).

- Errors: clip not found, or clip type has no audio.

### `Export`
Renders the current `Project` timeline to `output_path` using `preset` (e.g.
`TikTok9x16`, `Youtube16x9`, or a `Custom { width, height, fps }` variant). This is the one
command with a side effect beyond mutating `Project` in memory — it drives the media/compose
pipeline. Exact `ExportPreset` variants and progress-reporting shape are finalized during
the Phase 0 media-spine spike; treat what's above as intent, not final signature.

## Non-goals for v0

No effect/transition/keyframe commands, no plugin invocation commands, no multi-cam
commands — these arrive with their respective schema additions in later phases
(see [project-schema.md](project-schema.md) "What's intentionally not in v0"). Do not add
a command for a feature that has no schema representation yet.

## Version history

- **v0** (Phase 0): the 10 commands above, matching project schema v0.
