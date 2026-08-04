use std::path::PathBuf;
use std::sync::Mutex;

use super::{
    PlayBackend, PlayBackendPoll, PlayBackendStartReport, PlayBackendStopReport, PlaySnapshotStore,
    PlayStartRequest,
};

mod child;
mod command;
mod output;
#[cfg(test)]
mod tests;

use child::PlayChild;
use command::{runtime_executable_next_to_current_process, PlayProcessCommand};

pub struct ProcessPlayBackend {
    executable: PathBuf,
    snapshots: PlaySnapshotStore,
    active: Mutex<Option<PlayChild>>,
}

impl ProcessPlayBackend {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            snapshots: PlaySnapshotStore::default(),
            active: Mutex::new(None),
        }
    }

    pub fn for_current_install() -> Result<Self, String> {
        runtime_executable_next_to_current_process().map(Self::new)
    }

    pub fn is_active(&self) -> bool {
        self.active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
    }
}

impl PlayBackend for ProcessPlayBackend {
    fn start(&self, request: &PlayStartRequest) -> Result<PlayBackendStartReport, String> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if active.is_some() {
            return Err("runtime preview process is already active".to_string());
        }
        let project_root = request
            .project_root
            .as_deref()
            .ok_or_else(|| "process play requires an open project root".to_string())?;
        let source = request.scene_source.as_ref().ok_or_else(|| {
            "process play requires a persisted scene or world snapshot".to_string()
        })?;
        let scene = self.snapshots.materialize(project_root, source)?;
        let report_pipe = format!("zircon-play-report-{}", scene.instance_id());
        let command =
            PlayProcessCommand::new(&self.executable, project_root, scene.path(), &report_pipe);
        let arguments = command
            .arguments()
            .into_iter()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");
        let child = PlayChild::spawn(&command, scene)?;
        let pid = child.id();
        *active = Some(child);
        Ok(PlayBackendStartReport {
            diagnostics: vec![
                format!("process.pid={pid}"),
                format!("process.report_pipe={}", command.report_pipe()),
                format!("process.args={arguments}"),
            ],
            attachable: false,
        })
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        let child = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        let Some(child) = child else {
            return Ok(PlayBackendStopReport::default());
        };
        let outcome = child.stop()?;
        Ok(PlayBackendStopReport {
            diagnostics: outcome.diagnostics,
        })
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(child) = active.as_mut() else {
            return Ok(PlayBackendPoll::Running {
                diagnostics: Vec::new(),
            });
        };
        let Some(status) = child.try_wait()? else {
            return Ok(PlayBackendPoll::Running {
                diagnostics: child.drain_output(),
            });
        };
        let child = active.take().ok_or_else(|| {
            "runtime preview process disappeared before terminal completion".to_string()
        })?;
        drop(active);
        let outcome = child.finish(status);
        Ok(PlayBackendPoll::Exited {
            exit_code: outcome.exit_code,
            diagnostics: outcome.diagnostics,
        })
    }
}

impl Drop for ProcessPlayBackend {
    fn drop(&mut self) {
        let child = self
            .active
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if let Some(child) = child {
            let _ = child.stop();
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
    fn production_poll_path_recovers_instead_of_panicking_on_a_missing_child() {
        let source = include_str!("mod.rs");
        let start = source
            .find("impl PlayBackend for ProcessPlayBackend")
            .expect("process backend implementation should remain available");
        let end = source
            .find("impl Drop for ProcessPlayBackend")
            .expect("process backend drop implementation should remain available");
        let production = &source[start..end];

        assert!(
            production.contains("runtime preview process disappeared before terminal completion")
        );
        assert!(!production
            .contains("active runtime preview should remain present until terminal poll"));
    }
}
