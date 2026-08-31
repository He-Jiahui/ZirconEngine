mod asset_import;
mod behavior;
mod callback;

use std::path::PathBuf;
use std::sync::Arc;

use libloading::Library;

pub(super) use self::behavior::NativePluginBehaviorSnapshot;
use self::behavior::{callback_rejected_report, missing_behavior_report};
pub use self::behavior::{NativePluginEditorCommandBinding, NativePluginEditorCommandBindingError};
pub use self::callback::NativePluginCallbackDiagnostics;
pub(super) use self::callback::{
    NativePluginCallbackLease, NativePluginCallbackLeaseError, NativePluginLibraryGenerationOwner,
    NativePluginLifecycleTransitionError, NativePluginStableLibrary,
};
use super::behavior_calls::NativePluginBehavior;
use super::{
    NativePluginBehaviorCallReport, NativePluginBehaviorHealth,
    NativePluginBehaviorValidationReport, NativePluginDescriptor, NativePluginEntryReport,
};

#[derive(Clone)]
pub struct LoadedNativePlugin {
    pub plugin_id: String,
    pub library_path: PathBuf,
    pub descriptor: Option<NativePluginDescriptor>,
    pub runtime_entry_report: Option<NativePluginEntryReport>,
    pub editor_entry_report: Option<NativePluginEntryReport>,
    pub(super) library: Arc<NativePluginStableLibrary>,
}

impl LoadedNativePlugin {
    pub(super) fn stable_library(library: Library) -> Arc<NativePluginStableLibrary> {
        NativePluginStableLibrary::new(library)
    }

    pub fn is_loaded(&self) -> bool {
        let _ = &self.library.library;
        true
    }

    pub fn callback_diagnostics(&self) -> NativePluginCallbackDiagnostics {
        self.library.diagnostics()
    }

    /// Enables or disables duration/count aggregation without changing callback admission.
    pub fn set_callback_diagnostics_enabled(&self, enabled: bool) {
        self.library.set_diagnostics_enabled(enabled);
    }

    pub(super) fn callback_owner_lease(
        &self,
    ) -> Result<NativePluginCallbackLease, NativePluginCallbackLeaseError> {
        self.library.acquire_callback()
    }

    pub(super) fn library_generation_owner(&self) -> NativePluginLibraryGenerationOwner {
        NativePluginLibraryGenerationOwner::new(self.library.clone())
    }

    pub(super) fn runtime_behavior_snapshot(
        &self,
    ) -> Result<NativePluginBehaviorSnapshot, NativePluginCallbackLeaseError> {
        Ok(NativePluginBehaviorSnapshot::new(
            self.runtime_entry_report
                .as_ref()
                .and_then(|report| report.behavior.as_ref())
                .map(NativePluginBehavior::callback_snapshot),
            "runtime",
            self.library_generation_owner(),
        ))
    }

    fn editor_behavior_snapshot(
        &self,
    ) -> Result<NativePluginBehaviorSnapshot, NativePluginCallbackLeaseError> {
        Ok(NativePluginBehaviorSnapshot::new(
            self.editor_entry_report
                .as_ref()
                .and_then(|report| report.behavior.as_ref())
                .map(NativePluginBehavior::callback_snapshot),
            "editor",
            self.library_generation_owner(),
        ))
    }

    pub(super) fn begin_lifecycle_transition(
        &self,
    ) -> Result<(), NativePluginLifecycleTransitionError> {
        self.library.begin_lifecycle_transition()
    }

    pub(super) fn cancel_lifecycle_transition(&self) {
        self.library.cancel_lifecycle_transition();
    }

    pub fn runtime_behavior_is_stateless(&self) -> Option<bool> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .map(|behavior| behavior.is_stateless)
    }

    pub fn runtime_behavior_validation_report(
        &self,
    ) -> Option<&NativePluginBehaviorValidationReport> {
        self.runtime_entry_report
            .as_ref()
            .map(|report| &report.behavior_validation)
    }

    pub fn editor_behavior_validation_report(
        &self,
    ) -> Option<&NativePluginBehaviorValidationReport> {
        self.editor_entry_report
            .as_ref()
            .map(|report| &report.behavior_validation)
    }

    pub fn runtime_behavior_health(&self) -> Option<NativePluginBehaviorHealth> {
        self.runtime_behavior_validation_report()
            .map(|report| report.health)
    }

    pub fn editor_behavior_health(&self) -> Option<NativePluginBehaviorHealth> {
        self.editor_behavior_validation_report()
            .map(|report| report.health)
    }

    pub fn editor_behavior_is_stateless(&self) -> Option<bool> {
        self.editor_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .map(|behavior| behavior.is_stateless)
    }

    pub fn runtime_command_manifest(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.command_manifest.as_deref())
    }

    pub fn runtime_event_manifest(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.event_manifest.as_deref())
    }

    pub fn runtime_registration_manifest(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.registration_manifest.as_deref())
    }

    pub fn runtime_state_schema_version(&self) -> Option<u32> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .map(|behavior| behavior.state_schema_version)
    }

    pub fn runtime_command_manifest_schema(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.command_manifest_schema.as_deref())
    }

    pub fn runtime_event_manifest_schema(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.event_manifest_schema.as_deref())
    }

    pub fn runtime_registration_manifest_schema(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.registration_manifest_schema.as_deref())
    }

    pub fn invoke_runtime_command(
        &self,
        name: &str,
        payload: &[u8],
    ) -> NativePluginBehaviorCallReport {
        self.runtime_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("runtime", error),
            |snapshot| snapshot.invoke_command(name, payload),
        )
    }

    pub fn bind_editor_command(
        &self,
        command_name: impl Into<String>,
    ) -> Result<NativePluginEditorCommandBinding, NativePluginEditorCommandBindingError> {
        let command_name = command_name.into();
        let snapshot = self.editor_behavior_snapshot().map_err(|error| {
            NativePluginEditorCommandBindingError::CallbackSnapshotUnavailable {
                plugin_id: self.plugin_id.clone(),
                detail: error.to_string(),
            }
        })?;
        snapshot.bind_editor_command(self.plugin_id.clone(), command_name)
    }

    pub fn save_runtime_state(&self) -> NativePluginBehaviorCallReport {
        self.runtime_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("runtime", error),
            |snapshot| snapshot.save_state(),
        )
    }

    pub fn restore_runtime_state(&self, state: &[u8]) -> NativePluginBehaviorCallReport {
        self.runtime_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("runtime", error),
            |snapshot| snapshot.restore_state(state),
        )
    }

    pub fn unload_runtime_behavior(&self) -> NativePluginBehaviorCallReport {
        self.runtime_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("runtime", error),
            |snapshot| snapshot.unload(),
        )
    }

    pub fn save_editor_state(&self) -> NativePluginBehaviorCallReport {
        self.editor_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("editor", error),
            |snapshot| snapshot.save_state(),
        )
    }

    pub fn unload_editor_behavior(&self) -> NativePluginBehaviorCallReport {
        self.editor_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("editor", error),
            |snapshot| snapshot.unload(),
        )
    }

    pub(super) fn save_runtime_state_during_transition(&self) -> NativePluginBehaviorCallReport {
        let behavior = self
            .runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref());
        behavior.map_or_else(
            || missing_behavior_report("runtime"),
            |behavior| self.invoke_lifecycle_callback(|| behavior.save_state()),
        )
    }

    pub(super) fn restore_runtime_state_during_transition(
        &self,
        state: &[u8],
    ) -> NativePluginBehaviorCallReport {
        let behavior = self
            .runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref());
        behavior.map_or_else(
            || missing_behavior_report("runtime"),
            |behavior| self.invoke_lifecycle_callback(|| behavior.restore_state(state)),
        )
    }

    pub(super) fn unload_behavior_during_transition(
        &self,
        module_kind: crate::plugin::PluginModuleKind,
    ) -> NativePluginBehaviorCallReport {
        let behavior = match module_kind {
            crate::plugin::PluginModuleKind::Runtime => self
                .runtime_entry_report
                .as_ref()
                .and_then(|report| report.behavior.as_ref()),
            crate::plugin::PluginModuleKind::Editor => self
                .editor_entry_report
                .as_ref()
                .and_then(|report| report.behavior.as_ref()),
            crate::plugin::PluginModuleKind::Native | crate::plugin::PluginModuleKind::Vm => None,
        };
        behavior.map_or_else(
            || {
                missing_behavior_report(match module_kind {
                    crate::plugin::PluginModuleKind::Editor => "editor",
                    _ => "runtime",
                })
            },
            |behavior| self.invoke_lifecycle_callback(|| behavior.unload()),
        )
    }

    fn invoke_lifecycle_callback(
        &self,
        callback: impl FnOnce() -> NativePluginBehaviorCallReport,
    ) -> NativePluginBehaviorCallReport {
        let started_at = self.library.begin_callback_measurement();
        let report = callback();
        self.library.complete_callback_measurement(started_at);
        report
    }
}

impl std::fmt::Debug for LoadedNativePlugin {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LoadedNativePlugin")
            .field("plugin_id", &self.plugin_id)
            .field("library_path", &self.library_path)
            .field("descriptor", &self.descriptor)
            .field("runtime_entry_report", &self.runtime_entry_report)
            .field("editor_entry_report", &self.editor_entry_report)
            .finish_non_exhaustive()
    }
}
