use super::editor_capabilities::EditorCapabilitySnapshot;
use super::editor_error::EditorError;
use super::editor_subsystems::EditorSubsystemReport;
use super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    /// Applies one already-resolved subsystem report without acquiring a second runtime handle.
    pub(super) fn apply_capability_report(
        &self,
        subsystem_report: EditorSubsystemReport,
    ) -> Result<EditorCapabilitySnapshot, EditorError> {
        let snapshot =
            EditorCapabilitySnapshot::from_reports(&self.minimal_report, &subsystem_report);
        *self.lock_subsystem_report() = subsystem_report;
        *self.lock_capability_snapshot() = snapshot.clone();
        self.register_builtin_views()?;
        Ok(snapshot)
    }
}
