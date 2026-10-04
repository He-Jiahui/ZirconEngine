use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::HubError;

/// 项目删除动作的 Windows 回收站命令投影，供执行路径与安全转义测试共用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecycleDeleteCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl RecycleDeleteCommand {
    pub fn for_project(path: impl AsRef<Path>) -> Result<Self, HubError> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(HubError::message("Project path is required"));
        }
        if !cfg!(target_os = "windows") {
            return Err(HubError::message(
                "Project deletion is only available on Windows in this Hub build",
            ));
        }
        Ok(Self::windows_delete_directory(path))
    }

    fn windows_delete_directory(path: &Path) -> Self {
        let escaped = path.to_string_lossy().replace('\'', "''");
        let script = format!(
            "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory('{escaped}', 'OnlyErrorDialogs', 'SendToRecycleBin')"
        );
        Self {
            program: "powershell".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-STA".to_string(),
                "-Command".to_string(),
                script,
            ],
        }
    }
}

/// 将项目交给系统回收站；调用方应先确认项目身份并在成功后同步近期项目登记。
/// 命令退出成功仅表示系统接受删除动作，恢复能力由系统回收站决定。
pub fn recycle_delete_project(path: impl Into<PathBuf>) -> Result<(), HubError> {
    let command = RecycleDeleteCommand::for_project(path.into())?;
    let output = Command::new(&command.program)
        .args(&command.args)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!(
            "Recycle Bin deletion failed with status {}",
            output
                .status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        )
    };
    Err(HubError::message(detail))
}

#[cfg(test)]
#[path = "tests/recycle_bin.rs"]
mod tests;
