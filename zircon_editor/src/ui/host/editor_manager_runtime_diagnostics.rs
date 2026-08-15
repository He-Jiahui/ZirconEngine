use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn runtime_diagnostics(&self) -> RuntimeDiagnosticsSnapshot {
        self.host.runtime_services.runtime_diagnostics()
    }
}
