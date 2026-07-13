use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::jobs::EditorJobSystem;

use super::ExportProcessError;

static OUTPUT_CAPTURE_SEQUENCE: AtomicU64 = AtomicU64::new(1);
const OUTPUT_CAPTURE_READ_CHUNK_BYTES: u64 = 64 * 1024;

pub(in crate::ui::host) trait ExportProcessJoin: Sync {
    fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send;
}

impl ExportProcessJoin for EditorJobSystem {
    fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.join(task_a, task_b)
    }
}

#[cfg(test)]
impl ExportProcessJoin for zircon_runtime::core::runtime::tasks::TaskPool {
    fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.join(task_a, task_b)
    }
}

pub(in crate::ui::host) struct ExportProcessOutputReader {
    file: File,
    label: String,
    stream_name: &'static str,
}

impl ExportProcessOutputReader {
    fn read_available(&mut self) -> Result<Vec<u8>, ExportProcessError> {
        let mut bytes = Vec::new();
        self.file
            .by_ref()
            .take(OUTPUT_CAPTURE_READ_CHUNK_BYTES)
            .read_to_end(&mut bytes)
            .map_err(|error| {
                ExportProcessError::io(
                    "failed to read export output capture",
                    self.label.clone(),
                    Some(self.stream_name),
                    None,
                    error,
                )
            })?;
        Ok(bytes)
    }
}

pub(in crate::ui::host) struct ExportProcessOutputReaders {
    stdout: ExportProcessOutputReader,
    stderr: ExportProcessOutputReader,
    _cleanup: OutputCaptureCleanup,
}

#[derive(Default)]
pub(in crate::ui::host) struct CapturedOutputChunk {
    pub(in crate::ui::host) stdout: Vec<u8>,
    pub(in crate::ui::host) stderr: Vec<u8>,
}

pub(in crate::ui::host) fn create_output_capture(
    label: &str,
) -> Result<(File, File, ExportProcessOutputReaders), ExportProcessError> {
    loop {
        let sequence = OUTPUT_CAPTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let prefix = format!("zircon-export-{}-{sequence}", std::process::id());
        let stdout_path = std::env::temp_dir().join(format!("{prefix}-stdout.log"));
        let stderr_path = std::env::temp_dir().join(format!("{prefix}-stderr.log"));
        let stdout_writer = match create_capture_writer(&stdout_path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(ExportProcessError::io(
                    "failed to create export output capture",
                    label,
                    Some("stdout"),
                    Some(stdout_path),
                    error,
                ));
            }
        };
        let stderr_writer = match create_capture_writer(&stderr_path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                drop(stdout_writer);
                let _ = fs::remove_file(&stdout_path);
                continue;
            }
            Err(error) => {
                drop(stdout_writer);
                let _ = fs::remove_file(&stdout_path);
                return Err(ExportProcessError::io(
                    "failed to create export output capture",
                    label,
                    Some("stderr"),
                    Some(stderr_path),
                    error,
                ));
            }
        };
        let cleanup = OutputCaptureCleanup {
            stdout_path,
            stderr_path,
        };
        let stdout_reader = match open_capture_reader(&cleanup.stdout_path, label, "stdout") {
            Ok(reader) => reader,
            Err(error) => {
                drop(stdout_writer);
                drop(stderr_writer);
                return Err(ExportProcessError::io(
                    "failed to open export output capture for reading",
                    label,
                    Some("stdout"),
                    Some(cleanup.stdout_path.clone()),
                    error,
                ));
            }
        };
        let stderr_reader = match open_capture_reader(&cleanup.stderr_path, label, "stderr") {
            Ok(reader) => reader,
            Err(error) => {
                drop(stdout_reader);
                drop(stdout_writer);
                drop(stderr_writer);
                return Err(ExportProcessError::io(
                    "failed to open export output capture for reading",
                    label,
                    Some("stderr"),
                    Some(cleanup.stderr_path.clone()),
                    error,
                ));
            }
        };
        return Ok((
            stdout_writer,
            stderr_writer,
            ExportProcessOutputReaders {
                stdout: stdout_reader,
                stderr: stderr_reader,
                _cleanup: cleanup,
            },
        ));
    }
}

pub(in crate::ui::host) fn join_output_with_poll<J, P, T>(
    jobs: &J,
    readers: &mut ExportProcessOutputReaders,
    poll: P,
) -> (Result<CapturedOutputChunk, ExportProcessError>, T)
where
    J: ExportProcessJoin,
    P: FnOnce() -> T + Send,
    T: Send,
{
    let stdout = &mut readers.stdout;
    let stderr = &mut readers.stderr;
    let (stdout, (stderr, polled)) = jobs.join(
        move || stdout.read_available(),
        move || jobs.join(move || stderr.read_available(), poll),
    );
    let output =
        stdout.and_then(|stdout| stderr.map(|stderr| CapturedOutputChunk { stdout, stderr }));
    (output, polled)
}

pub(in crate::ui::host) fn final_output_drain<J: ExportProcessJoin>(
    jobs: &J,
    readers: &mut ExportProcessOutputReaders,
) -> Result<CapturedOutputChunk, ExportProcessError> {
    let mut drained = CapturedOutputChunk::default();
    loop {
        let stdout = &mut readers.stdout;
        let stderr = &mut readers.stderr;
        let (stdout, stderr) = jobs.join(
            move || stdout.read_available(),
            move || stderr.read_available(),
        );
        let stdout = stdout?;
        let stderr = stderr?;
        let complete = stdout.is_empty() && stderr.is_empty();
        drained.stdout.extend(stdout);
        drained.stderr.extend(stderr);
        if complete {
            return Ok(drained);
        }
    }
}

fn create_capture_writer(path: &PathBuf) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

fn open_capture_reader(
    path: &PathBuf,
    label: &str,
    stream_name: &'static str,
) -> io::Result<ExportProcessOutputReader> {
    OpenOptions::new()
        .read(true)
        .open(path)
        .map(|file| ExportProcessOutputReader {
            file,
            label: label.to_string(),
            stream_name,
        })
}

struct OutputCaptureCleanup {
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

impl Drop for OutputCaptureCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.stdout_path);
        let _ = fs::remove_file(&self.stderr_path);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zircon_runtime::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};

    use super::*;

    #[test]
    fn single_worker_join_reads_capture_and_polls_without_blocking() {
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
        let (mut stdout_writer, mut stderr_writer, mut readers) =
            create_output_capture("single-worker contract").expect("capture should open");
        stdout_writer.write_all(b"stdout\n").expect("stdout write");
        stderr_writer.write_all(b"stderr\n").expect("stderr write");

        let (output, polled) = join_output_with_poll(&pool, &mut readers, || 42_u32);

        let output = output.expect("capture reads should succeed");
        assert_eq!(output.stdout, b"stdout\n");
        assert_eq!(output.stderr, b"stderr\n");
        assert_eq!(polled, 42);
    }

    #[test]
    fn capture_reader_yields_at_the_byte_budget_while_output_remains() {
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
        let (mut stdout_writer, _stderr_writer, mut readers) =
            create_output_capture("bounded capture contract").expect("capture should open");
        let output = vec![b'x'; OUTPUT_CAPTURE_READ_CHUNK_BYTES as usize * 2 + 7];
        stdout_writer.write_all(&output).expect("stdout write");

        let (first, first_poll) = join_output_with_poll(&pool, &mut readers, || 1_u32);
        let (second, second_poll) = join_output_with_poll(&pool, &mut readers, || 2_u32);

        assert_eq!(
            first.expect("first capture read").stdout.len(),
            OUTPUT_CAPTURE_READ_CHUNK_BYTES as usize
        );
        assert_eq!(
            second.expect("second capture read").stdout.len(),
            OUTPUT_CAPTURE_READ_CHUNK_BYTES as usize
        );
        assert_eq!((first_poll, second_poll), (1, 2));
    }
}
