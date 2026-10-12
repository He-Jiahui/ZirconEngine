use super::super::RetainedEditorHost;
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

#[cfg(feature = "profiling")]
use super::snapshot_merge::merge_profile_snapshot;
#[cfg(feature = "profiling")]
use zircon_runtime_interface::{ProfileControlCommand, ProfileControlRequest, ProfileSnapshot};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn runtime_diagnostics_with_profile(
        &self,
    ) -> Result<RuntimeDiagnosticsSnapshot, crate::ui::host::EditorError> {
        #[cfg(feature = "profiling")]
        {
            let mut diagnostics = self.editor_manager.enter()?.runtime_diagnostics();
            self.merge_dynamic_runtime_profile(&mut diagnostics.profile);
            Ok(diagnostics)
        }

        #[cfg(not(feature = "profiling"))]
        {
            Ok(self.editor_manager.enter()?.runtime_diagnostics())
        }
    }

    #[cfg(feature = "profiling")]
    fn merge_dynamic_runtime_profile(&self, editor_profile: &mut ProfileSnapshot) {
        let request = ProfileControlRequest {
            command: ProfileControlCommand::Snapshot,
            config: None,
        };
        let Ok(Some(response)) = self.runtime_gateway.profile_control(&request) else {
            return;
        };
        let Some(runtime_profile) = response.snapshot else {
            return;
        };
        merge_profile_snapshot(editor_profile, runtime_profile);
    }
}
