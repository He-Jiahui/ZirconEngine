use crate::ui::host::EditorRuntimeSessionShutdownReceipt;

use super::RetainedEditorHost;

impl RetainedEditorHost {
    /// Runs the editor-owned session retirement exactly once before the host releases its App
    /// runtime owner. The receipt remains available for terminal diagnostics after remote cleanup
    /// has failed or the transport has already been replaced.
    pub(in crate::ui::retained_host::app) fn shutdown_runtime_session(&mut self) {
        if self.runtime_shutdown_receipt.is_some() {
            return;
        }
        self.hierarchy_world_watch.take();
        self.runtime_shutdown_receipt = Some(self.runtime.shutdown_runtime_session());
    }

    pub(in crate::ui::retained_host::app) fn runtime_shutdown_receipt(
        &self,
    ) -> Option<&EditorRuntimeSessionShutdownReceipt> {
        self.runtime_shutdown_receipt.as_ref()
    }
}
