use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use crate::error::HubError;
use crate::process::SupervisedChild;
use crate::state::{TaskCancellationToken, TaskExecutionOutcome};

use super::command::BuildCommand;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildExecutionReport {
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl BuildExecutionReport {
    /// Reports only the process-stage result. Staged artifacts still require BuildSet qualification.
    pub fn process_exited_successfully(&self) -> bool {
        self.status_code == Some(0)
    }

    pub fn summary_line(&self) -> String {
        self.stderr
            .lines()
            .rev()
            .chain(self.stdout.lines().rev())
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("build produced no output")
            .to_string()
    }

    pub fn recovery_hint(&self) -> String {
        match self.status_code {
            Some(code) => format!(
                "tools/zircon_build.py exited with code {code}; open Build History and fix the first reported error before retrying"
            ),
            None => "tools/zircon_build.py was terminated by the OS; check the build log and retry from Hub".to_string(),
        }
    }

    pub fn log_excerpt(&self) -> String {
        log_excerpt_from_streams(&self.stdout, &self.stderr)
    }
}

fn log_excerpt_from_streams(stdout: &str, stderr: &str) -> String {
    stderr
        .lines()
        .chain(stdout.lines())
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

const BUILD_PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(50);
const BUILD_CAPTURE_CREATE_ATTEMPTS: usize = 32;
const BUILD_REPORT_TAIL_BYTES: u64 = 64 * 1024;
static NEXT_BUILD_CAPTURE_ID: AtomicU64 = AtomicU64::new(1);

pub fn run_build_command(
    command: &BuildCommand,
    cancellation: &TaskCancellationToken,
) -> Result<TaskExecutionOutcome<BuildExecutionReport>, HubError> {
    if cancellation.is_cancellation_requested() {
        return Ok(TaskExecutionOutcome::Cancelled);
    }

    let (capture, stdout, stderr) = BuildOutputCapture::create(&command.capture_dir)?;
    let mut process = Command::new(&command.program);
    process
        .args(&command.args)
        .current_dir(&command.cwd)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    if cancellation.is_cancellation_requested() {
        let capture_error = capture.discard().err();
        return if let Some(capture_error) = capture_error {
            Err(combine_errors(
                HubError::message("build cancellation did not close its output capture"),
                [Some(capture_error)],
            ))
        } else {
            Ok(TaskExecutionOutcome::Cancelled)
        };
    }
    let spawn_result = SupervisedChild::spawn(&mut process, "build");
    drop(process);
    let mut child = match spawn_result {
        Ok(child) => child,
        Err(error) => {
            let capture_error = capture.discard().err();
            return Err(combine_errors(error, [capture_error]));
        }
    };

    if cancellation.is_cancellation_requested() {
        let termination_error = child.terminate_tree_and_reap().err();
        let capture_error = capture.discard().err();
        if termination_error.is_some() || capture_error.is_some() {
            return Err(combine_errors(
                HubError::message("build cancellation did not close cleanly"),
                [termination_error, capture_error],
            ));
        }
        return Ok(TaskExecutionOutcome::Cancelled);
    }

    loop {
        let status = match child.try_wait() {
            Ok(status) => status,
            Err(error) => {
                let termination_error = child.terminate_tree_and_reap().err();
                let capture_error = capture.discard().err();
                return Err(combine_errors(error, [termination_error, capture_error]));
            }
        };
        if let Some(status) = status {
            let termination_error = child.terminate_tree_and_reap().err();
            let output = match (termination_error, capture.collect()) {
                (None, Ok(output)) => output,
                (termination_error, output) => {
                    return Err(combine_errors(
                        HubError::message("build process tree did not close cleanly"),
                        [termination_error, output.err()],
                    ));
                }
            };
            return Ok(TaskExecutionOutcome::Completed(BuildExecutionReport {
                status_code: status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            }));
        }
        if cancellation.is_cancellation_requested() {
            let termination_error = child.terminate_tree_and_reap().err();
            let capture_error = capture.discard().err();
            if termination_error.is_some() || capture_error.is_some() {
                return Err(combine_errors(
                    HubError::message("build cancellation did not close cleanly"),
                    [termination_error, capture_error],
                ));
            }
            return Ok(TaskExecutionOutcome::Cancelled);
        }
        thread::sleep(BUILD_PROCESS_POLL_INTERVAL);
    }
}

fn combine_errors(
    primary: HubError,
    extras: impl IntoIterator<Item = Option<HubError>>,
) -> HubError {
    let mut messages = vec![primary.to_string()];
    messages.extend(extras.into_iter().flatten().map(|error| error.to_string()));
    HubError::message(messages.join("; "))
}

struct BuildOutputCapture {
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

struct CapturedBuildOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl BuildOutputCapture {
    fn create(directory: &Path) -> Result<(Self, File, File), HubError> {
        fs::create_dir_all(directory)?;
        for _ in 0..BUILD_CAPTURE_CREATE_ATTEMPTS {
            let capture_id = NEXT_BUILD_CAPTURE_ID.fetch_add(1, Ordering::Relaxed);
            let prefix = format!("{}-{capture_id}", std::process::id());
            let stdout_path = directory.join(format!("{prefix}.stdout"));
            let stderr_path = directory.join(format!("{prefix}.stderr"));
            let stdout = match create_capture_file(&stdout_path) {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            };
            let stderr = match create_capture_file(&stderr_path) {
                Ok(file) => file,
                Err(error) => {
                    let cleanup_error = fs::remove_file(&stdout_path).err().map(HubError::from);
                    if error.kind() == io::ErrorKind::AlreadyExists && cleanup_error.is_none() {
                        continue;
                    }
                    return Err(combine_errors(error.into(), [cleanup_error]));
                }
            };
            return Ok((
                Self {
                    stdout_path,
                    stderr_path,
                },
                stdout,
                stderr,
            ));
        }
        Err(HubError::message(format!(
            "failed to allocate a unique build output capture in {}",
            directory.display()
        )))
    }

    fn collect(self) -> Result<CapturedBuildOutput, HubError> {
        let stdout = read_capture_tail(&self.stdout_path);
        let stderr = read_capture_tail(&self.stderr_path);
        let cleanup = self.cleanup();
        match (stdout, stderr, cleanup) {
            (Ok(stdout), Ok(stderr), Ok(())) => Ok(CapturedBuildOutput { stdout, stderr }),
            (stdout, stderr, cleanup) => Err(combine_errors(
                HubError::message("failed to collect build output capture"),
                [stdout.err(), stderr.err(), cleanup.err()],
            )),
        }
    }

    fn discard(self) -> Result<(), HubError> {
        self.cleanup()
    }

    fn cleanup(&self) -> Result<(), HubError> {
        let stdout_error = remove_capture_file(&self.stdout_path)
            .err()
            .map(HubError::from);
        let stderr_error = remove_capture_file(&self.stderr_path)
            .err()
            .map(HubError::from);
        if stdout_error.is_none() && stderr_error.is_none() {
            Ok(())
        } else {
            Err(combine_errors(
                HubError::message("failed to remove build output capture"),
                [stdout_error, stderr_error],
            ))
        }
    }
}

fn read_capture_tail(path: &Path) -> Result<Vec<u8>, HubError> {
    let mut file = File::open(path).map_err(HubError::from)?;
    let length = file.metadata().map_err(HubError::from)?.len();
    let start = length.saturating_sub(BUILD_REPORT_TAIL_BYTES);
    file.seek(SeekFrom::Start(start)).map_err(HubError::from)?;
    let capacity = usize::try_from(length - start)
        .map_err(|_| HubError::message("build output tail exceeds addressable memory"))?;
    let mut output = Vec::with_capacity(capacity);
    file.read_to_end(&mut output).map_err(HubError::from)?;
    Ok(output)
}

fn create_capture_file(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

fn remove_capture_file(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
#[path = "tests/runner.rs"]
mod tests;
