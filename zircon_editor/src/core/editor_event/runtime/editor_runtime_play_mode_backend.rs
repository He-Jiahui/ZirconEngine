use std::path::Path;
use std::sync::{Arc, Mutex};

use zircon_runtime::{plugin::NativePluginLiveHost, plugin::NativePluginRuntimePlayModeSnapshot};

use super::editor_event_runtime::EditorEventRuntime;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorRuntimePlayModeBackendReport {
    pub diagnostics: Vec<String>,
}

impl EditorRuntimePlayModeBackendReport {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub trait EditorRuntimePlayModeBackend: Send + Sync {
    fn enter_play_mode(
        &self,
        project_root: Option<&Path>,
    ) -> Result<EditorRuntimePlayModeBackendReport, String>;

    fn exit_play_mode(&self) -> Result<EditorRuntimePlayModeBackendReport, String>;
}

pub type SharedEditorRuntimePlayModeBackend = Arc<dyn EditorRuntimePlayModeBackend>;

#[derive(Default)]
pub struct NoopEditorRuntimePlayModeBackend;

impl EditorRuntimePlayModeBackend for NoopEditorRuntimePlayModeBackend {
    fn enter_play_mode(
        &self,
        _project_root: Option<&Path>,
    ) -> Result<EditorRuntimePlayModeBackendReport, String> {
        Ok(EditorRuntimePlayModeBackendReport::default())
    }

    fn exit_play_mode(&self) -> Result<EditorRuntimePlayModeBackendReport, String> {
        Ok(EditorRuntimePlayModeBackendReport::default())
    }
}

pub struct NativePluginEditorRuntimePlayModeBackend {
    live_host: Arc<NativePluginLiveHost>,
    active_snapshot: Mutex<Option<NativePluginRuntimePlayModeSnapshot>>,
}

impl NativePluginEditorRuntimePlayModeBackend {
    pub fn new(live_host: Arc<NativePluginLiveHost>) -> Self {
        Self {
            live_host,
            active_snapshot: Mutex::new(None),
        }
    }
}

impl EditorRuntimePlayModeBackend for NativePluginEditorRuntimePlayModeBackend {
    fn enter_play_mode(
        &self,
        project_root: Option<&Path>,
    ) -> Result<EditorRuntimePlayModeBackendReport, String> {
        let mut active = self
            .active_snapshot
            .lock()
            .map_err(|_| "runtime play-mode backend lock is poisoned".to_string())?;
        if active.is_some() {
            return Err("runtime play-mode backend already has an active snapshot".to_string());
        }

        let mut diagnostics = Vec::new();
        if let Some(project_root) = project_root {
            let load = self
                .live_host
                .load_runtime_plugins_from_project_root(project_root)?;
            diagnostics.extend(load.diagnostics);
        } else {
            diagnostics.push(
                "runtime native plugin load skipped because the editor project root is unavailable"
                    .to_string(),
            );
        }

        let snapshot = self.live_host.enter_runtime_play_mode()?;
        diagnostics.extend(snapshot.combined_diagnostics());
        diagnostics.sort();
        diagnostics.dedup();
        *active = Some(snapshot);
        Ok(EditorRuntimePlayModeBackendReport { diagnostics })
    }

    fn exit_play_mode(&self) -> Result<EditorRuntimePlayModeBackendReport, String> {
        let snapshot = self
            .active_snapshot
            .lock()
            .map_err(|_| "runtime play-mode backend lock is poisoned".to_string())?
            .take();
        let Some(snapshot) = snapshot else {
            return Ok(EditorRuntimePlayModeBackendReport {
                diagnostics: vec![
                    "runtime play-mode backend had no active snapshot to restore".to_string(),
                ],
            });
        };

        let report = self.live_host.exit_runtime_play_mode(&snapshot)?;
        Ok(EditorRuntimePlayModeBackendReport {
            diagnostics: report.combined_diagnostics(),
        })
    }
}

impl EditorEventRuntime {
    pub fn set_runtime_play_mode_backend(&self, backend: SharedEditorRuntimePlayModeBackend) {
        self.lock_inner().runtime_play_mode_backend = backend;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_plugin_play_mode_backend_roundtrips_empty_live_host() {
        let backend = NativePluginEditorRuntimePlayModeBackend::new(Arc::new(
            NativePluginLiveHost::default(),
        ));

        let entered = backend
            .enter_play_mode(None)
            .expect("empty native live host should enter play mode");
        assert!(entered
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("project root is unavailable")));

        let exited = backend
            .exit_play_mode()
            .expect("empty native live host should exit play mode");
        assert!(exited.is_clean());
    }
}
