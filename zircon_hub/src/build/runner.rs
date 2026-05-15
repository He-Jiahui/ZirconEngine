use std::process::{Command, Stdio};

use crate::error::HubError;

use super::command::BuildCommand;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildExecutionReport {
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl BuildExecutionReport {
    pub fn succeeded(&self) -> bool {
        self.status_code == Some(0)
    }
}

pub fn run_build_command(command: &BuildCommand) -> Result<BuildExecutionReport, HubError> {
    let output = Command::new(&command.program)
        .args(&command.args)
        .current_dir(&command.cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    Ok(BuildExecutionReport {
        status_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
