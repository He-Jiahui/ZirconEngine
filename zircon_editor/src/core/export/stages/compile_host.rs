use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use zircon_runtime_interface::export::{ExportPreset, ExportTargetMode};

const ZIRCON_BUILD_SCRIPT: &str = "tools/zircon_build.py";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZirconBuildCommand {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub working_directory: PathBuf,
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
        ZirconBuildCommand {
            program: self.python.clone(),
            args,
            working_directory: self.repo_root.clone(),
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
        let output = Command::new(&command.program)
            .args(&command.args)
            .current_dir(&command.working_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|source| ZirconBuildCommandError::Spawn {
                program: command.program.clone(),
                source,
            })?;
        if !output.status.success() {
            return Err(ZirconBuildCommandError::Exit {
                program: command.program.clone(),
                code: output.status.code(),
                stderr: output.stderr,
            });
        }
        Ok(ZirconBuildCommandExecution {
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

#[derive(Debug)]
pub enum ZirconBuildCommandError {
    Spawn {
        program: OsString,
        source: std::io::Error,
    },
    Exit {
        program: OsString,
        code: Option<i32>,
        stderr: Vec<u8>,
    },
}

impl fmt::Display for ZirconBuildCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { program, source } => write!(
                formatter,
                "failed to start Zircon build command {:?}: {source}",
                program
            ),
            Self::Exit {
                program,
                code,
                stderr,
            } => write!(
                formatter,
                "Zircon build command {:?} exited with {code:?}: {}",
                program,
                String::from_utf8_lossy(stderr)
            ),
        }
    }
}

impl Error for ZirconBuildCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } => Some(source),
            Self::Exit { .. } => None,
        }
    }
}
