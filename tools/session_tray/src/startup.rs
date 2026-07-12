use std::path::Path;
use std::process::{Command, Output};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::TrayError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupAction {
    Install,
    Update,
    Remove,
    Query,
}

impl StartupAction {
    pub fn argument(self) -> &'static str {
        match self {
            Self::Install => "Install",
            Self::Update => "Update",
            Self::Remove => "Remove",
            Self::Query => "Query",
        }
    }
}

#[derive(Clone, Debug)]
pub struct StartupPreview {
    pub action: StartupAction,
    state_fingerprint: String,
}

impl StartupPreview {
    pub fn new(
        action: StartupAction,
        current: &StartupManagementResult,
    ) -> Result<Self, TrayError> {
        Ok(Self {
            action,
            state_fingerprint: startup_state_fingerprint(current)?,
        })
    }

    pub fn matches(
        &self,
        action: StartupAction,
        current: &StartupManagementResult,
    ) -> Result<bool, TrayError> {
        Ok(self.action == action && self.state_fingerprint == startup_state_fingerprint(current)?)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupComponentResult {
    pub component: &'static str,
    pub attempted: bool,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupManagementResult {
    pub action: &'static str,
    pub coordinator: StartupComponentResult,
    pub tray: StartupComponentResult,
}

impl StartupManagementResult {
    pub fn success(&self) -> bool {
        self.coordinator.success && self.tray.success
    }

    pub fn summary(&self) -> String {
        format!(
            "协调服务：{}；托盘：{}",
            component_summary(&self.coordinator),
            component_summary(&self.tray)
        )
    }
}

pub fn manage(
    repo_root: &Path,
    action: StartupAction,
    dry_run: bool,
) -> Result<StartupManagementResult, TrayError> {
    let coordinator = component_result(
        "coordinator",
        run_coordinator_installer(repo_root, action, dry_run)?,
    );
    let tray = if coordinator.success || action == StartupAction::Query {
        component_result("tray", run_tray_installer(repo_root, action, dry_run)?)
    } else {
        StartupComponentResult {
            component: "tray",
            attempted: false,
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: "skipped because coordinator startup management failed".into(),
        }
    };
    Ok(StartupManagementResult {
        action: action.argument(),
        coordinator,
        tray,
    })
}

pub fn preview(repo_root: &Path, action: StartupAction) -> Result<StartupPreview, TrayError> {
    let current = manage(repo_root, StartupAction::Query, false)?;
    if !current.success() {
        return Err(TrayError::Http(
            "startup state query was incomplete; mutation preview was not created".into(),
        ));
    }
    StartupPreview::new(action, &current)
}

pub fn execute_preview(
    repo_root: &Path,
    preview: &StartupPreview,
) -> Result<StartupManagementResult, TrayError> {
    let current = manage(repo_root, StartupAction::Query, false)?;
    if !current.success() {
        return Err(TrayError::Http(
            "startup state query was incomplete; mutation was not executed".into(),
        ));
    }
    if !preview.matches(preview.action, &current)? {
        return Err(TrayError::Http(
            "startup state changed after preview; create a new preview".into(),
        ));
    }
    manage(repo_root, preview.action, false)
}

fn startup_state_fingerprint(current: &StartupManagementResult) -> Result<String, TrayError> {
    let bytes = serde_json::to_vec(current)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn component_result(component: &'static str, output: Output) -> StartupComponentResult {
    StartupComponentResult {
        component,
        attempted: true,
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn component_summary(result: &StartupComponentResult) -> String {
    if !result.attempted {
        return "未执行".into();
    }
    let state = if result.success { "成功" } else { "失败" };
    let detail = if result.stdout.trim().is_empty() {
        result.stderr.trim()
    } else {
        result.stdout.trim()
    };
    if detail.is_empty() {
        state.into()
    } else {
        let bounded = detail
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(160)
            .collect::<String>();
        format!("{state}：{bounded}")
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divergent_component_states_remain_visible_in_summary_and_json() {
        let result = StartupManagementResult {
            action: "Query",
            coordinator: StartupComponentResult {
                component: "coordinator",
                attempted: true,
                success: true,
                exit_code: Some(0),
                stdout: r#"{"installed":false}"#.into(),
                stderr: String::new(),
            },
            tray: StartupComponentResult {
                component: "tray",
                attempted: true,
                success: true,
                exit_code: Some(0),
                stdout: r#"{"installed":true}"#.into(),
                stderr: String::new(),
            },
        };

        assert!(result.success());
        assert!(result.summary().contains(r#"{"installed":false}"#));
        assert!(result.summary().contains(r#"{"installed":true}"#));
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(true, json["coordinator"]["success"]);
        assert_eq!(true, json["tray"]["success"]);
    }

    #[test]
    fn mutating_preview_rejects_changed_startup_state() {
        let original = StartupManagementResult {
            action: "Query",
            coordinator: StartupComponentResult {
                component: "coordinator",
                attempted: true,
                success: true,
                exit_code: Some(0),
                stdout: r#"{"installed":true,"backend":"UserStartup"}"#.into(),
                stderr: String::new(),
            },
            tray: StartupComponentResult {
                component: "tray",
                attempted: true,
                success: true,
                exit_code: Some(0),
                stdout: r#"{"installed":true}"#.into(),
                stderr: String::new(),
            },
        };
        let preview = StartupPreview::new(StartupAction::Remove, &original).unwrap();

        assert!(preview.matches(StartupAction::Remove, &original).unwrap());

        let mut changed = original.clone();
        changed.tray.stdout = r#"{"installed":false}"#.into();
        assert!(!preview.matches(StartupAction::Remove, &changed).unwrap());
        assert!(!preview.matches(StartupAction::Update, &original).unwrap());
    }
}
