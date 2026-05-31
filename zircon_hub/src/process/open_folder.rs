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

    pub fn command_line(&self) -> Vec<String> {
        std::iter::once(self.program.clone())
            .chain(self.args.iter().cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_folder_command_line_preserves_program_and_path() {
        let command = OpenFolderCommand {
            program: "xdg-open".to_string(),
            args: vec!["/tmp/Zircon Output".to_string()],
        };

        assert_eq!(
            command.command_line(),
            vec!["xdg-open".to_string(), "/tmp/Zircon Output".to_string()]
        );
    }
}

pub fn open_folder(command: &OpenFolderCommand) -> Result<Child, HubError> {
    Ok(Command::new(&command.program).args(&command.args).spawn()?)
}
