use std::path::PathBuf;
use std::process::{Child, Command};

use crate::error::HubError;

/// 浏览输出或学习资源时传给平台文件管理器的命令投影，路径作为独立参数传递。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenFolderCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl OpenFolderCommand {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into().to_string_lossy().into_owned();
        if cfg!(target_os = "windows") {
            Self {
                program: "explorer".to_string(),
                args: vec![path],
            }
        } else if cfg!(target_os = "macos") {
            Self {
                program: "open".to_string(),
                args: vec![path],
            }
        } else {
            Self {
                program: "xdg-open".to_string(),
                args: vec![path],
            }
        }
    }

    pub fn command_line(&self) -> Vec<String> {
        std::iter::once(self.program.clone())
            .chain(self.args.iter().cloned())
            .collect()
    }
}

#[cfg(test)]
#[path = "tests/open_folder.rs"]
mod tests;

/// 从后台动作启动文件管理器并返回子进程句柄；成功启动不代表目标已被用户看到。
pub fn open_folder(command: &OpenFolderCommand) -> Result<Child, HubError> {
    Ok(Command::new(&command.program).args(&command.args).spawn()?)
}
