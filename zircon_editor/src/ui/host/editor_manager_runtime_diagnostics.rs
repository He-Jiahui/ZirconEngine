use zircon_runtime::core::diagnostics::{collect_runtime_diagnostics, RuntimeDiagnosticsSnapshot};

use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn runtime_diagnostics(&self) -> RuntimeDiagnosticsSnapshot {
        collect_runtime_diagnostics(&self.host.core)
    }
}
