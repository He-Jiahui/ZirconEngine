use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayProcessCommand {
    executable: PathBuf,
    project_root: PathBuf,
    scene_path: PathBuf,
    report_pipe: String,
}

impl PlayProcessCommand {
    pub(super) fn new(
        executable: impl Into<PathBuf>,
        project_root: impl Into<PathBuf>,
        scene_path: impl Into<PathBuf>,
        report_pipe: impl Into<String>,
    ) -> Self {
        Self {
            executable: executable.into(),
            project_root: project_root.into(),
            scene_path: scene_path.into(),
            report_pipe: report_pipe.into(),
        }
    }

    pub(super) fn configure(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command
            .arg("--project")
            .arg(&self.project_root)
            .arg("--runtime-session-profile")
            .arg("runtime")
            .arg("--play-scene")
            .arg(&self.scene_path)
            .arg("--play-report-pipe")
            .arg(&self.report_pipe)
            .current_dir(&self.project_root);
        command
    }

    pub(super) fn arguments(&self) -> Vec<OsString> {
        vec![
            "--project".into(),
            self.project_root.as_os_str().to_owned(),
            "--runtime-session-profile".into(),
            "runtime".into(),
            "--play-scene".into(),
            self.scene_path.as_os_str().to_owned(),
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

pub(super) fn runtime_executable_next_to_current_process() -> Result<PathBuf, String> {
    let current = std::env::current_exe()
        .map_err(|error| format!("failed to resolve current editor executable: {error}"))?;
    let file_name = format!("zircon_runtime{}", std::env::consts::EXE_SUFFIX);
    Ok(current.with_file_name(file_name))
}
