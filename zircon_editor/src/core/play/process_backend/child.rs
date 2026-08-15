use std::process::{ExitStatus, Stdio};

use crate::core::play::MaterializedPlayScene;
use crate::core::process::{terminate_process_tree, ProcessTreeLease};

use super::command::PlayProcessCommand;
use super::output::PlayOutputPump;

pub(super) struct PlayChild {
    child: std::process::Child,
    tree: Option<ProcessTreeLease>,
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
        let tree = ProcessTreeLease::attach_and_start(&child, "runtime preview process").map_err(
            |error| {
                let cleanup = terminate_untracked_spawn(&mut child, "runtime preview process");
                format!("{error}; {cleanup}")
            },
        )?;
        let Some(stdout) = child.stdout.take() else {
            return Err(
                terminate_and_reap(&mut child, tree, "runtime preview process").map_or_else(
                    |cleanup| format!("runtime preview did not expose piped stdout; {cleanup}"),
                    |_| "runtime preview did not expose piped stdout".to_string(),
                ),
            );
        };
        let Some(stderr) = child.stderr.take() else {
            return Err(
                terminate_and_reap(&mut child, tree, "runtime preview process").map_or_else(
                    |cleanup| format!("runtime preview did not expose piped stderr; {cleanup}"),
                    |_| "runtime preview did not expose piped stderr".to_string(),
                ),
            );
        };
        let output = match PlayOutputPump::capture(stdout, stderr) {
            Ok(output) => output,
            Err(error) => {
                return match terminate_and_reap(&mut child, tree, "runtime preview process") {
                    Ok(_) => Err(error.finish()),
                    Err(cleanup) => Err(format!(
                        "{}; output readers were not joined because process-tree termination failed: {cleanup}",
                        error.message()
                    )),
                };
            }
        };
        Ok(Self {
            child,
            tree: Some(tree),
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
        let mut diagnostics = match self.terminate_tree() {
            Ok(termination_diagnostic) => {
                let mut diagnostics = self
                    .output
                    .take()
                    .map_or_else(Vec::new, PlayOutputPump::finish);
                diagnostics.push(termination_diagnostic);
                diagnostics
            }
            Err(termination_diagnostic) => vec![format!(
                "{termination_diagnostic}; play output readers were not joined because persistent process-tree termination failed"
            )],
        };
        if let Err(error) = self.scene.cleanup() {
            diagnostics.push(error);
        }
        PlayChildOutcome {
            exit_code: status.code(),
            diagnostics,
        }
    }

    pub(super) fn stop(mut self) -> Result<PlayChildOutcome, String> {
        let termination_diagnostic = self.terminate_tree()?;
        let status = self
            .child
            .wait()
            .map_err(|error| format!("failed to reap runtime preview process: {error}"))?;
        Ok(self.finish_after_tree_termination(status, termination_diagnostic))
    }

    fn terminate_tree(&mut self) -> Result<String, String> {
        let tree = self.tree.take().ok_or_else(|| {
            "runtime preview persistent process-tree lease was already consumed".to_string()
        })?;
        let termination = tree.terminate("runtime preview process");
        if termination.succeeded {
            Ok(termination.diagnostic)
        } else {
            Err(termination.diagnostic)
        }
    }

    fn finish_after_tree_termination(
        mut self,
        status: ExitStatus,
        termination_diagnostic: String,
    ) -> PlayChildOutcome {
        let mut diagnostics = self
            .output
            .take()
            .map_or_else(Vec::new, PlayOutputPump::finish);
        diagnostics.push(termination_diagnostic);
        if let Err(error) = self.scene.cleanup() {
            diagnostics.push(error);
        }
        PlayChildOutcome {
            exit_code: status.code(),
            diagnostics,
        }
    }
}

fn terminate_and_reap(
    child: &mut std::process::Child,
    tree: ProcessTreeLease,
    label: &str,
) -> Result<(ExitStatus, String), String> {
    let termination = tree.terminate(label);
    if !termination.succeeded {
        return Err(termination.diagnostic);
    }
    let status = child
        .wait()
        .map_err(|error| format!("failed to reap {label}: {error}"))?;
    Ok((status, termination.diagnostic))
}

fn terminate_untracked_spawn(child: &mut std::process::Child, label: &str) -> String {
    let termination = terminate_process_tree(child, label);
    let reap = child
        .wait()
        .map(|_| "process reaped".to_string())
        .unwrap_or_else(|error| format!("failed to reap process: {error}"));
    format!("{}; {reap}", termination.diagnostic)
}

#[cfg(test)]
mod tests {
    #[test]
    fn play_child_captures_and_terminates_the_persistent_tree_before_joining_output() {
        let source = include_str!("child.rs");
        let attach = source
            .find("ProcessTreeLease::attach_and_start")
            .expect("spawn must attach the process tree");
        let output_capture = source
            .find("PlayOutputPump::capture")
            .expect("spawn must capture output");
        assert!(attach < output_capture);

        let finish = source
            .split("pub(super) fn finish")
            .nth(1)
            .and_then(|body| body.split("pub(super) fn stop").next())
            .expect("finish body should remain available");
        let terminate = finish
            .find("self.terminate_tree()")
            .expect("terminal finish must terminate the tree");
        let join = finish
            .find("PlayOutputPump::finish")
            .expect("terminal finish must join output readers");
        assert!(terminate < join);
    }
}
