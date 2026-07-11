import { useEffect, useState } from "react";
import { useEditorStore } from "../../store/editorStore";
import { formatTimecode, parseTimecode } from "../../lib/format";
import { timelineDuration } from "../../timeline/layout";

export function TransportBar({
  fullscreen,
  onToggleFullscreen,
}: {
  fullscreen: boolean;
  onToggleFullscreen: () => void;
}) {
  const project = useEditorStore((s) => s.project);
  const playhead = useEditorStore((s) => s.playhead);
  const playing = useEditorStore((s) => s.playing);
  const startPlayback = useEditorStore((s) => s.startPlayback);
  const stopPlayback = useEditorStore((s) => s.stopPlayback);
  const seekTo = useEditorStore((s) => s.seekTo);

  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(formatTimecode(playhead));

  useEffect(() => {
    if (!editing) setDraft(formatTimecode(playhead));
  }, [playhead, editing]);

  const hasClips = !!project?.tracks.some((t) => t.clips.length > 0);
  const duration = project && hasClips ? formatTimecode(timelineDuration(project)) : "—";
  const fps = project?.settings.fps || 60;
  const frameStep = 1 / fps;

  function commitTimecode() {
    const secs = parseTimecode(draft);
    if (secs !== null) void seekTo(secs);
    else setDraft(formatTimecode(playhead));
    setEditing(false);
  }

  return (
    <div className="transport-bar">
      <button
        type="button"
        className="btn-icon-only transport-step"
        title="Previous frame (←)"
        disabled={!project}
        onClick={() => void seekTo(Math.max(0, playhead - frameStep))}
      >
        ‹
      </button>
      <button
        type="button"
        className={`play-btn${playing ? " playing" : ""}`}
        title="Play / Pause (Space)"
        disabled={!project}
        onClick={() => (playing ? stopPlayback() : void startPlayback())}
      >
        {playing ? "⏸" : "▶"}
      </button>
      <button
        type="button"
        className="btn-icon-only transport-step"
        title="Next frame (→)"
        disabled={!project}
        onClick={() => void seekTo(playhead + frameStep)}
      >
        ›
      </button>
      <input
        type="text"
        className="timecode"
        title="Edit timecode (Enter to seek)"
        value={editing ? draft : formatTimecode(playhead)}
        disabled={!project}
        onFocus={() => {
          setEditing(true);
          setDraft(formatTimecode(playhead));
        }}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commitTimecode}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            commitTimecode();
            (e.target as HTMLInputElement).blur();
          }
          if (e.key === "Escape") {
            setDraft(formatTimecode(playhead));
            setEditing(false);
            (e.target as HTMLInputElement).blur();
          }
        }}
      />
      <span className="timecode-sub">/ {duration}</span>
      <span className="spacer" />
      <button
        type="button"
        className={`btn btn-ghost btn-sm${fullscreen ? " active" : ""}`}
        title={fullscreen ? "Exit fullscreen (Esc)" : "Fullscreen preview"}
        disabled={!project}
        onClick={onToggleFullscreen}
      >
        {fullscreen ? "Exit" : "⛶"}
      </button>
    </div>
  );
}
