use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::JoinHandle;

use zircon_runtime_interface::export::{ExportPreset, ExportTargetMode};

const ZIRCON_BUILD_SCRIPT: &str = "tools/zircon_build.py";
const MAX_COMMAND_OUTPUT_TAIL_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZirconBuildCommand {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub working_directory: PathBuf,
    pub stdout_log: PathBuf,
    pub stderr_log: PathBuf,
    pub output_manifest: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZirconBuildCommandExecution {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub trait ZirconBuildCommandRunner {
    type Error: Error + Send + Sync + 'static;

    fn run(
        &mut self,
        command: &ZirconBuildCommand,
    ) -> Result<ZirconBuildCommandExecution, Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompileHostStage {
    repo_root: PathBuf,
    output_root: PathBuf,
    python: OsString,
    cargo: OsString,
    locked: bool,
    dry_run: bool,
}

impl CompileHostStage {
    pub fn new(repo_root: impl Into<PathBuf>, output_root: impl Into<PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
            output_root: output_root.into(),
            python: OsString::from("python"),
            cargo: OsString::from("cargo"),
            locked: true,
            dry_run: false,
        }
    }

    pub fn with_python(mut self, python: impl Into<OsString>) -> Self {
        self.python = python.into();
        self
    }

    pub fn with_cargo(mut self, cargo: impl Into<OsString>) -> Self {
        self.cargo = cargo.into();
        self
    }

    pub fn without_lock(mut self) -> Self {
        self.locked = false;
        self
    }

    pub fn with_dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    pub fn output_root(&self) -> &Path {
        &self.output_root
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    pub fn python_program(&self) -> &std::ffi::OsStr {
        &self.python
    }

    pub fn cargo_program(&self) -> &std::ffi::OsStr {
        &self.cargo
    }

    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn staged_engine_root(&self) -> PathBuf {
        self.output_root.join("ZirconEngine")
    }

    pub fn command(&self, preset: &ExportPreset) -> ZirconBuildCommand {
        let targets = match preset.target_mode {
            ExportTargetMode::ClientRuntime => "hub,editor,runtime",
            ExportTargetMode::ServerRuntime => "runtime",
        };
        let runtime_feature = match preset.target_mode {
            ExportTargetMode::ClientRuntime => "target-client",
            ExportTargetMode::ServerRuntime => "target-server",
        };
        let mode = if preset.debug { "debug" } else { "release" };
        let mut args = vec![
            self.repo_root.join(ZIRCON_BUILD_SCRIPT).into_os_string(),
            OsString::from("--targets"),
            OsString::from(targets),
            OsString::from("--out"),
            self.output_root.clone().into_os_string(),
            OsString::from("--mode"),
            OsString::from(mode),
            OsString::from("--runtime-features"),
            OsString::from(runtime_feature),
            OsString::from("--cargo"),
            self.cargo.clone(),
        ];
        if !self.locked {
            args.push(OsString::from("--no-locked"));
        }
        if self.dry_run {
            args.push(OsString::from("--dry-run"));
        }
        let log_root = self.output_root.join("stages").join("compile_host");
        ZirconBuildCommand {
            program: self.python.clone(),
            args,
            working_directory: self.repo_root.clone(),
            stdout_log: log_root.join("stdout.log"),
            stderr_log: log_root.join("stderr.log"),
            output_manifest: log_root.join("output-log.json"),
        }
    }

    pub fn execute<R>(
        &self,
        preset: &ExportPreset,
        runner: &mut R,
    ) -> Result<ZirconBuildCommandExecution, R::Error>
    where
        R: ZirconBuildCommandRunner,
    {
        runner.run(&self.command(preset))
    }
}

#[derive(Default)]
pub struct SystemZirconBuildCommandRunner;

impl ZirconBuildCommandRunner for SystemZirconBuildCommandRunner {
    type Error = ZirconBuildCommandError;

    fn run(
        &mut self,
        command: &ZirconBuildCommand,
    ) -> Result<ZirconBuildCommandExecution, Self::Error> {
        let stdout_log = create_output_file(&command.stdout_log).map_err(|source| {
            ZirconBuildCommandError::io("create stdout log", &command.stdout_log, source)
        })?;
        let stderr_log = create_output_file(&command.stderr_log).map_err(|source| {
            ZirconBuildCommandError::io("create stderr log", &command.stderr_log, source)
        })?;
        let mut child = Command::new(&command.program)
            .args(&command.args)
            .current_dir(&command.working_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| ZirconBuildCommandError::Spawn {
                program: command.program.clone(),
                source,
            })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            ZirconBuildCommandError::io(
                "open stdout pipe",
                &command.stdout_log,
                std::io::Error::other("spawned Zircon build command has no stdout pipe"),
            )
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            ZirconBuildCommandError::io(
                "open stderr pipe",
                &command.stderr_log,
                std::io::Error::other("spawned Zircon build command has no stderr pipe"),
            )
        })?;
        let stdout_capture = std::thread::spawn(move || capture_output_stream(stdout, stdout_log));
        let stderr_capture = std::thread::spawn(move || capture_output_stream(stderr, stderr_log));
        let status = child.wait().map_err(|source| {
            ZirconBuildCommandError::io(
                "wait for Zircon build command",
                &command.output_manifest,
                source,
            )
        })?;
        let stdout = join_output_capture(stdout_capture, "stdout").map_err(|source| {
            ZirconBuildCommandError::io("capture stdout", &command.stdout_log, source)
        })?;
        let stderr = join_output_capture(stderr_capture, "stderr").map_err(|source| {
            ZirconBuildCommandError::io("capture stderr", &command.stderr_log, source)
        })?;
        write_output_manifest(command, &stdout, &stderr).map_err(|source| {
            ZirconBuildCommandError::io("write output manifest", &command.output_manifest, source)
        })?;
        if !status.success() {
            return Err(ZirconBuildCommandError::Exit {
                program: command.program.clone(),
                code: status.code(),
                stderr_tail: stderr.tail,
            });
        }
        Ok(ZirconBuildCommandExecution {
            stdout: stdout.tail,
            stderr: stderr.tail,
        })
    }
}

#[derive(Debug)]
pub enum ZirconBuildCommandError {
    Spawn {
        program: OsString,
        source: std::io::Error,
    },
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    Exit {
        program: OsString,
        code: Option<i32>,
        stderr_tail: Vec<u8>,
    },
}

impl ZirconBuildCommandError {
    fn io(operation: &'static str, path: &Path, source: std::io::Error) -> Self {
        Self::Io {
            operation,
            path: path.to_path_buf(),
            source,
        }
    }
}

impl fmt::Display for ZirconBuildCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { program, source } => write!(
                formatter,
                "failed to start Zircon build command {:?}: {source}",
                program
            ),
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "failed to {operation} at {}: {source}",
                path.display()
            ),
            Self::Exit {
                program,
                code,
                stderr_tail,
            } => write!(
                formatter,
                "Zircon build command {:?} exited with {code:?}: {}",
                program,
                String::from_utf8_lossy(stderr_tail)
            ),
        }
    }
}

impl Error for ZirconBuildCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } | Self::Io { source, .. } => Some(source),
            Self::Exit { .. } => None,
        }
    }
}

struct CapturedCommandOutput {
    tail: Vec<u8>,
    byte_count: u64,
    digest: blake3::Hash,
}

fn create_output_file(path: &Path) -> std::io::Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(path)
}

fn capture_output_stream(
    mut reader: impl Read,
    mut output: File,
) -> std::io::Result<CapturedCommandOutput> {
    let mut tail = std::collections::VecDeque::with_capacity(MAX_COMMAND_OUTPUT_TAIL_BYTES);
    let mut byte_count = 0_u64;
    let mut digest = blake3::Hasher::new();
    let mut chunk = [0_u8; 16 * 1024];
    loop {
        let read = reader.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        let bytes = &chunk[..read];
        output.write_all(bytes)?;
        digest.update(bytes);
        byte_count = byte_count.saturating_add(read as u64);
        tail.extend(bytes.iter().copied());
        while tail.len() > MAX_COMMAND_OUTPUT_TAIL_BYTES {
            tail.pop_front();
        }
    }
    output.sync_all()?;
    Ok(CapturedCommandOutput {
        tail: tail.into_iter().collect(),
        byte_count,
        digest: digest.finalize(),
    })
}

fn join_output_capture(
    capture: JoinHandle<std::io::Result<CapturedCommandOutput>>,
    stream: &'static str,
) -> std::io::Result<CapturedCommandOutput> {
    capture
        .join()
        .map_err(|_| std::io::Error::other(format!("{stream} capture thread panicked")))?
}

fn write_output_manifest(
    command: &ZirconBuildCommand,
    stdout: &CapturedCommandOutput,
    stderr: &CapturedCommandOutput,
) -> std::io::Result<()> {
    let manifest = serde_json::json!({
        "format_version": 1,
        "digest_algorithm": "blake3",
        "stdout": {
            "path": command.stdout_log.display().to_string(),
            "byte_count": stdout.byte_count,
            "digest": stdout.digest.to_hex().to_string(),
            "tail_byte_count": stdout.tail.len(),
        },
        "stderr": {
            "path": command.stderr_log.display().to_string(),
            "byte_count": stderr.byte_count,
            "digest": stderr.digest.to_hex().to_string(),
            "tail_byte_count": stderr.tail.len(),
        },
    });
    let encoded = serde_json::to_vec_pretty(&manifest).map_err(std::io::Error::other)?;
    let mut file = create_output_file(&command.output_manifest)?;
    file.write_all(&encoded)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_runner_streams_full_logs_and_bounds_memory_tails() {
        let root = std::env::temp_dir().join(format!(
            "zircon-editor-system-build-output-{}-{:x}",
            std::process::id(),
            fixture_nonce()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let log_path = root.join("stdout.log");
        let bytes = (0..(MAX_COMMAND_OUTPUT_TAIL_BYTES + 8192))
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>();

        let captured = capture_output_stream(
            std::io::Cursor::new(bytes.clone()),
            File::create(&log_path).unwrap(),
        )
        .unwrap();

        assert_eq!(captured.byte_count, bytes.len() as u64);
        assert_eq!(captured.digest, blake3::hash(&bytes));
        assert_eq!(captured.tail.len(), MAX_COMMAND_OUTPUT_TAIL_BYTES);
        assert_eq!(
            captured.tail,
            bytes[bytes.len() - MAX_COMMAND_OUTPUT_TAIL_BYTES..]
        );
        assert_eq!(fs::read(&log_path).unwrap(), bytes);
        let _ = fs::remove_dir_all(root);
    }

    fn fixture_nonce() -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        std::time::SystemTime::now().hash(&mut hasher);
        hasher.finish()
    }
}
