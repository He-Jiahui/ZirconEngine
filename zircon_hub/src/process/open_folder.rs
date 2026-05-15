use std::path::PathBuf;
use std::process::{Child, Command};

use crate::error::HubError;

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
}

pub fn open_folder(command: &OpenFolderCommand) -> Result<Child, HubError> {
    Ok(Command::new(&command.program).args(&command.args).spawn()?)
}
