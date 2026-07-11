import { ASPECT_PRESETS, type Project } from "../../lib/types";
import { setProjectSettings } from "../../lib/commands";
import { useEditorStore } from "../../store/editorStore";

function originalDims(project: Project): { width: number; height: number } | null {
  const firstVideo = project.media.find(
    (m) => m.kind === "video" && m.width && m.height && m.width > 0 && m.height > 0,
  );
  if (!firstVideo?.width || !firstVideo?.height) return null;
  return { width: firstVideo.width, height: firstVideo.height };
}

function currentPresetId(project: Project): string {
  const { width, height } = project.settings;
  const match = ASPECT_PRESETS.find((p) => p.width === width && p.height === height);
  if (match) return match.id;
  const orig = originalDims(project);
  if (orig && orig.width === width && orig.height === height) return "original";
  return "custom";
}

export function RatioMenu() {
  const project = useEditorStore((s) => s.project);
  const dispatch = useEditorStore((s) => s.dispatch);
  const toast = useEditorStore((s) => s.toast);

  if (!project) {
    return (
      <select className="ratio-select" disabled title="Canvas aspect ratio">
        <option>9:16</option>
      </select>
    );
  }

  const value = currentPresetId(project);
  const orig = originalDims(project);

  return (
    <select
      className="ratio-select"
      title="Canvas aspect ratio"
      value={value}
      onChange={(e) => {
        const id = e.target.value;
        if (id === "custom") return;
        void (async () => {
          if (id === "original") {
            if (!orig) {
              toast("Import a video first to use Original", "info");
              return;
            }
            const ok = await dispatch(setProjectSettings({ width: orig.width, height: orig.height }));
            if (ok) toast(`Canvas ${orig.width}×${orig.height}`, "success");
            return;
          }
          const preset = ASPECT_PRESETS.find((p) => p.id === id);
          if (!preset) return;
          const ok = await dispatch(setProjectSettings({ width: preset.width, height: preset.height }));
          if (ok) toast(`Canvas ${preset.label}`, "success");
        })();
      }}
    >
      {ASPECT_PRESETS.map((p) => (
        <option key={p.id} value={p.id}>
          {p.label} ({p.width}×{p.height})
        </option>
      ))}
      <option value="original" disabled={!orig}>
        Original{orig ? ` (${orig.width}×${orig.height})` : ""}
      </option>
      {value === "custom" && (
        <option value="custom">
          Custom ({project.settings.width}×{project.settings.height})
        </option>
      )}
    </select>
  );
}
