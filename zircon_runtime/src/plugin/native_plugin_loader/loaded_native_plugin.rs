use std::path::PathBuf;

use libloading::Library;

use super::{
    NativePluginBehaviorCallReport, NativePluginDescriptor, NativePluginEntryReport,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
};

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
