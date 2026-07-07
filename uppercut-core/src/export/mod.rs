//! Timeline → decode → wgpu composite → encode export pipeline (Phase 0 milestone).

use crate::commands::ExportPreset;
use crate::compose::{ComposeError, Compositor};
use crate::media::{FfmpegCliError, RgbaFrame, VideoEncoder, VideoReader};
use crate::project::{Clip, MediaKind, Project, TrackKind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("{0}")]
    Ffmpeg(#[from] FfmpegCliError),
    #[error("{0}")]
    Compose(#[from] ComposeError),
    #[error("no enabled video clips on the timeline")]
    EmptyTimeline,
    #[error("media not found: {0}")]
    MediaNotFound(uuid::Uuid),
    #[error("media {0} is not video")]
    NotVideo(uuid::Uuid),
}

#[derive(Debug, Clone, Copy)]
pub struct ExportSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}

impl ExportSettings {
    pub fn from_preset(preset: &ExportPreset, project: &Project) -> Self {
        match preset {
            ExportPreset::TikTok9x16 => Self {
                width: 1080,
                height: 1920,
                fps: project.settings.fps,
            },
            ExportPreset::Youtube16x9 => Self {
                width: 1920,
                height: 1080,
                fps: project.settings.fps,
            },
            ExportPreset::Custom { width, height, fps } => Self {
                width: *width,
                height: *height,
                fps: *fps,
            },
        }
    }
}

struct ActiveLayer {
    media_id: uuid::Uuid,
    path: PathBuf,
    source_time: f64,
}

struct DecoderState {
    path: PathBuf,
    reader: Option<VideoReader>,
    last_source_time: f64,
}

impl DecoderState {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            reader: None,
            last_source_time: f64::NAN,
        }
    }

    fn frame_at(&mut self, source_time: f64) -> Result<Option<RgbaFrame>, FfmpegCliError> {
        let needs_reopen = self.reader.is_none()
            || source_time + 1e-6 < self.last_source_time
            || (source_time - self.last_source_time).abs() > 0.5;

        if needs_reopen {
            self.reader = Some(VideoReader::open(&self.path, source_time)?);
        }

        let reader = self.reader.as_mut().expect("reader just opened");
        let frame = reader.read_frame()?;
        self.last_source_time = source_time;
        Ok(frame)
    }
}

/// Render the project's video tracks to an MP4 file.
pub fn export_project(
    project: &Project,
    output_path: &Path,
    preset: ExportPreset,
) -> Result<(), ExportError> {
    if !crate::media::ffmpeg_available() {
        return Err(FfmpegCliError::NotFound.into());
    }

    let settings = ExportSettings::from_preset(&preset, project);
    let duration_secs = timeline_duration(project);
    if duration_secs <= 0.0 {
        return Err(ExportError::EmptyTimeline);
    }

    if !has_video_content(project) {
        return Err(ExportError::EmptyTimeline);
    }

    let total_frames = (duration_secs * settings.fps).ceil() as u64;
    if total_frames == 0 {
        return Err(ExportError::EmptyTimeline);
    }

    let mut compositor = Compositor::new(settings.width, settings.height)?;
    let mut encoder =
        VideoEncoder::open(output_path, settings.width, settings.height, settings.fps)?;

    let mut decoders: HashMap<uuid::Uuid, DecoderState> = HashMap::new();

    for frame_idx in 0..total_frames {
        let t = frame_idx as f64 / settings.fps;
        let layers = active_layers(project, t)?;

        let mut rgba_layers = Vec::with_capacity(layers.len());
        for layer in &layers {
            let decoder = decoders
                .entry(layer.media_id)
                .or_insert_with(|| DecoderState::new(layer.path.clone()));

            if let Some(frame) = decoder.frame_at(layer.source_time)? {
                rgba_layers.push(frame);
            }
        }

        let pixels = compositor.composite(&rgba_layers)?;
        encoder.write_frame(&pixels)?;
    }

    encoder.finish()?;
    Ok(())
}

fn has_video_content(project: &Project) -> bool {
    project.tracks.iter().any(|track| {
        track.kind == TrackKind::Video
            && track.clips.iter().any(|clip| match clip {
                Clip::Video(c) => c.enabled,
                _ => false,
            })
    })
}

fn timeline_duration(project: &Project) -> f64 {
    project
        .tracks
        .iter()
        .flat_map(|t| t.clips.iter())
        .map(|c| c.end_secs())
        .fold(0.0_f64, f64::max)
}

fn active_layers(project: &Project, t: f64) -> Result<Vec<ActiveLayer>, ExportError> {
    let mut layers = Vec::new();

    for track in &project.tracks {
        if track.kind != TrackKind::Video {
            continue;
        }
        for clip in &track.clips {
            let Clip::Video(v) = clip else { continue };
            if !v.enabled {
                continue;
            }
            let start = v.position_secs;
            let end = start + (v.source_out_secs - v.source_in_secs);
            if t >= start && t < end {
                let media = project
                    .find_media(v.media_id)
                    .ok_or(ExportError::MediaNotFound(v.media_id))?;
                if media.kind != MediaKind::Video && media.kind != MediaKind::Image {
                    return Err(ExportError::NotVideo(v.media_id));
                }
                let source_time = v.source_in_secs + (t - start);
                layers.push(ActiveLayer {
                    media_id: v.media_id,
                    path: media.path.clone(),
                    source_time,
                });
                break; // at most one clip per track at time t
            }
        }
    }

    Ok(layers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{Project, Settings};
    use std::process::Command;

    fn ffmpeg_available() -> bool {
        crate::media::ffmpeg_available()
    }

    fn generate_test_video(path: &Path) {
        let status = Command::new(crate::media::ffmpeg_path().unwrap())
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=1:size=320x240:rate=30",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
            ])
            .arg(path)
            .status()
            .expect("ffmpeg");
        assert!(status.success());
    }

    #[test]
    fn export_cuts_only_timeline_to_mp4() {
        if !ffmpeg_available() {
            eprintln!("skipping export test: ffmpeg not on PATH");
            return;
        }

        let dir = std::env::temp_dir().join(format!("uppercut-export-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let video_path = dir.join("src.mp4");
        let output_path = dir.join("out.mp4");
        generate_test_video(&video_path);

        let mut project = Project::new("export-test", Settings::default());
        let media_id = uuid::Uuid::new_v4();
        project.media.push(crate::project::MediaItem {
            id: media_id,
            path: video_path.clone(),
            kind: MediaKind::Video,
            duration_secs: Some(1.0),
            width: Some(320),
            height: Some(240),
            fps: Some(30.0),
        });
        let track = crate::project::Track::new(TrackKind::Video, "V1");
        project.tracks.push(track);
        project.tracks[0]
            .clips
            .push(Clip::Video(crate::project::MediaClip {
                id: uuid::Uuid::new_v4(),
                media_id,
                position_secs: 0.0,
                source_in_secs: 0.0,
                source_out_secs: 0.5,
                gain_db: 0.0,
                enabled: true,
            }));

        export_project(
            &project,
            &output_path,
            ExportPreset::Custom {
                width: 320,
                height: 240,
                fps: 30.0,
            },
        )
        .expect("export");

        assert!(output_path.is_file());
        let meta = std::fs::metadata(&output_path).unwrap();
        assert!(meta.len() > 0);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn timeline_duration_uses_latest_clip_end() {
        let mut project = Project::new("t", Settings::default());
        let track = crate::project::Track::new(TrackKind::Video, "V1");
        project.tracks.push(track);
        project.tracks[0]
            .clips
            .push(Clip::Video(crate::project::MediaClip {
                id: uuid::Uuid::new_v4(),
                media_id: uuid::Uuid::new_v4(),
                position_secs: 1.0,
                source_in_secs: 0.0,
                source_out_secs: 2.0,
                gain_db: 0.0,
                enabled: true,
            }));
        assert!((timeline_duration(&project) - 3.0).abs() < 1e-9);
    }
}
