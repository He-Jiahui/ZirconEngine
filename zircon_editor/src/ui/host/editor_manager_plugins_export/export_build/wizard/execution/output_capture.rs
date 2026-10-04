use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::ui::host::export_process_support::CapturedOutputChunk;

use super::super::output_tail::push_bounded_output_line;
use super::super::ExportWizardPipelineStageCommand;
use super::{ExportWizardCommandOutputLine, ExportWizardCommandOutputStream};

const MAX_CAPTURED_LINE_BYTES: usize = 16 * 1024;

pub(super) struct ExportWizardOutputCapture {
    stdout: StreamCapture,
    stderr: StreamCapture,
    manifest: ArtifactDestination,
}

pub(super) struct ExportWizardOutputCaptureResult {
    pub(super) stdout_lines: Vec<String>,
    pub(super) stderr_lines: Vec<String>,
    pub(super) artifact_lines: Vec<ExportWizardCommandOutputLine>,
}

impl ExportWizardOutputCapture {
    pub(super) fn open(command: &ExportWizardPipelineStageCommand) -> io::Result<Self> {
        let working_directory = if let Some(path) = command.native_working_dir.as_deref() {
            path.to_path_buf()
        } else if let Some(path) = command.working_dir.as_deref() {
            PathBuf::from(path)
        } else {
            std::env::current_dir()?
        };
        let paths = OutputCapturePaths {
            stdout: artifact_destination(command, "stdout_log", &working_directory)?,
            stderr: artifact_destination(command, "stderr_log", &working_directory)?,
            manifest: artifact_destination(command, "output_log_manifest", &working_directory)?,
        };
        Self::open_paths(paths)
    }

    fn open_paths(paths: OutputCapturePaths) -> io::Result<Self> {
        Ok(Self {
            stdout: StreamCapture::open(paths.stdout)?,
            stderr: StreamCapture::open(paths.stderr)?,
            manifest: paths.manifest,
        })
    }

    pub(super) fn record(
        &mut self,
        output: CapturedOutputChunk,
        finalize: bool,
        emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
    ) -> io::Result<()> {
        self.stdout.record(
            output.stdout,
            finalize,
            ExportWizardCommandOutputStream::Stdout,
            emit_output,
        )?;
        self.stderr.record(
            output.stderr,
            finalize,
            ExportWizardCommandOutputStream::Stderr,
            emit_output,
        )
    }

    pub(super) fn finish(self) -> io::Result<ExportWizardOutputCaptureResult> {
        let stdout = self.stdout.finish()?;
        let stderr = self.stderr.finish()?;
        let manifest = OutputLogManifest {
            version: 1,
            digest_algorithm: "blake3",
            stdout: OutputLogStreamManifest::from(&stdout),
            stderr: OutputLogStreamManifest::from(&stderr),
        };
        write_manifest(&self.manifest.io_path, &manifest)?;

        let artifact_lines = [
            ("stdout_log", stdout.locator.as_str()),
            ("stderr_log", stderr.locator.as_str()),
            ("output_log_manifest", self.manifest.locator.as_str()),
        ]
        .into_iter()
        .map(|(key, path)| ExportWizardCommandOutputLine {
            stream: ExportWizardCommandOutputStream::Stdout,
            line: format!("{key}={path}"),
        })
        .collect();

        Ok(ExportWizardOutputCaptureResult {
            stdout_lines: stdout.tail_lines,
            stderr_lines: stderr.tail_lines,
            artifact_lines,
        })
    }
}

struct StreamCapture {
    destination: ArtifactDestination,
    writer: File,
    hasher: blake3::Hasher,
    byte_count: u64,
    dropped_line_count: u64,
    tail_lines: VecDeque<String>,
    line_buffer: IncrementalLineBuffer,
}

impl StreamCapture {
    fn open(destination: ArtifactDestination) -> io::Result<Self> {
        ensure_parent(&destination.io_path)?;
        let writer = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&destination.io_path)?;
        Ok(Self {
            destination,
            writer,
            hasher: blake3::Hasher::new(),
            byte_count: 0,
            dropped_line_count: 0,
            tail_lines: VecDeque::new(),
            line_buffer: IncrementalLineBuffer::default(),
        })
    }

    fn record(
        &mut self,
        bytes: Vec<u8>,
        finalize: bool,
        stream: ExportWizardCommandOutputStream,
        emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
    ) -> io::Result<()> {
        self.writer.write_all(&bytes)?;
        self.hasher.update(&bytes);
        self.byte_count = self.byte_count.saturating_add(bytes.len() as u64);
        let dropped_line_count = &mut self.dropped_line_count;
        let tail_lines = &mut self.tail_lines;
        self.line_buffer
            .for_each_line(bytes, finalize, &mut |line| {
                *dropped_line_count = dropped_line_count
                    .saturating_add(push_bounded_output_line(tail_lines, line.clone()));
                emit_output(ExportWizardCommandOutputLine { stream, line });
            });
        Ok(())
    }

    fn finish(mut self) -> io::Result<StreamCaptureSummary> {
        self.writer.flush()?;
        self.writer.sync_all()?;
        Ok(StreamCaptureSummary {
            locator: self.destination.locator,
            byte_count: self.byte_count,
            digest: self.hasher.finalize().to_hex().to_string(),
            dropped_line_count: self.dropped_line_count,
            tail_lines: self.tail_lines.into_iter().collect(),
        })
    }
}

struct StreamCaptureSummary {
    locator: String,
    byte_count: u64,
    digest: String,
    dropped_line_count: u64,
    tail_lines: Vec<String>,
}

#[derive(Serialize)]
struct OutputLogManifest<'a> {
    version: u32,
    digest_algorithm: &'static str,
    stdout: OutputLogStreamManifest<'a>,
    stderr: OutputLogStreamManifest<'a>,
}

#[derive(Serialize)]
struct OutputLogStreamManifest<'a> {
    path: &'a str,
    byte_count: u64,
    digest: &'a str,
    tail_line_count: usize,
    dropped_line_count: u64,
}

impl<'a> From<&'a StreamCaptureSummary> for OutputLogStreamManifest<'a> {
    fn from(summary: &'a StreamCaptureSummary) -> Self {
        Self {
            path: &summary.locator,
            byte_count: summary.byte_count,
            digest: &summary.digest,
            tail_line_count: summary.tail_lines.len(),
            dropped_line_count: summary.dropped_line_count,
        }
    }
}

struct OutputCapturePaths {
    stdout: ArtifactDestination,
    stderr: ArtifactDestination,
    manifest: ArtifactDestination,
}

struct ArtifactDestination {
    locator: String,
    io_path: PathBuf,
}

fn artifact_destination(
    command: &ExportWizardPipelineStageCommand,
    key: &str,
    working_directory: &Path,
) -> io::Result<ArtifactDestination> {
    let locator = command
        .produced_artifacts
        .iter()
        .find(|artifact| artifact.key == key)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("export stage {:?} does not declare {key}", command.stage),
            )
        })?;
    let path = PathBuf::from(&locator);
    let io_path = if path.is_absolute() {
        path
    } else {
        working_directory.join(path)
    };
    Ok(ArtifactDestination { locator, io_path })
}

fn write_manifest(path: &Path, manifest: &OutputLogManifest<'_>) -> io::Result<()> {
    ensure_parent(path)?;
    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    file.write_all(&bytes)?;
    file.flush()?;
    file.sync_all()
}

fn ensure_parent(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[derive(Default)]
struct IncrementalLineBuffer {
    pending: Vec<u8>,
    scan_from: usize,
}

impl IncrementalLineBuffer {
    fn for_each_line(
        &mut self,
        mut bytes: Vec<u8>,
        finalize: bool,
        emit_line: &mut impl FnMut(String),
    ) {
        if self.pending.is_empty() {
            self.pending = bytes;
        } else {
            self.pending.append(&mut bytes);
        }
        let mut line_start = 0;
        for index in self.scan_from.min(self.pending.len())..self.pending.len() {
            let reached_newline = self.pending[index] == b'\n';
            let reached_limit = index + 1 - line_start == MAX_CAPTURED_LINE_BYTES
                && match self.pending.get(index + 1) {
                    Some(b'\n') => false,
                    Some(_) => true,
                    None => finalize,
                };
            if reached_newline || reached_limit {
                let line_end = if reached_newline { index } else { index + 1 };
                emit_line(decode_output_line(&self.pending[line_start..line_end]));
                line_start = index + 1;
            }
        }
        if line_start > 0 {
            self.pending.drain(..line_start);
        }
        if finalize && !self.pending.is_empty() {
            emit_line(decode_output_line(&self.pending));
            self.pending.clear();
        }
        self.scan_from = self.pending.len().saturating_sub(1);
    }

    #[cfg(test)]
    fn push(&mut self, bytes: Vec<u8>, finalize: bool) -> Vec<String> {
        let mut lines = Vec::new();
        self.for_each_line(bytes, finalize, &mut |line| lines.push(line));
        lines
    }
}

fn decode_output_line(bytes: &[u8]) -> String {
    let bytes = bytes.strip_suffix(b"\r").unwrap_or(bytes);
    String::from_utf8_lossy(bytes).to_string()
}

#[cfg(test)]
#[path = "tests/output_capture.rs"]
mod tests;
