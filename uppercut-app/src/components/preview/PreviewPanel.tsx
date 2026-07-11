import { useEffect, useRef, useState } from "react";
import { useEditorStore } from "../../store/editorStore";
import * as ipc from "../../lib/ipc";
import { TransportBar } from "./TransportBar";

/// Fits the project's aspect ratio inside the host, letterboxed, and sends that sub-rect
/// (not the full host rect) to the backend — the native wgpu child window is sized to
/// exactly the letterboxed content area, so the host's dark background shows through as
/// pillar/letterbox bars. See docs/architecture.md "Playback engine".
async function syncPreviewBounds(host: HTMLElement, aspect: number) {
  await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
  const rect = host.getBoundingClientRect();
  if (rect.width < 1 || rect.height < 1) return;

  const hostAspect = rect.width / rect.height;
  let width: number;
  let height: number;
  if (hostAspect > aspect) {
    height = rect.height;
    width = height * aspect;
  } else {
    width = rect.width;
    height = width / aspect;
  }
  const x = rect.left + (rect.width - width) / 2;
  const y = rect.top + (rect.height - height) / 2;

  try {
    await ipc.setPreviewBounds(Math.round(x), Math.round(y), Math.round(width), Math.round(height));
  } catch (e) {
    console.warn("preview bounds:", e);
  }
}

export function PreviewPanel() {
  const project = useEditorStore((s) => s.project);
  const importBusy = useEditorStore((s) => s.importBusy);
  const playing = useEditorStore((s) => s.playing);
  const hostRef = useRef<HTMLDivElement | null>(null);
  const [fullscreen, setFullscreen] = useState(false);

  const hasClips = !!project?.tracks.some((t) => t.clips.length > 0);
  const aspect = project ? project.settings.width / project.settings.height : 9 / 16;

  useEffect(() => {
    const host = hostRef.current;
    if (!host || !project) return;

    const syncAndRender = async () => {
      await syncPreviewBounds(host, aspect);
      if (!useEditorStore.getState().playing) {
        await ipc.seek(useEditorStore.getState().playhead).catch(() => {});
      }
    };

    void syncAndRender();
    const observer = new ResizeObserver(() => void syncAndRender());
    observer.observe(host);
    return () => observer.disconnect();
  }, [project, aspect, fullscreen]);

  useEffect(() => {
    if (!fullscreen) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        setFullscreen(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [fullscreen]);

  let hintContent: React.ReactNode = null;
  if (importBusy) {
    hintContent = (
      <>
        <span className="icon spinner" />
        <span>Importing your media…</span>
      </>
    );
  } else if (!project) {
    hintContent = (
      <>
        <span className="icon">🎬</span>
        <span>Create or import to start editing</span>
      </>
    );
  } else if (!hasClips) {
    hintContent = (
      <>
        <span className="icon">⬆</span>
        <span>
          <strong>Import a video</strong>
          <br />
          Click the media panel or Import button
        </span>
      </>
    );
  } else if (!playing) {
    hintContent = (
      <>
        <span className="icon">▶</span>
        <span>Press play to preview</span>
      </>
    );
  }

  return (
    <div className={`preview-column${fullscreen ? " preview-fullscreen" : ""}`}>
      <section
        id="preview-host"
        ref={hostRef}
        className={`preview-host${hasClips ? " has-clips" : ""}${importBusy ? " loading" : ""}${playing ? " is-playing" : ""}`}
      >
        {hintContent && <div className="hint">{hintContent}</div>}
      </section>
      <TransportBar
        fullscreen={fullscreen}
        onToggleFullscreen={() => setFullscreen((v) => !v)}
      />
    </div>
  );
}
