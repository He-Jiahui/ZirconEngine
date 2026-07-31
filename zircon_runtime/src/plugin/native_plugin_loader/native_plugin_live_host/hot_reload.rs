use crate::plugin::PluginModuleKind;

use super::super::{
    LoadedNativePlugin, NativePluginBehaviorCallReport, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::keys::module_kind_label;

pub(super) type NativePluginHotReloadResult<T> = std::result::Result<T, NativePluginHotReloadError>;

#[derive(Debug)]
pub(super) enum NativePluginHotReloadError {
    SaveRuntimeState {
        plugin_id: String,
        status_code: u32,
    },
    MissingRuntimeStatePayload {
        plugin_id: String,
    },
    StateSchemaMismatch {
        plugin_id: String,
        snapshot_schema: Option<u32>,
        loaded_schema: Option<u32>,
    },
    RestoreRuntimeState {
        plugin_id: String,
        status_code: u32,
        diagnostics: Vec<String>,
    },
}

impl std::fmt::Display for NativePluginHotReloadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SaveRuntimeState {
                plugin_id,
                status_code,
            } => write!(
                formatter,
                "plugin {plugin_id} hot reload failed while saving runtime state: status {status_code}"
            ),
            Self::MissingRuntimeStatePayload { plugin_id } => write!(
                formatter,
                "plugin {plugin_id} hot reload failed because runtime save-state returned no payload"
            ),
            Self::StateSchemaMismatch {
                plugin_id,
                snapshot_schema,
                loaded_schema,
            } => write!(
                formatter,
                "plugin {plugin_id} hot reload restore-state skipped because snapshot state schema {snapshot_schema:?} does not match loaded state schema {loaded_schema:?}"
            ),
            Self::RestoreRuntimeState {
                plugin_id,
                status_code,
                diagnostics,
            } => write!(
                formatter,
                "plugin {plugin_id} hot reload failed while restoring runtime state: status {status_code}; {}",
                diagnostics.join("; ")
            ),
        }
    }
}

impl std::error::Error for NativePluginHotReloadError {}

#[derive(Debug)]
pub(super) struct NativePluginHotReloadState {
    pub(super) module_kind: PluginModuleKind,
    pub(super) key: String,
    existing: Option<LoadedNativePlugin>,
    previous_plugin_disposition: PreviousPluginDisposition,
    diagnostics: Vec<String>,
    runtime_snapshot: Option<PluginStateSnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreviousPluginDisposition {
    NotLoaded,
    HeldForRollback,
    Unloaded,
    Restored,
}

impl NativePluginHotReloadState {
    pub(super) fn new(
        module_kind: PluginModuleKind,
        key: String,
        existing: Option<LoadedNativePlugin>,
    ) -> Self {
        let previous_plugin_disposition = if existing.is_some() {
            PreviousPluginDisposition::HeldForRollback
        } else {
            PreviousPluginDisposition::NotLoaded
        };
        Self {
            module_kind,
            key,
            existing,
            previous_plugin_disposition,
            diagnostics: Vec::new(),
            runtime_snapshot: None,
        }
    }

    pub(super) fn save_existing_runtime_snapshot(
        &mut self,
        plugin_id: &str,
    ) -> NativePluginHotReloadResult<Option<&PluginStateSnapshot>> {
        if self.module_kind != PluginModuleKind::Runtime {
            return Ok(None);
        }
        let Some(existing) = self.existing.as_ref() else {
            return Ok(None);
        };
        if existing.runtime_behavior_is_stateless() != Some(false) {
            return Ok(None);
        }
        let report = existing.save_runtime_state_during_transition();
        self.diagnostics.extend(prefixed_behavior_diagnostics(
            "runtime save-state before hot reload",
            &report,
        ));
        if report.status_code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
            return Err(NativePluginHotReloadError::SaveRuntimeState {
                plugin_id: plugin_id.to_string(),
                status_code: report.status_code,
            });
        }
        let Some(blob) = report.payload else {
            return Err(NativePluginHotReloadError::MissingRuntimeStatePayload {
                plugin_id: plugin_id.to_string(),
            });
        };
        self.runtime_snapshot = Some(PluginStateSnapshot {
            plugin_id: plugin_id.to_string(),
            module_kind: PluginModuleKind::Runtime,
            schema_version: existing.runtime_state_schema_version(),
            blob,
        });
        Ok(self.runtime_snapshot.as_ref())
    }

    pub(super) fn take_runtime_snapshot(&mut self) -> Option<PluginStateSnapshot> {
        self.runtime_snapshot.take()
    }

    pub(super) fn take_existing_for_unload(&mut self) -> Option<LoadedNativePlugin> {
        self.existing.take()
    }

    pub(super) fn mark_existing_unloaded(&mut self, diagnostics: Vec<String>) {
        self.previous_plugin_disposition = PreviousPluginDisposition::Unloaded;
        self.diagnostics.extend(diagnostics);
    }

    pub(super) fn mark_existing_restored(&mut self) {
        self.previous_plugin_disposition = PreviousPluginDisposition::Restored;
    }

    pub(super) fn rollback_error(&mut self, error: String) -> String {
        format!("{error}; {}", self.rollback_diagnostic())
    }

    pub(super) fn rollback_diagnostic(&self) -> String {
        let rollback = match self.previous_plugin_disposition {
            PreviousPluginDisposition::HeldForRollback | PreviousPluginDisposition::Restored => {
                format!(
                    "rolled back to the previously loaded {} native package",
                    module_kind_label(self.module_kind)
                )
            }
            PreviousPluginDisposition::Unloaded => format!(
                "rollback unavailable because previous {} native package was already unloaded",
                module_kind_label(self.module_kind)
            ),
            PreviousPluginDisposition::NotLoaded => format!(
                "rollback not needed because no {} native package was previously loaded",
                module_kind_label(self.module_kind)
            ),
        };
        if self.diagnostics.is_empty() {
            rollback
        } else {
            format!("{rollback}; {}", self.diagnostics.join("; "))
        }
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
) -> NativePluginHotReloadResult<Vec<String>> {
    if snapshot.module_kind != PluginModuleKind::Runtime {
        return Ok(Vec::new());
    }
    let loaded_schema = plugin.runtime_state_schema_version();
    if snapshot.schema_version != loaded_schema {
        return Err(NativePluginHotReloadError::StateSchemaMismatch {
            plugin_id: snapshot.plugin_id.clone(),
            snapshot_schema: snapshot.schema_version,
            loaded_schema,
        });
    }
    let report = plugin.restore_runtime_state_during_transition(&snapshot.blob);
    let diagnostics =
        prefixed_behavior_diagnostics("runtime restore-state after hot reload", &report);
    if report.status_code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
        return Err(NativePluginHotReloadError::RestoreRuntimeState {
            plugin_id: snapshot.plugin_id.clone(),
            status_code: report.status_code,
            diagnostics,
        });
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
