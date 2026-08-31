use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

use super::editor_manager::EditorManager;
use crate::core::logging::{EditorLogError, RuntimeTaskDiagnosticProjectionReport};

impl EditorManager {
    pub fn runtime_diagnostics(&self) -> RuntimeDiagnosticsSnapshot {
        self.host.runtime_services.runtime_diagnostics()
    }

    pub(crate) fn pump_runtime_task_diagnostics(
        &self,
        timestamp_frame: u64,
    ) -> Result<RuntimeTaskDiagnosticProjectionReport, EditorLogError> {
        self.runtime_task_diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pump(self.context.logs(), timestamp_frame)
    }
}
