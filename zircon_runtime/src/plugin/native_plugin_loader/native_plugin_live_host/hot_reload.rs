use crate::plugin::PluginModuleKind;

use super::super::{
    LoadedNativePlugin, NativePluginBehaviorCallReport, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::keys::module_kind_label;

#[derive(Debug)]
pub(super) struct NativePluginHotReloadState {
    pub(super) module_kind: PluginModuleKind,
    pub(super) key: String,
    existing: Option<LoadedNativePlugin>,
    previous_unloaded: bool,
    diagnostics: Vec<String>,
    runtime_snapshot: Option<PluginStateSnapshot>,
}

impl NativePluginHotReloadState {
    pub(super) fn new(
        module_kind: PluginModuleKind,
        key: String,
        existing: Option<LoadedNativePlugin>,
    ) -> Self {
        Self {
            module_kind,
            key,
            existing,
            previous_unloaded: false,
            diagnostics: Vec::new(),
            runtime_snapshot: None,
        }
    }

    pub(super) fn save_existing_runtime_snapshot(
        &mut self,
        plugin_id: &str,
    ) -> Result<Option<&PluginStateSnapshot>, String> {
        if self.module_kind != PluginModuleKind::Runtime {
            return Ok(None);
        }
        let Some(existing) = self.existing.as_ref() else {
            return Ok(None);
        };
        if existing.runtime_behavior_is_stateless() != Some(false) {
            return Ok(None);
        }
        let report = existing.save_runtime_state();
        self.diagnostics.extend(prefixed_behavior_diagnostics(
            "runtime save-state before hot reload",
            &report,
        ));
        if report.status_code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
            return Err(format!(
                "plugin {plugin_id} hot reload failed while saving runtime state: status {}",
                report.status_code
            ));
        }
        let Some(blob) = report.payload else {
            return Err(format!(
                "plugin {plugin_id} hot reload failed because runtime save-state returned no payload"
            ));
        };
        self.runtime_snapshot = Some(PluginStateSnapshot {
            plugin_id: plugin_id.to_string(),
            module_kind: PluginModuleKind::Runtime,
            schema_version: existing.runtime_state_schema_version(),
            blob,
        });
        Ok(self.runtime_snapshot.as_ref())
    }

    pub(super) fn runtime_snapshot(&self) -> Option<&PluginStateSnapshot> {
        self.runtime_snapshot.as_ref()
    }

    pub(super) fn take_existing_for_unload(&mut self) -> Option<LoadedNativePlugin> {
        self.existing.take()
    }

    pub(super) fn mark_existing_unloaded(&mut self, diagnostics: Vec<String>) {
        self.previous_unloaded = true;
        self.diagnostics.extend(diagnostics);
    }

    pub(super) fn rollback_error(&mut self, error: String) -> String {
        let rollback = if self.existing.is_some() {
            format!(
                "rolled back to the previously loaded {} native package",
                module_kind_label(self.module_kind)
            )
        } else if self.previous_unloaded {
            format!(
                "rollback unavailable because previous {} native package was already unloaded",
                module_kind_label(self.module_kind)
            )
        } else {
            format!(
                "rollback not needed because no {} native package was previously loaded",
                module_kind_label(self.module_kind)
            )
        };
        let diagnostics = if self.diagnostics.is_empty() {
            rollback
        } else {
            format!("{rollback}; {}", self.diagnostics.join("; "))
        };
        format!("{error}; {diagnostics}")
    }

    pub(super) fn into_rollback_plugin(self) -> Option<LoadedNativePlugin> {
        self.existing
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PluginStateSnapshot {
    pub(super) plugin_id: String,
    pub(super) module_kind: PluginModuleKind,
    pub(super) schema_version: Option<u32>,
    pub(super) blob: Vec<u8>,
}

pub(super) fn restore_runtime_snapshot(
    snapshot: &PluginStateSnapshot,
    plugin: &LoadedNativePlugin,
) -> Result<Vec<String>, String> {
    if snapshot.module_kind != PluginModuleKind::Runtime {
        return Ok(Vec::new());
    }
    let loaded_schema = plugin.runtime_state_schema_version();
    if snapshot.schema_version != loaded_schema {
        return Err(format!(
            "plugin {} hot reload restore-state skipped because snapshot state schema {:?} does not match loaded state schema {:?}",
            snapshot.plugin_id, snapshot.schema_version, loaded_schema
        ));
    }
    let report = plugin.restore_runtime_state(&snapshot.blob);
    let diagnostics =
        prefixed_behavior_diagnostics("runtime restore-state after hot reload", &report);
    if report.status_code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
        return Err(format!(
            "plugin {} hot reload failed while restoring runtime state: status {}; {}",
            snapshot.plugin_id,
            report.status_code,
            diagnostics.join("; ")
        ));
    }
    Ok(diagnostics)
}

fn prefixed_behavior_diagnostics(
    label: &str,
    report: &NativePluginBehaviorCallReport,
) -> Vec<String> {
    if report.diagnostics.is_empty() {
        if report.status_code == ZIRCON_NATIVE_PLUGIN_STATUS_OK {
            Vec::new()
        } else {
            vec![format!("{label} returned status {}", report.status_code)]
        }
    } else {
        report
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{label}: {diagnostic}"))
            .collect()
    }
}
