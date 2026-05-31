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

    pub fn summary_line(&self) -> String {
        self.stderr
            .lines()
            .rev()
            .chain(self.stdout.lines().rev())
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("build produced no output")
            .to_string()
    }

    pub fn recovery_hint(&self) -> String {
        match self.status_code {
            Some(code) => format!(
                "tools/zircon_build.py exited with code {code}; open Build History and fix the first reported error before retrying"
            ),
            None => "tools/zircon_build.py was terminated by the OS; check the build log and retry from Hub".to_string(),
        }
    }

    pub fn log_excerpt(&self) -> String {
        log_excerpt_from_streams(&self.stdout, &self.stderr)
    }
}

fn log_excerpt_from_streams(stdout: &str, stderr: &str) -> String {
    stderr
        .lines()
        .chain(stdout.lines())
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_report_summary_uses_last_non_empty_diagnostic() {
        let report = BuildExecutionReport {
            status_code: Some(101),
            stdout: "Compiling zircon_hub\n".to_string(),
            stderr: "error: failed to compile\n\n".to_string(),
        };

        assert_eq!(report.summary_line(), "error: failed to compile");
        assert_eq!(
            report.recovery_hint(),
            "tools/zircon_build.py exited with code 101; open Build History and fix the first reported error before retrying"
        );
        assert_eq!(
            report.log_excerpt(),
            "error: failed to compile\nCompiling zircon_hub"
        );
    }
}
