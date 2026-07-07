//! Invoke `ffmpeg` / `ffprobe` as subprocesses. Phase 0 uses the user's installed FFmpeg
//! binaries (no link-time dependency on libav); linked decode/encode via `ffmpeg-the-third`
//! lands once vcpkg/FFMPEG_DIR is wired up for all dev/CI environments.

use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FfmpegCliError {
    #[error("ffmpeg/ffprobe not found on PATH; install FFmpeg to use media I/O")]
    NotFound,
    #[error("failed to run {tool}: {message}")]
    SpawnFailed { tool: &'static str, message: String },
    #[error("ffmpeg exited with status {0}")]
    NonZeroExit(i32),
    #[error("unexpected ffmpeg output: {0}")]
    BadOutput(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

static FFMPEG: OnceLock<PathBuf> = OnceLock::new();
static FFPROBE: OnceLock<PathBuf> = OnceLock::new();

fn resolve_tool(name: &str, cache: &OnceLock<PathBuf>) -> Result<PathBuf, FfmpegCliError> {
    if let Some(path) = cache.get() {
        return Ok(path.clone());
    }
    let found = which_tool(name).ok_or(FfmpegCliError::NotFound)?;
    let _ = cache.set(found.clone());
    Ok(found)
}

fn which_tool(name: &str) -> Option<PathBuf> {
    let with_exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(&with_exe);
            candidate.is_file().then_some(candidate)
        })
    })
}

pub fn ffmpeg_path() -> Result<PathBuf, FfmpegCliError> {
    resolve_tool("ffmpeg", &FFMPEG)
}

pub fn ffprobe_path() -> Result<PathBuf, FfmpegCliError> {
    resolve_tool("ffprobe", &FFPROBE)
}

/// Returns true when both `ffmpeg` and `ffprobe` are discoverable on PATH.
pub fn is_available() -> bool {
    ffmpeg_path().is_ok() && ffprobe_path().is_ok()
}

#[derive(Debug, Clone)]
pub struct ProbedVideo {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration_secs: f64,
}

pub fn probe_video(path: &Path) -> Result<ProbedVideo, FfmpegCliError> {
    let output = Command::new(ffprobe_path()?)
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,r_frame_rate,duration",
            "-show_entries",
            "format=duration",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .map_err(|e| FfmpegCliError::SpawnFailed {
            tool: "ffprobe",
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(FfmpegCliError::NonZeroExit(
            output.status.code().unwrap_or(-1),
        ));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| FfmpegCliError::BadOutput(e.to_string()))?;

    let stream = json
        .get("streams")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| FfmpegCliError::BadOutput("no video stream".into()))?;

    let width = stream
        .get("width")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| FfmpegCliError::BadOutput("missing width".into()))? as u32;
    let height = stream
        .get("height")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| FfmpegCliError::BadOutput("missing height".into()))? as u32;

    let fps = stream
        .get("r_frame_rate")
        .and_then(|v| v.as_str())
        .map(parse_rational)
        .transpose()?
        .unwrap_or(30.0);

    let duration_secs = stream
        .get("duration")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| {
            json.get("format")
                .and_then(|f| f.get("duration"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
        })
        .ok_or_else(|| FfmpegCliError::BadOutput("missing duration".into()))?;

    Ok(ProbedVideo {
        width,
        height,
        fps,
        duration_secs,
    })
}

fn parse_rational(s: &str) -> Result<f64, FfmpegCliError> {
    if let Some((num, den)) = s.split_once('/') {
        let num: f64 = num
            .parse()
            .map_err(|e: std::num::ParseFloatError| FfmpegCliError::BadOutput(e.to_string()))?;
        let den: f64 = den
            .parse()
            .map_err(|e: std::num::ParseFloatError| FfmpegCliError::BadOutput(e.to_string()))?;
        if den == 0.0 {
            return Err(FfmpegCliError::BadOutput(format!(
                "zero denominator in {s}"
            )));
        }
        Ok(num / den)
    } else {
        s.parse::<f64>()
            .map_err(|e: std::num::ParseFloatError| FfmpegCliError::BadOutput(e.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct RgbaFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Sequential RGBA frame reader backed by a long-lived `ffmpeg` decode pipe.
pub struct VideoReader {
    child: Child,
    stdout: BufReader<ChildStdout>,
    width: u32,
    height: u32,
    frame_bytes: usize,
}

type ChildStdout = std::process::ChildStdout;

impl VideoReader {
    pub fn open(path: &Path, start_secs: f64) -> Result<Self, FfmpegCliError> {
        let probed = probe_video(path)?;
        let mut child = Command::new(ffmpeg_path()?)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-ss",
                &format!("{start_secs:.6}"),
                "-i",
            ])
            .arg(path)
            .args(["-an", "-f", "rawvideo", "-pix_fmt", "rgba", "pipe:1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| FfmpegCliError::SpawnFailed {
                tool: "ffmpeg",
                message: e.to_string(),
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| FfmpegCliError::BadOutput("no stdout".into()))?;
        let frame_bytes = (probed.width * probed.height * 4) as usize;

        Ok(Self {
            child,
            stdout: BufReader::new(stdout),
            width: probed.width,
            height: probed.height,
            frame_bytes,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn read_frame(&mut self) -> Result<Option<RgbaFrame>, FfmpegCliError> {
        let mut buf = vec![0u8; self.frame_bytes];
        match self.stdout.read_exact(&mut buf) {
            Ok(()) => Ok(Some(RgbaFrame {
                width: self.width,
                height: self.height,
                pixels: buf,
            })),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

impl Drop for VideoReader {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// H.264 MP4 encoder that accepts raw RGBA frames on stdin.
pub struct VideoEncoder {
    child: Child,
    frame_bytes: usize,
}

impl VideoEncoder {
    pub fn open(
        output_path: &Path,
        width: u32,
        height: u32,
        fps: f64,
    ) -> Result<Self, FfmpegCliError> {
        let child = Command::new(ffmpeg_path()?)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgba",
                "-s",
                &format!("{width}x{height}"),
                "-r",
                &format!("{fps:.6}"),
                "-i",
                "pipe:0",
                "-an",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-movflags",
                "+faststart",
            ])
            .arg(output_path)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .map_err(|e| FfmpegCliError::SpawnFailed {
                tool: "ffmpeg",
                message: e.to_string(),
            })?;

        Ok(Self {
            child,
            frame_bytes: (width * height * 4) as usize,
        })
    }

    pub fn write_frame(&mut self, pixels: &[u8]) -> Result<(), FfmpegCliError> {
        if pixels.len() != self.frame_bytes {
            return Err(FfmpegCliError::BadOutput(format!(
                "expected {} bytes, got {}",
                self.frame_bytes,
                pixels.len()
            )));
        }
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| FfmpegCliError::BadOutput("encoder stdin closed".into()))?;
        stdin.write_all(pixels)?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<(), FfmpegCliError> {
        drop(self.child.stdin.take());
        let status = self.child.wait()?;
        if !status.success() {
            return Err(FfmpegCliError::NonZeroExit(status.code().unwrap_or(-1)));
        }
        Ok(())
    }
}
