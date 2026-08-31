use std::mem;
use std::path::PathBuf;
use std::sync::Mutex;

use super::{
    PlayBackend, PlayBackendPoll, PlayBackendRetireReport, PlayBackendStartFailure,
    PlayBackendStartReport, PlayBackendStopReport, PlaySnapshotStore, PlayStartRequest,
};

mod child;
mod command;
mod error;
mod output;
#[cfg(test)]
mod tests;

use child::{PlayChild, PlayProcessCleanup};
use command::{runtime_executable_next_to_current_process, PlayProcessCommand};

pub use error::ProcessPlayBackendInstallError;

enum ActivePlayProcess {
    Idle,
    Running(PlayChild),
    CleanupPending(PlayProcessCleanup),
    Stopping,
    Finalizing,
}

pub struct ProcessPlayBackend {
    executable: PathBuf,
    snapshots: PlaySnapshotStore,
    active: Mutex<ActivePlayProcess>,
}

impl ProcessPlayBackend {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            snapshots: PlaySnapshotStore::default(),
            active: Mutex::new(ActivePlayProcess::Idle),
        }
    }

    pub fn for_current_install() -> Result<Self, ProcessPlayBackendInstallError> {
        runtime_executable_next_to_current_process().map(Self::new)
    }

    pub fn is_active(&self) -> bool {
        self.active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_active()
    }
}

impl ActivePlayProcess {
    fn is_active(&self) -> bool {
        !matches!(self, Self::Idle)
    }
}

impl PlayBackend for ProcessPlayBackend {
    fn start(
        &self,
        request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &*active {
            ActivePlayProcess::Idle => {}
            ActivePlayProcess::Running(_) => {
                return Err(PlayBackendStartFailure::new(
                    "runtime preview process is already active",
                ));
            }
            ActivePlayProcess::CleanupPending(_) => {
                return Err(PlayBackendStartFailure::retirement_pending(
                    "runtime preview process cleanup is pending",
                ));
            }
            ActivePlayProcess::Stopping => {
                return Err(PlayBackendStartFailure::new(
                    "runtime preview process is stopping",
                ));
            }
            ActivePlayProcess::Finalizing => {
                return Err(PlayBackendStartFailure::new(
                    "runtime preview process is finalizing terminal output",
                ));
            }
        }
        let project_root = request.project_root.as_deref().ok_or_else(|| {
            PlayBackendStartFailure::new("process play requires an open project root")
        })?;
        let source = request.scene_source.as_ref().ok_or_else(|| {
            PlayBackendStartFailure::new(
                "process play requires a persisted scene or world snapshot",
            )
        })?;
        let scene = match self.snapshots.materialize(project_root, source) {
            Ok(scene) => scene,
            Err(failure) => {
                let (cleanup_owner, message) = failure.into_parts();
                if let Some(scene) = cleanup_owner {
                    *active = ActivePlayProcess::CleanupPending(PlayProcessCleanup::Scene(scene));
                    return Err(PlayBackendStartFailure::retirement_pending(message));
                }
                *active = ActivePlayProcess::Idle;
                return Err(PlayBackendStartFailure::new(message));
            }
        };
        let report_pipe = format!("zircon-play-report-{}", scene.instance_id());
        let command = PlayProcessCommand::new(
            &self.executable,
            project_root,
            scene.relative_path().clone(),
            &report_pipe,
        );
        let arguments = command
            .arguments()
            .into_iter()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");
        let child = match PlayChild::spawn(&command, scene) {
            Ok(child) => child,
            Err(failure) => {
                let (cleanup, message) = failure.into_parts();
                if let Some(cleanup) = cleanup {
                    *active = ActivePlayProcess::CleanupPending(cleanup);
                    return Err(PlayBackendStartFailure::retirement_pending(message));
                }
                *active = ActivePlayProcess::Idle;
                return Err(PlayBackendStartFailure::new(message));
            }
        };
        let pid = child.id();
        *active = ActivePlayProcess::Running(child);
        Ok(PlayBackendStartReport::default().with_diagnostics(vec![
            format!("process.pid={pid}"),
            format!("process.report_pipe={}", command.report_pipe()),
            format!("process.args={arguments}"),
        ]))
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        let child = {
            let mut active = self
                .active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match mem::replace(&mut *active, ActivePlayProcess::Stopping) {
                ActivePlayProcess::Idle => {
                    *active = ActivePlayProcess::Idle;
                    return Ok(PlayBackendStopReport::default());
                }
                ActivePlayProcess::Running(child) => child,
                ActivePlayProcess::CleanupPending(cleanup) => {
                    *active = ActivePlayProcess::CleanupPending(cleanup);
                    return Err("runtime preview process cleanup is pending".to_string());
                }
                ActivePlayProcess::Stopping => {
                    *active = ActivePlayProcess::Stopping;
                    return Err("runtime preview process stop is already in progress".to_string());
                }
                ActivePlayProcess::Finalizing => {
                    *active = ActivePlayProcess::Finalizing;
                    return Err(
                        "runtime preview process terminal cleanup is already in progress"
                            .to_string(),
                    );
                }
            }
        };
        match child.stop() {
            Ok(outcome) => {
                *self
                    .active
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = ActivePlayProcess::Idle;
                Ok(PlayBackendStopReport {
                    diagnostics: outcome.diagnostics,
                    retirement_pending: false,
                })
            }
            Err(failure) => {
                let (child, error) = failure.into_parts();
                *self
                    .active
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                    ActivePlayProcess::Running(child);
                Err(error)
            }
        }
    }

    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        let cleanup = {
            let mut active = self
                .active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match mem::replace(&mut *active, ActivePlayProcess::Finalizing) {
                ActivePlayProcess::CleanupPending(cleanup) => cleanup,
                ActivePlayProcess::Idle => {
                    *active = ActivePlayProcess::Idle;
                    return Ok(PlayBackendRetireReport::default());
                }
                ActivePlayProcess::Running(child) => {
                    *active = ActivePlayProcess::Running(child);
                    return Err("runtime preview process must stop before retirement".to_string());
                }
                ActivePlayProcess::Stopping => {
                    *active = ActivePlayProcess::Stopping;
                    return Err("runtime preview process stop is already in progress".to_string());
                }
                ActivePlayProcess::Finalizing => {
                    *active = ActivePlayProcess::Finalizing;
                    return Err(
                        "runtime preview process terminal cleanup is already in progress"
                            .to_string(),
                    );
                }
            }
        };
        match cleanup.retry() {
            Ok(diagnostics) => {
                *self
                    .active
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = ActivePlayProcess::Idle;
                Ok(PlayBackendRetireReport { diagnostics })
            }
            Err((cleanup, message)) => {
                *self
                    .active
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                    ActivePlayProcess::CleanupPending(cleanup);
                Err(message)
            }
        }
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let status = match &mut *active {
            ActivePlayProcess::Idle => {
                return Err(
                    "runtime preview process is not active while Play is running".to_string(),
                );
            }
            ActivePlayProcess::Stopping => {
                return Err("runtime preview process stop is already in progress".to_string());
            }
            ActivePlayProcess::CleanupPending(_) => {
                return Err("runtime preview process cleanup is pending".to_string());
            }
            ActivePlayProcess::Finalizing => {
                return Err(
                    "runtime preview process terminal cleanup is already in progress".to_string(),
                );
            }
            ActivePlayProcess::Running(child) => child.try_wait()?,
        };
        let Some(status) = status else {
            let ActivePlayProcess::Running(child) = &*active else {
                unreachable!("active Play child disappeared during poll");
            };
            return Ok(PlayBackendPoll::Running {
                diagnostics: child.drain_output(),
            });
        };
        let child = match mem::replace(&mut *active, ActivePlayProcess::Finalizing) {
            ActivePlayProcess::Running(child) => child,
            _ => unreachable!("active Play child disappeared during terminal poll"),
        };
        drop(active);
        match child.finish(status) {
            Ok(outcome) => {
                *self
                    .active
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = ActivePlayProcess::Idle;
                Ok(PlayBackendPoll::Exited {
                    exit_code: outcome.exit_code,
                    diagnostics: outcome.diagnostics,
                })
            }
            Err(failure) => {
                let (child, error) = failure.into_parts();
                *self
                    .active
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                    ActivePlayProcess::Running(child);
                Err(error)
            }
        }
    }
}

impl Drop for ProcessPlayBackend {
    fn drop(&mut self) {
        let active = self
            .active
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let child = match mem::replace(active, ActivePlayProcess::Idle) {
            ActivePlayProcess::Running(child) => Some(PlayProcessCleanup::Child(child)),
            ActivePlayProcess::CleanupPending(cleanup) => Some(cleanup),
            ActivePlayProcess::Idle
            | ActivePlayProcess::Stopping
            | ActivePlayProcess::Finalizing => None,
        };
        if let Some(cleanup) = child {
            match cleanup.retry() {
                Ok(_) => {}
                Err((cleanup, error)) => {
                    tracing::error!(%error, "runtime preview cleanup failed during backend drop");
                    cleanup.cleanup_on_drop();
                }
            }
        }
    }
}

#[cfg(test)]
mod performance_source_guards {
    #[test]
    fn terminal_poll_releases_the_active_lock_before_join_and_cleanup() {
        let source = include_str!("mod.rs");
        let body = source
            .split("fn poll(&self)")
            .nth(1)
            .and_then(|body| body.split("impl Drop").next())
            .expect("process poll body should remain available");
        let release = body
            .find("drop(active)")
            .expect("poll should release active lock");
        let finish = body
            .find("child.finish(status)")
            .expect("poll should finish child");

        assert!(release < finish);
    }

    #[test]
    fn production_poll_path_rejects_play_without_an_owned_child() {
        let source = include_str!("mod.rs");
        let start = source
            .find("impl PlayBackend for ProcessPlayBackend")
            .expect("process backend implementation should remain available");
        let end = source
            .find("impl Drop for ProcessPlayBackend")
            .expect("process backend drop implementation should remain available");
        let production = &source[start..end];

        assert!(production.contains("runtime preview process is not active while Play is running"));
        assert!(production.contains("ActivePlayProcess::Idle"));
        assert!(!production.contains("diagnostics: Vec::new()"));
    }
}
