#![cfg(debug_assertions)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, TrySendError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use zircon_runtime::plugin::native::{NativePluginHostHandle, NativePluginHostWeakHandle};

const RELOAD_DEBOUNCE: Duration = Duration::from_millis(350);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct DevelopmentPluginWatchKey {
    project_root: PathBuf,
    plugin_id: String,
    artifact_path: PathBuf,
}

impl DevelopmentPluginWatchKey {
    pub(super) fn new(
        project_root: &Path,
        plugin_id: &str,
        artifact_path: &Path,
    ) -> Result<Self, String> {
        let project_root = std::fs::canonicalize(project_root).map_err(|error| {
            format!(
                "cannot watch native plugin `{plugin_id}` under {}: {error}",
                project_root.display()
            )
        })?;
        let artifact_path = std::fs::canonicalize(artifact_path).map_err(|error| {
            format!(
                "cannot watch native plugin `{plugin_id}` artifact {}: {error}",
                artifact_path.display()
            )
        })?;
        if !artifact_path.starts_with(&project_root) {
            return Err(format!(
                "native plugin `{plugin_id}` artifact {} is outside project root {}",
                artifact_path.display(),
                project_root.display()
            ));
        }
        Ok(Self {
            project_root,
            plugin_id: plugin_id.to_string(),
            artifact_path,
        })
    }

    pub(super) fn plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

pub(super) struct DevelopmentPluginWatch {
    watcher: Option<RecommendedWatcher>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl DevelopmentPluginWatch {
    pub(super) fn start(
        live_host: &NativePluginHostHandle,
        key: DevelopmentPluginWatchKey,
    ) -> Result<Self, String> {
        let (changed, pending_changes) = mpsc::sync_channel(1);
        let callback_plugin_id = key.plugin_id.clone();
        let callback_artifact_path = key.artifact_path.clone();
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
            let should_reload = event
                .as_ref()
                .is_ok_and(|event| {
                    development_event_requests_reload(event, &callback_artifact_path)
                });
            if !should_reload {
                if let Err(error) = event {
                    eprintln!(
                        "[zircon_editor] native development watch for `{callback_plugin_id}` failed: {error}"
                    );
                }
                return;
            }
            match changed.try_send(()) {
                Ok(()) | Err(TrySendError::Full(())) => {}
                Err(TrySendError::Disconnected(())) => {}
            }
        })
        .map_err(|error| format!("native development watcher creation failed: {error}"))?;
        let artifact_parent = key.artifact_path.parent().ok_or_else(|| {
            format!(
                "native plugin `{}` artifact has no parent directory: {}",
                key.plugin_id,
                key.artifact_path.display()
            )
        })?;
        watcher
            .watch(artifact_parent, RecursiveMode::NonRecursive)
            .map_err(|error| {
                format!(
                    "native development watcher could not watch {}: {error}",
                    artifact_parent.display()
                )
            })?;

        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let host = live_host.downgrade();
        let worker_key = key.clone();
        let worker = thread::Builder::new()
            .name(format!("zr-plugin-watch-{}", key.plugin_id))
            .spawn(move || {
                run_development_watch_worker(host, worker_key, worker_stop, pending_changes)
            })
            .map_err(|error| format!("native development watcher worker failed: {error}"))?;

        Ok(Self {
            watcher: Some(watcher),
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for DevelopmentPluginWatch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.watcher.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run_development_watch_worker(
    live_host: NativePluginHostWeakHandle,
    key: DevelopmentPluginWatchKey,
    stop: Arc<AtomicBool>,
    pending_changes: mpsc::Receiver<()>,
) {
    while pending_changes.recv().is_ok() {
        loop {
            match pending_changes.recv_timeout(RELOAD_DEBOUNCE) {
                Ok(()) => {}
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        if stop.load(Ordering::Acquire) {
            return;
        }
        let Some(live_host) = live_host.upgrade() else {
            return;
        };
        match live_host.hot_reload_editor_plugin(&key.project_root, &key.plugin_id) {
            Ok(outcome) => eprintln!(
                "[zircon_editor] native plugin `{}` hot reloaded after artifact change: {}",
                key.plugin_id,
                outcome.diagnostics.join("; ")
            ),
            Err(error) => eprintln!(
                "[zircon_editor] native plugin `{}` development hot reload failed: {error}",
                key.plugin_id
            ),
        }
    }
}

fn development_event_requests_reload(event: &Event, artifact_path: &Path) -> bool {
    if !matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) {
        return false;
    }
    event.paths.iter().any(|path| path == artifact_path)
}

#[cfg(test)]
mod tests {
    use notify::event::ModifyKind;

    use super::*;

    #[test]
    fn development_watch_filters_for_the_exact_loaded_artifact() {
        let artifact_path = PathBuf::from("project/native/plugin.dll");
        assert!(development_event_requests_reload(
            &Event {
                kind: EventKind::Modify(ModifyKind::Any),
                paths: vec![artifact_path.clone()],
                attrs: Default::default(),
            },
            &artifact_path,
        ));

        for path in [
            PathBuf::from("project/other/native/other.dll"),
            PathBuf::from("project/plugin.toml"),
            PathBuf::from("project/native/plugin.pdb"),
        ] {
            assert!(!development_event_requests_reload(
                &Event {
                    kind: EventKind::Modify(ModifyKind::Any),
                    paths: vec![path],
                    attrs: Default::default(),
                },
                &artifact_path,
            ));
        }
        assert!(!development_event_requests_reload(
            &Event {
                kind: EventKind::Access(notify::event::AccessKind::Any),
                paths: vec![artifact_path.clone()],
                attrs: Default::default(),
            },
            &artifact_path,
        ));
    }
}
