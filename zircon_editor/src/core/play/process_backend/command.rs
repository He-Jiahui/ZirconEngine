use std::ffi::OsString;
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
            .arg("--project")
            .arg(".")
            .arg("--runtime-session-profile")
            .arg("runtime")
            .arg("--play-scene")
            .arg(self.scene.as_str())
            .arg("--play-report-pipe")
            .arg(&self.report_pipe)
            .current_dir(&self.working_directory);
        configure_process_tree_cancellation(&mut command);
        configure_process_tree_suspended_spawn(&mut command);
        command
    }

    pub(super) fn arguments(&self) -> Vec<OsString> {
        vec![
            "--project".into(),
            ".".into(),
            "--runtime-session-profile".into(),
            "runtime".into(),
            "--play-scene".into(),
            self.scene.as_str().into(),
            "--play-report-pipe".into(),
            self.report_pipe.clone().into(),
        ]
    }

    pub(super) fn executable(&self) -> &Path {
        &self.executable
    }

    pub(super) fn report_pipe(&self) -> &str {
        &self.report_pipe
    }
}

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
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::asset::project::ProjectPaths;

    use super::{runtime_executable_next_to_path, ProcessPlayBackendInstallError};

    #[test]
    fn runtime_executable_resolution_rejects_a_path_without_an_install_parent() {
        let error = runtime_executable_next_to_path(Path::new(""))
            .expect_err("an empty executable path cannot identify an installation directory");

        assert!(matches!(
            error,
            ProcessPlayBackendInstallError::MissingEditorInstallDirectory
        ));
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn play_runtime_executable_uses_the_physical_editor_install_identity() {
        let parent = unique_play_install_root("alias");
        let physical_install = parent.join("physical-install");
        fs::create_dir_all(&physical_install).unwrap();
        let install_alias = parent.join("install-alias");
        create_directory_link(&physical_install, &install_alias);

        let actual = runtime_executable_next_to_path(
            &install_alias.join(format!("zircon_editor{}", std::env::consts::EXE_SUFFIX)),
        )
        .expect("runtime executable should resolve from the editor installation");
        let expected = ProjectPaths::resolve_existing_path(&physical_install)
            .unwrap()
            .join(format!("zircon_runtime{}", std::env::consts::EXE_SUFFIX));

        fs::remove_dir_all(&parent).unwrap();
        assert_eq!(actual, expected);
    }

    fn unique_play_install_root(case_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zircon-editor-play-install-{case_name}-{}-{timestamp}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    #[cfg(unix)]
    fn create_directory_link(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).expect("create editor-install alias fixture");
    }

    #[cfg(windows)]
    fn create_directory_link(target: &Path, link: &Path) {
        let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
        let output = std::process::Command::new("cmd")
            .args(["/D", "/S", "/C"])
            .arg(command)
            .output()
            .expect("start mklink for editor-install alias fixture");
        assert!(
            output.status.success(),
            "create editor-install junction fixture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
