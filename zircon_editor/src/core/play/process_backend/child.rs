use std::mem;
use std::process::{ExitStatus, Stdio};

use crate::core::play::MaterializedPlayScene;
use crate::core::process::{terminate_process_tree, ProcessTreeLease, ProcessTreeTerminationError};

use super::command::PlayProcessCommand;
use super::output::{PlayOutputCaptureError, PlayOutputPump};

enum PlayProcessTree {
    Persistent(ProcessTreeLease),
    Untracked,
    Terminated,
}

enum PlayChildOutput {
    Captured(PlayOutputPump),
    CaptureFailed(PlayOutputCaptureError),
}

impl PlayChildOutput {
    fn drain(&self) -> Vec<String> {
        match self {
            Self::Captured(output) => output.drain(),
            Self::CaptureFailed(_) => Vec::new(),
        }
    }

    fn finish(self) -> Vec<String> {
        match self {
            Self::Captured(output) => output.finish(),
            Self::CaptureFailed(error) => vec![error.finish()],
        }
    }
}

pub(super) struct PlayChild {
    child: std::process::Child,
    tree: PlayProcessTree,
    terminal_status: Option<ExitStatus>,
    terminal_diagnostics: Vec<String>,
    output: Option<PlayChildOutput>,
    scene: MaterializedPlayScene,
}

pub(super) enum PlayProcessCleanup {
    Child(PlayChild),
    Scene(MaterializedPlayScene),
}

impl PlayProcessCleanup {
    pub(super) fn retry(self) -> Result<Vec<String>, (Self, String)> {
        match self {
            Self::Child(child) => {
                child
                    .stop()
                    .map(|outcome| outcome.diagnostics)
                    .map_err(|failure| {
                        let (child, message) = failure.into_parts();
                        (Self::Child(child), message)
                    })
            }
            Self::Scene(mut scene) => scene
                .cleanup()
                .map(|()| Vec::new())
                .map_err(|message| (Self::Scene(scene), message)),
        }
    }

    pub(super) fn cleanup_on_drop(self) {
        match self {
            Self::Child(child) => child.cleanup_on_drop(),
            Self::Scene(mut scene) => {
                if let Err(error) = scene.cleanup() {
                    tracing::error!(
                        %error,
                        "runtime preview snapshot could not be cleaned during backend drop"
                    );
                }
            }
        }
    }
}

pub(super) struct PlayChildStartFailure {
    cleanup: Option<PlayProcessCleanup>,
    message: String,
}

impl PlayChildStartFailure {
    pub(super) fn into_parts(self) -> (Option<PlayProcessCleanup>, String) {
        (self.cleanup, self.message)
    }

    fn before_spawn(mut scene: MaterializedPlayScene, message: String) -> Self {
        match scene.cleanup() {
            Ok(()) => Self {
                cleanup: None,
                message,
            },
            Err(cleanup_error) => Self {
                cleanup: Some(PlayProcessCleanup::Scene(scene)),
                message: format!(
                    "{message}; play snapshot cleanup remains pending: {cleanup_error}"
                ),
            },
        }
    }

    fn after_spawn(child: PlayChild, message: String) -> Self {
        match child.stop() {
            Ok(_) => Self {
                cleanup: None,
                message,
            },
            Err(failure) => {
                let (child, cleanup_error) = failure.into_parts();
                Self {
                    cleanup: Some(PlayProcessCleanup::Child(child)),
                    message: format!("{message}; cleanup remains pending: {cleanup_error}"),
                }
            }
        }
    }
}

pub(super) struct PlayChildOutcome {
    pub(super) exit_code: Option<i32>,
    pub(super) diagnostics: Vec<String>,
}

pub(super) struct PlayChildStopFailure {
    child: PlayChild,
    message: String,
}

impl PlayChildStopFailure {
    pub(super) fn into_parts(self) -> (PlayChild, String) {
        (self.child, self.message)
    }
}

impl PlayChild {
    pub(super) fn spawn(
        command: &PlayProcessCommand,
        scene: MaterializedPlayScene,
    ) -> Result<Self, PlayChildStartFailure> {
        let mut process = command.configure();
        process
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("ZIRCON_PROCESS_OUTPUT_ENCODING", "utf-8");
        let mut child = match process.spawn() {
            Ok(child) => child,
            Err(error) => {
                return Err(PlayChildStartFailure::before_spawn(
                    scene,
                    format!(
                        "failed to spawn runtime preview {}: {error}",
                        command.executable().display()
                    ),
                ));
            }
        };
        let tree = match ProcessTreeLease::attach_and_start(&child, "runtime preview process") {
            Ok(tree) => tree,
            Err(error) => {
                return Err(PlayChildStartFailure::after_spawn(
                    Self::with_tree(child, PlayProcessTree::Untracked, None, scene),
                    error.to_string(),
                ));
            }
        };
        let Some(stdout) = child.stdout.take() else {
            return Err(PlayChildStartFailure::after_spawn(
                Self::with_tree(child, PlayProcessTree::Persistent(tree), None, scene),
                "runtime preview did not expose piped stdout".to_string(),
            ));
        };
        let Some(stderr) = child.stderr.take() else {
            drop(stdout);
            return Err(PlayChildStartFailure::after_spawn(
                Self::with_tree(child, PlayProcessTree::Persistent(tree), None, scene),
                "runtime preview did not expose piped stderr".to_string(),
            ));
        };
        match PlayOutputPump::capture(stdout, stderr) {
            Ok(output) => Ok(Self::with_tree(
                child,
                PlayProcessTree::Persistent(tree),
                Some(PlayChildOutput::Captured(output)),
                scene,
            )),
            Err(error) => {
                let message = error.message().to_string();
                Err(PlayChildStartFailure::after_spawn(
                    Self::with_tree(
                        child,
                        PlayProcessTree::Persistent(tree),
                        Some(PlayChildOutput::CaptureFailed(error)),
                        scene,
                    ),
                    message,
                ))
            }
        }
    }

    fn with_tree(
        child: std::process::Child,
        tree: PlayProcessTree,
        output: Option<PlayChildOutput>,
        scene: MaterializedPlayScene,
    ) -> Self {
        Self {
            child,
            tree,
            terminal_status: None,
            terminal_diagnostics: Vec::new(),
            output,
            scene,
        }
    }

    pub(super) fn id(&self) -> u32 {
        self.child.id()
    }

    pub(super) fn try_wait(&mut self) -> Result<Option<ExitStatus>, String> {
        if let Some(status) = self.terminal_status {
            return Ok(Some(status));
        }
        self.child
            .try_wait()
            .map_err(|error| format!("failed to poll runtime preview process: {error}"))
    }

    pub(super) fn drain_output(&self) -> Vec<String> {
        self.output
            .as_ref()
            .map_or_else(Vec::new, PlayChildOutput::drain)
    }

    pub(super) fn finish(
        mut self,
        status: ExitStatus,
    ) -> Result<PlayChildOutcome, PlayChildStopFailure> {
        self.terminal_status = Some(status);
        if let Err(message) = self.terminate_tree() {
            return Err(PlayChildStopFailure {
                child: self,
                message: format!("{message}; play output readers remain owned for retry"),
            });
        }
        self.finish_terminal()
    }

    pub(super) fn stop(self) -> Result<PlayChildOutcome, PlayChildStopFailure> {
        let mut child = self;
        if let Err(message) = child.terminate_tree() {
            return Err(PlayChildStopFailure { child, message });
        }
        if child.terminal_status.is_none() {
            let status = match child.child.wait() {
                Ok(status) => status,
                Err(error) => {
                    return Err(PlayChildStopFailure {
                        child,
                        message: format!("failed to reap runtime preview process: {error}"),
                    });
                }
            };
            child.terminal_status = Some(status);
        }
        child.finish_terminal()
    }

    fn terminate_tree(&mut self) -> Result<(), String> {
        match &mut self.tree {
            PlayProcessTree::Terminated => Ok(()),
            PlayProcessTree::Persistent(tree) => {
                let termination = tree.terminate("runtime preview process");
                if !termination.succeeded {
                    return Err(termination.diagnostic);
                }
                self.terminal_diagnostics.push(termination.diagnostic);
                self.tree = PlayProcessTree::Terminated;
                Ok(())
            }
            PlayProcessTree::Untracked => {
                if let Some(status) = self
                    .child
                    .try_wait()
                    .map_err(|error| format!("failed to poll untracked runtime preview: {error}"))?
                {
                    self.terminal_status = Some(status);
                    self.tree = PlayProcessTree::Terminated;
                    self.terminal_diagnostics.push(
                        "untracked runtime preview had already exited before cleanup".to_string(),
                    );
                    return Ok(());
                }
                let termination =
                    terminate_process_tree(&mut self.child, "runtime preview process");
                if termination.succeeded {
                    self.terminal_diagnostics.push(termination.diagnostic);
                    self.tree = PlayProcessTree::Terminated;
                    return Ok(());
                }
                Err(termination.diagnostic)
            }
        }
    }

    fn finish_terminal(mut self) -> Result<PlayChildOutcome, PlayChildStopFailure> {
        if let Some(output) = self.output.take() {
            self.terminal_diagnostics.extend(output.finish());
        }
        if let Err(error) = self.scene.cleanup() {
            return Err(PlayChildStopFailure {
                child: self,
                message: format!("play snapshot cleanup remains pending: {error}"),
            });
        }
        let status = self
            .terminal_status
            .take()
            .expect("terminal Play cleanup requires a reaped process status");
        Ok(PlayChildOutcome {
            exit_code: status.code(),
            diagnostics: mem::take(&mut self.terminal_diagnostics),
        })
    }

    pub(super) fn cleanup_on_drop(mut self) {
        let termination = self.terminate_tree();
        let mut root_reap_expected = termination.is_ok();
        let mut inherited_pipes_closed = matches!(self.tree, PlayProcessTree::Terminated);
        if let Err(error) = termination {
            let fallback = terminate_process_tree(&mut self.child, "runtime preview process");
            root_reap_expected = fallback.succeeded
                || matches!(
                    fallback.error.as_ref(),
                    Some(ProcessTreeTerminationError::TreeTerminationIncomplete { .. })
                );
            inherited_pipes_closed = fallback.succeeded;
            tracing::error!(
                %error,
                fallback = %fallback.diagnostic,
                "runtime preview process tree could not be retired during backend drop"
            );
        }

        if self.terminal_status.is_none() && root_reap_expected {
            match self.child.wait() {
                Ok(status) => self.terminal_status = Some(status),
                Err(error) => {
                    tracing::error!(%error, "runtime preview process could not be reaped during backend drop");
                }
            }
        } else if self.terminal_status.is_none() {
            match self.child.try_wait() {
                Ok(Some(status)) => self.terminal_status = Some(status),
                Ok(None) => tracing::error!(
                    "runtime preview process remains active after all backend-drop termination attempts"
                ),
                Err(error) => tracing::error!(
                    %error,
                    "runtime preview process state could not be observed during backend drop"
                ),
            }
        }
        if inherited_pipes_closed {
            if let Some(output) = self.output.take() {
                let _ = output.finish();
            }
        } else {
            // A failed tree termination cannot prove that inherited pipes are closed.
            // Drop the readers without blocking backend teardown indefinitely.
            drop(self.output.take());
        }
        if let Err(error) = self.scene.cleanup() {
            tracing::error!(%error, "runtime preview snapshot could not be cleaned during backend drop");
        }
    }
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
        let finalize = finish
            .find("self.finish_terminal()")
            .expect("terminal finish must finalize owned output and snapshot state");
        assert!(terminate < finalize);
    }

    #[test]
    fn terminal_cleanup_retry_preserves_the_terminated_tree_phase() {
        let source = include_str!("child.rs");
        let terminate = source
            .split("fn terminate_tree")
            .nth(1)
            .and_then(|body| body.split("fn finish_terminal").next())
            .expect("play child tree termination implementation");
        let finalize = source
            .split("fn finish_terminal")
            .nth(1)
            .and_then(|body| body.split("pub(super) fn cleanup_on_drop").next())
            .expect("play child terminal finalization implementation");

        assert!(terminate.contains("PlayProcessTree::Terminated => Ok(())"));
        assert!(terminate.contains("self.tree = PlayProcessTree::Terminated"));
        assert!(!finalize.contains("self.tree"));
        assert!(finalize.contains("play snapshot cleanup remains pending"));
    }

    #[test]
    fn start_failure_retains_every_unfinished_cleanup_owner() {
        let source = include_str!("child.rs");

        assert!(source.contains("enum PlayProcessCleanup"));
        assert!(source.contains("CaptureFailed(PlayOutputCaptureError)"));
        assert!(source.contains("cleanup: Some(PlayProcessCleanup::Child(child))"));
        assert!(source.contains("cleanup: Some(PlayProcessCleanup::Scene(scene))"));
        assert!(!source.contains("terminate_untracked_spawn"));
        assert!(!source.contains("terminate_and_reap"));
    }
}
