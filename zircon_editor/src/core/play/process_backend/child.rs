use std::process::{ExitStatus, Stdio};

use crate::core::play::MaterializedPlayScene;

use super::command::PlayProcessCommand;
use super::output::PlayOutputPump;

pub(super) struct PlayChild {
    child: std::process::Child,
    output: Option<PlayOutputPump>,
    scene: MaterializedPlayScene,
}

pub(super) struct PlayChildOutcome {
    pub(super) exit_code: Option<i32>,
    pub(super) diagnostics: Vec<String>,
}

impl PlayChild {
    pub(super) fn spawn(
        command: &PlayProcessCommand,
        scene: MaterializedPlayScene,
    ) -> Result<Self, String> {
        let mut process = command.configure();
        process
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("ZIRCON_PROCESS_OUTPUT_ENCODING", "utf-8");
        let mut child = process.spawn().map_err(|error| {
            format!(
                "failed to spawn runtime preview {}: {error}",
                command.executable().display()
            )
        })?;
        let Some(stdout) = child.stdout.take() else {
            terminate_and_reap(&mut child);
            return Err("runtime preview did not expose piped stdout".to_string());
        };
        let Some(stderr) = child.stderr.take() else {
            terminate_and_reap(&mut child);
            return Err("runtime preview did not expose piped stderr".to_string());
        };
        let output = match PlayOutputPump::capture(stdout, stderr) {
            Ok(output) => output,
            Err(error) => {
                terminate_and_reap(&mut child);
                return Err(error);
            }
        };
        Ok(Self {
            child,
            output: Some(output),
            scene,
        })
    }

    pub(super) fn id(&self) -> u32 {
        self.child.id()
    }

    pub(super) fn try_wait(&mut self) -> Result<Option<ExitStatus>, String> {
        self.child
            .try_wait()
            .map_err(|error| format!("failed to poll runtime preview process: {error}"))
    }

    pub(super) fn drain_output(&self) -> Vec<String> {
        self.output
            .as_ref()
            .map_or_else(Vec::new, PlayOutputPump::drain)
    }

    pub(super) fn finish(mut self, status: ExitStatus) -> PlayChildOutcome {
        let mut diagnostics = self
            .output
            .take()
            .map_or_else(Vec::new, PlayOutputPump::finish);
        if let Err(error) = self.scene.cleanup() {
            diagnostics.push(error);
        }
        PlayChildOutcome {
            exit_code: status.code(),
            diagnostics,
        }
    }

    pub(super) fn stop(mut self) -> Result<PlayChildOutcome, String> {
        let status = match self.try_wait()? {
            Some(status) => status,
            None => {
                if let Err(kill_error) = self.child.kill() {
                    if self.try_wait()?.is_none() {
                        return Err(format!(
                            "failed to stop runtime preview process: {kill_error}"
                        ));
                    }
                }
                self.child
                    .wait()
                    .map_err(|error| format!("failed to reap runtime preview process: {error}"))?
            }
        };
        Ok(self.finish(status))
    }
}

fn terminate_and_reap(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}
