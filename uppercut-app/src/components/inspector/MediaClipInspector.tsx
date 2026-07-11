import { useEffect, useState } from "react";
import { useEditorStore } from "../../store/editorStore";
import {
  deleteClip,
  setAudioFade,
  setAudioGain,
  setClipEnabled,
  setTrackAudioRole,
  trimClip,
} from "../../lib/commands";
import type { MediaClip, Track, TrackAudioRole } from "../../lib/types";

const AUDIO_ROLES: { value: TrackAudioRole | ""; label: string }[] = [
  { value: "", label: "None" },
  { value: "voiceover", label: "Voiceover" },
  { value: "dialog", label: "Dialog" },
  { value: "music", label: "Music" },
  { value: "ambience", label: "Ambience" },
];

export function MediaClipInspector({ track, clip }: { track: Track; clip: MediaClip }) {
  const dispatch = useEditorStore((s) => s.dispatch);
  const select = useEditorStore((s) => s.select);
  const media = useEditorStore((s) => s.project?.media.find((m) => m.id === clip.media_id));

  const [gain, setGain] = useState(clip.gain_db);
  const [sourceIn, setSourceIn] = useState(clip.source_in_secs);
  const [sourceOut, setSourceOut] = useState(clip.source_out_secs);
  const [fadeIn, setFadeIn] = useState(clip.fade_in_secs);
  const [fadeOut, setFadeOut] = useState(clip.fade_out_secs);

  useEffect(() => {
    setGain(clip.gain_db);
    setSourceIn(clip.source_in_secs);
    setSourceOut(clip.source_out_secs);
    setFadeIn(clip.fade_in_secs);
    setFadeOut(clip.fade_out_secs);
  }, [clip.id, clip.gain_db, clip.source_in_secs, clip.source_out_secs, clip.fade_in_secs, clip.fade_out_secs]);

  async function commitTrim() {
    await dispatch(trimClip(track.id, clip.id, sourceIn, sourceOut));
  }

  const showAudio = clip.type === "audio" || track.kind === "audio" || clip.type === "video";

  return (
    <div className="inspector">
      <div className="inspector-section">
        <h3>{clip.type === "audio" ? "Audio clip" : "Video clip"}</h3>
        <p>
          {track.name} · {track.kind}
          {media ? ` · ${media.path.split(/[/\\]/).pop()}` : ""}
        </p>
        {media?.width && media?.height ? (
          <p className="empty-hint">
            Source {media.width}×{media.height}
            {media.fps ? ` · ${media.fps.toFixed(2)} fps` : ""}
            {media.duration_secs != null ? ` · ${media.duration_secs.toFixed(1)}s` : ""}
          </p>
        ) : null}
        <div className="field toggle-row">
          <input
            id="clip-enabled"
            type="checkbox"
            checked={clip.enabled}
            disabled={track.locked}
            onChange={(e) => void dispatch(setClipEnabled(track.id, clip.id, e.target.checked))}
          />
          <label htmlFor="clip-enabled">Enabled</label>
        </div>
      </div>

      <div className="inspector-section">
        <h3>Trim (source in/out)</h3>
        <div className="field">
          <label>In (seconds)</label>
          <input
            type="number"
            step="0.1"
            min={0}
            value={sourceIn}
            disabled={track.locked}
            onChange={(e) => setSourceIn(parseFloat(e.target.value))}
            onBlur={() => void commitTrim()}
          />
        </div>
        <div className="field">
          <label>Out (seconds)</label>
          <input
            type="number"
            step="0.1"
            min={0}
            value={sourceOut}
            disabled={track.locked}
            onChange={(e) => setSourceOut(parseFloat(e.target.value))}
            onBlur={() => void commitTrim()}
          />
        </div>
      </div>

      {showAudio && (
        <div className="inspector-section">
          <h3>Audio</h3>
          <div className="field">
            <label>Volume (dB)</label>
            <input
              type="range"
              min={-24}
              max={12}
              step={0.5}
              value={gain}
              disabled={track.locked}
              onChange={(e) => setGain(parseFloat(e.target.value))}
              onMouseUp={() => void dispatch(setAudioGain(track.id, clip.id, gain))}
              onTouchEnd={() => void dispatch(setAudioGain(track.id, clip.id, gain))}
            />
            <span>{gain} dB</span>
          </div>
          <div className="field">
            <label>Fade in (seconds)</label>
            <input
              type="number"
              step="0.1"
              min={0}
              value={fadeIn}
              disabled={track.locked}
              onChange={(e) => setFadeIn(parseFloat(e.target.value) || 0)}
              onBlur={() => void dispatch(setAudioFade(track.id, clip.id, fadeIn, fadeOut))}
            />
          </div>
          <div className="field">
            <label>Fade out (seconds)</label>
            <input
              type="number"
              step="0.1"
              min={0}
              value={fadeOut}
              disabled={track.locked}
              onChange={(e) => setFadeOut(parseFloat(e.target.value) || 0)}
              onBlur={() => void dispatch(setAudioFade(track.id, clip.id, fadeIn, fadeOut))}
            />
          </div>
          {track.kind === "audio" && (
            <div className="field">
              <label>Track role</label>
              <select
                value={track.audio_role ?? ""}
                disabled={track.locked}
                onChange={(e) => {
                  const v = e.target.value as TrackAudioRole | "";
                  void dispatch(setTrackAudioRole(track.id, v === "" ? null : v));
                }}
              >
                {AUDIO_ROLES.map((r) => (
                  <option key={r.label} value={r.value}>
                    {r.label}
                  </option>
                ))}
              </select>
            </div>
          )}
        </div>
      )}

      <div className="inspector-actions">
        <button
          type="button"
          className="btn-danger"
          disabled={track.locked}
          onClick={async () => {
            await dispatch(deleteClip(track.id, clip.id, false));
            select(null);
          }}
        >
          <span className="btn-icon">🗑</span>
          <span>Delete</span>
        </button>
      </div>
    </div>
  );
}
