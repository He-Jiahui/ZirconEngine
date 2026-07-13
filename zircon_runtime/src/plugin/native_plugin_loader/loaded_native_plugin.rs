use std::path::PathBuf;

use libloading::Library;

use crate::asset::{
    NativeAssetImportCommandHost, NativeAssetImportCommandReport, NativeAssetImportCommandStatus,
};

use super::{
    NativePluginBehaviorCallReport, NativePluginBehaviorHealth,
    NativePluginBehaviorValidationReport, NativePluginDescriptor, NativePluginEntryReport,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
};

impl NativeAssetImportCommandHost for LoadedNativePlugin {
    fn command_host_id(&self) -> &str {
        &self.plugin_id
    }

    fn invoke_asset_import_command(
        &self,
        command: &str,
        payload: &[u8],
    ) -> NativeAssetImportCommandReport {
        let report = LoadedNativePlugin::invoke_runtime_command(self, command, payload);
        let status = match report.status_code {
            super::ZIRCON_NATIVE_PLUGIN_STATUS_OK => NativeAssetImportCommandStatus::Ok,
            super::ZIRCON_NATIVE_PLUGIN_STATUS_ERROR => NativeAssetImportCommandStatus::Error,
            super::ZIRCON_NATIVE_PLUGIN_STATUS_DENIED => NativeAssetImportCommandStatus::Denied,
            super::ZIRCON_NATIVE_PLUGIN_STATUS_PANIC => NativeAssetImportCommandStatus::Panic,
            status => NativeAssetImportCommandStatus::Unknown(status),
        };
        NativeAssetImportCommandReport {
            status,
            diagnostics: report.diagnostics,
            payload: report.payload,
        }
    }
}

pub struct LoadedNativePlugin {
    pub plugin_id: String,
    pub library_path: PathBuf,
    pub descriptor: Option<NativePluginDescriptor>,
    pub runtime_entry_report: Option<NativePluginEntryReport>,
    pub editor_entry_report: Option<NativePluginEntryReport>,
    pub(super) library: Library,
}

impl LoadedNativePlugin {
    pub fn is_loaded(&self) -> bool {
        let _ = &self.library;
        true
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
        let _library = &self.library;
        let Some(behavior) = self
            .runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
        else {
            return missing_behavior_report("runtime");
        };
        behavior.invoke_command(name, payload)
    }

    pub fn save_runtime_state(&self) -> NativePluginBehaviorCallReport {
        let _library = &self.library;
        let Some(behavior) = self
            .runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
        else {
            return missing_behavior_report("runtime");
        };
        behavior.save_state()
    }

    pub fn restore_runtime_state(&self, state: &[u8]) -> NativePluginBehaviorCallReport {
        let _library = &self.library;
        let Some(behavior) = self
            .runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
        else {
            return missing_behavior_report("runtime");
        };
        behavior.restore_state(state)
    }

    pub fn unload_runtime_behavior(&self) -> NativePluginBehaviorCallReport {
        let _library = &self.library;
        let Some(behavior) = self
            .runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
        else {
            return missing_behavior_report("runtime");
        };
        behavior.unload()
    }

    pub fn save_editor_state(&self) -> NativePluginBehaviorCallReport {
        let _library = &self.library;
        let Some(behavior) = self
            .editor_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
        else {
            return missing_behavior_report("editor");
        };
        behavior.save_state()
    }

    pub fn unload_editor_behavior(&self) -> NativePluginBehaviorCallReport {
        let _library = &self.library;
        let Some(behavior) = self
            .editor_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
        else {
            return missing_behavior_report("editor");
        };
        behavior.unload()
    }
}

fn missing_behavior_report(module_kind: &str) -> NativePluginBehaviorCallReport {
    NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: vec![format!("native plugin {module_kind} behavior is missing")],
        payload: None,
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
