use std::path::{Path, PathBuf};
use std::process::Command;

use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime_interface::project::RelPath;

use crate::core::process::{
    configure_process_tree_cancellation, configure_process_tree_suspended_spawn,
};

use super::ProcessPlayBackendInstallError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayProcessCommand {
    executable: PathBuf,
    working_directory: PathBuf,
    scene: RelPath,
    report_pipe: String,
}

impl PlayProcessCommand {
    pub(super) fn new(
        executable: impl Into<PathBuf>,
        working_directory: impl Into<PathBuf>,
        scene: RelPath,
        report_pipe: impl Into<String>,
    ) -> Self {
        Self {
            executable: executable.into(),
            working_directory: working_directory.into(),
            scene,
            report_pipe: report_pipe.into(),
        }
    }

    pub(super) fn configure(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command
            .args(self.arguments())
            .current_dir(&self.working_directory);
        configure_process_tree_cancellation(&mut command);
        configure_process_tree_suspended_spawn(&mut command);
        command
    }

    pub(super) fn arguments(&self) -> [&str; 8] {
        [
            "--project",
            ".",
            "--runtime-session-profile",
            "runtime",
            "--play-scene",
            self.scene.as_str(),
            "--play-report-pipe",
            self.report_pipe.as_str(),
        ]
    }

    pub(super) fn executable(&self) -> &Path {
        &self.executable
    }

    pub(super) fn report_pipe(&self) -> &str {
        &self.report_pipe
    }
}

#[cfg(test)]
#[path = "command/tests/borrowed_arguments_tests.rs"]
mod borrowed_arguments_tests;

pub(super) fn runtime_executable_next_to_current_process(
) -> Result<PathBuf, ProcessPlayBackendInstallError> {
    let current = std::env::current_exe()
        .map_err(|source| ProcessPlayBackendInstallError::CurrentEditorExecutable { source })?;
    runtime_executable_next_to_path(&current)
}

fn runtime_executable_next_to_path(
    current: &Path,
) -> Result<PathBuf, ProcessPlayBackendInstallError> {
    let directory = current
        .parent()
        .ok_or(ProcessPlayBackendInstallError::MissingEditorInstallDirectory)?;
    let product_directory = ProjectPaths::resolve_path(directory).map_err(|source| {
        ProcessPlayBackendInstallError::ResolveEditorInstallDirectory { source }
    })?;
    let runtime_name = format!("zircon_runtime{}", std::env::consts::EXE_SUFFIX);
    ProjectPaths::resolve_path_from(&product_directory, runtime_name)
        .map(|path| path.into_operation_path())
        .map_err(|source| ProcessPlayBackendInstallError::ResolveRuntimeExecutable { source })
}

#[cfg(test)]
#[path = "tests/command.rs"]
mod tests;
