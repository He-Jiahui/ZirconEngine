use zircon_runtime::core::diagnostics::{collect_runtime_diagnostics, RuntimeDiagnosticsSnapshot};

use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn runtime_diagnostics(&self) -> RuntimeDiagnosticsSnapshot {
        self.host
            .runtime_core()
            .map(|core| collect_runtime_diagnostics(&core))
            .unwrap_or_default()
    }
}
