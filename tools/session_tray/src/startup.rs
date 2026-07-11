use std::path::Path;
use std::process::{Command, Output};

use crate::TrayError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupAction {
    Install,
    Update,
    Remove,
    Query,
}

impl StartupAction {
    fn argument(self) -> &'static str {
        match self {
            Self::Install => "Install",
            Self::Update => "Update",
            Self::Remove => "Remove",
            Self::Query => "Query",
        }
    }
}

pub fn manage(repo_root: &Path, action: StartupAction, dry_run: bool) -> Result<Output, TrayError> {
    let coordinator = run_coordinator_installer(repo_root, action, dry_run)?;
    if !coordinator.status.success() {
        return Err(TrayError::Http(
            "coordinator startup installer failed".into(),
        ));
    }
    let tray = run_tray_installer(repo_root, action, dry_run)?;
    if !tray.status.success() {
        return Err(TrayError::Http("tray startup installer failed".into()));
    }
    Ok(tray)
}

fn run_coordinator_installer(
    repo_root: &Path,
    action: StartupAction,
    dry_run: bool,
) -> Result<Output, TrayError> {
    let script = repo_root
        .join("tools")
        .join("install-session-coordinator-task.ps1");
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(script)
        .arg("-Action")
        .arg(action.argument())
        .arg("-RepoRoot")
        .arg(repo_root);
    if dry_run {
        command.arg("-DryRun");
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    Ok(command.output()?)
}

fn run_tray_installer(
    repo_root: &Path,
    action: StartupAction,
    dry_run: bool,
) -> Result<Output, TrayError> {
    let script = repo_root
        .join("tools")
        .join("install-session-tray-startup.ps1");
    let executable = std::env::current_exe()?;
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(script)
        .arg("-Action")
        .arg(action.argument())
        .arg("-TrayExecutable")
        .arg(executable)
        .arg("-RepoRoot")
        .arg(repo_root);
    if dry_run {
        command.arg("-DryRun");
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    Ok(command.output()?)
}
