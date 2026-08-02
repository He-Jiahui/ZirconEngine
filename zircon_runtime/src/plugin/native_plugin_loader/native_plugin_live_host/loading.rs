use std::sync::MutexGuard;

use crate::plugin::PluginModuleKind;

use super::super::loaded_native_plugin::NativePluginLifecycleTransitionError;
use super::super::{LoadedNativePlugin, NativePluginLoadReport};
use super::bridge_methods::{
    discovered_runtime_bridge_method_binding_diagnostics,
    discovered_runtime_bridge_method_binding_error_diagnostic,
    discovered_runtime_bridge_method_bindings_result, NativePluginBridgeMethodError,
};
use super::diagnostics::{
    diagnostics_from_behavior_report, load_projected_report_diagnostics,
    NativePluginBehaviorDiagnosticError,
};
use super::keys::{live_key, module_kind_label, NativePluginLiveRegistry};
use super::reports::NativePluginLiveHostLoadReport;
use super::runtime_behavior::unload_behavior;
use super::{NativePluginLiveHost, ObservedLoadedNativePlugins};

pub(super) type NativePluginLiveHostLoadingResult<T> =
    std::result::Result<T, NativePluginLiveHostLoadingError>;

#[derive(Debug)]
pub(super) enum NativePluginLiveHostLoadingError {
    LiveHostLockPoisoned,
    PluginBusy {
        plugin_id: String,
        module_kind: PluginModuleKind,
        source: NativePluginLifecycleTransitionError,
    },
    UnloadBeforeReload {
        plugin_id: String,
        module_kind: PluginModuleKind,
        source: NativePluginBehaviorDiagnosticError,
    },
    RuntimeBridgeMethodBindings {
        plugin_id: String,
        source: Box<NativePluginBridgeMethodError>,
    },
}

impl std::fmt::Display for NativePluginLiveHostLoadingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LiveHostLockPoisoned => {
                formatter.write_str("native plugin live host lock is poisoned")
            }
            Self::PluginBusy {
                plugin_id,
                module_kind,
                source,
            } => write!(
                formatter,
                "{} plugin {plugin_id} lifecycle is busy during load: {source}",
                module_kind_label(*module_kind)
            ),
            Self::UnloadBeforeReload {
                plugin_id,
                module_kind,
                source,
            } => write!(
                formatter,
                "{} plugin {plugin_id} unload before reload failed: {source}",
                module_kind_label(*module_kind)
            ),
            Self::RuntimeBridgeMethodBindings { plugin_id, source } => write!(
                formatter,
                "runtime plugin {plugin_id} bridge method binding install failed while loading: {source}"
            ),
        }
    }
}

impl std::error::Error for NativePluginLiveHostLoadingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PluginBusy { source, .. } => Some(source),
            Self::UnloadBeforeReload { source, .. } => Some(source),
            Self::RuntimeBridgeMethodBindings { source, .. } => Some(source.as_ref()),
            Self::LiveHostLockPoisoned => None,
        }
    }
}

impl NativePluginLiveHost {
    pub fn load_runtime_plugins_from_export_root(
        &self,
        export_root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_runtime_from_load_manifest(export_root);
        self.load_reported_plugins_result(report, PluginModuleKind::Runtime)
            .map_err(|error| error.to_string())
    }

    pub fn load_editor_plugins_from_export_root(
        &self,
        export_root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_editor_from_load_manifest(export_root);
        self.load_reported_plugins_result(report, PluginModuleKind::Editor)
            .map_err(|error| error.to_string())
    }

    pub fn load_runtime_plugins_from_project_root(
        &self,
        root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_discovered_runtime(root);
        self.load_reported_plugins_result(report, PluginModuleKind::Runtime)
            .map_err(|error| error.to_string())
    }

    pub fn load_editor_plugins_from_project_root(
        &self,
        root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_discovered_editor(root);
        self.load_reported_plugins_result(report, PluginModuleKind::Editor)
            .map_err(|error| error.to_string())
    }

    pub fn loaded_plugin_ids(&self, module_kind: PluginModuleKind) -> Result<Vec<String>, String> {
        let loaded = lock_loaded_native_plugins(&self.loaded).map_err(|error| error.to_string())?;
        Ok(loaded.plugin_ids(module_kind).map(str::to_string).collect())
    }

    pub(super) fn load_reported_plugins(
        &self,
        report: NativePluginLoadReport,
        module_kind: PluginModuleKind,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.load_reported_plugins_result(report, module_kind)
            .map_err(|error| error.to_string())
    }

    pub(super) fn load_reported_plugins_result(
        &self,
        mut report: NativePluginLoadReport,
        module_kind: PluginModuleKind,
    ) -> NativePluginLiveHostLoadingResult<NativePluginLiveHostLoadReport> {
        let projection = report.projection();
        let (runtime_plugin_registration_reports, runtime_plugin_feature_registration_reports) =
            match module_kind {
                PluginModuleKind::Runtime => (
                    projection.runtime_plugin_registration_reports(),
                    projection.runtime_plugin_feature_registration_reports(),
                ),
                PluginModuleKind::Editor | PluginModuleKind::Native | PluginModuleKind::Vm => {
                    (Vec::new(), Vec::new())
                }
            };
        let mut diagnostics = load_projected_report_diagnostics(&report, &projection);
        let mut loaded_plugin_ids = Vec::new();

        for plugin in report.take_loaded() {
            let plugin_id = plugin.plugin_id.clone();
            let key = live_key(module_kind, &plugin_id);
            let bridge_binding_update = if module_kind == PluginModuleKind::Runtime {
                match discovered_runtime_bridge_method_bindings_result(&plugin) {
                    Ok(Some(bindings)) => {
                        diagnostics.push(discovered_runtime_bridge_method_binding_diagnostics(
                            &plugin_id,
                            bindings.len(),
                        ));
                        Some((plugin_id.clone(), Some(bindings)))
                    }
                    Ok(None) => Some((plugin_id.clone(), None)),
                    Err(error) => {
                        diagnostics.push(
                            discovered_runtime_bridge_method_binding_error_diagnostic(
                                &plugin_id, &error,
                            ),
                        );
                        Some((plugin_id.clone(), None))
                    }
                }
            } else {
                None
            };
            let existing = {
                let mut loaded = lock_loaded_native_plugins(&self.loaded)?;
                let Some(existing) = loaded.get(&key) else {
                    if let Some((binding_plugin_id, bindings)) = bridge_binding_update {
                        self.publish_runtime_bridge_method_bindings_under_loaded_lock_result(
                            &loaded,
                            &binding_plugin_id,
                            bindings,
                        )
                        .map_err(|source| {
                            NativePluginLiveHostLoadingError::RuntimeBridgeMethodBindings {
                                plugin_id: binding_plugin_id,
                                source: Box::new(source),
                            }
                        })?;
                    }
                    loaded.insert(key, plugin);
                    if module_kind == PluginModuleKind::Runtime {
                        self.invalidate_runtime_registration_replay_generation(&plugin_id);
                    }
                    drop(loaded);
                    loaded_plugin_ids.push(plugin_id.clone());
                    continue;
                };
                if let Err(source) = existing.begin_lifecycle_transition() {
                    return Err(NativePluginLiveHostLoadingError::PluginBusy {
                        plugin_id,
                        module_kind,
                        source,
                    });
                }
                existing.clone()
            };
            match diagnostics_from_behavior_report(
                &format!("{} unload before reload", module_kind_label(module_kind)),
                unload_behavior(&existing, module_kind),
            ) {
                Ok(unload_diagnostics) => diagnostics.extend(unload_diagnostics),
                Err(error) => {
                    existing.cancel_lifecycle_transition();
                    return Err(NativePluginLiveHostLoadingError::UnloadBeforeReload {
                        plugin_id,
                        module_kind,
                        source: error,
                    });
                }
            }
            let mut loaded = match lock_loaded_native_plugins(&self.loaded) {
                Ok(loaded) => loaded,
                Err(error) => {
                    existing.cancel_lifecycle_transition();
                    return Err(error);
                }
            };
            if let Some((binding_plugin_id, bindings)) = bridge_binding_update {
                if let Err(source) = self
                    .publish_runtime_bridge_method_bindings_under_loaded_lock_result(
                        &loaded,
                        &binding_plugin_id,
                        bindings,
                    )
                {
                    existing.cancel_lifecycle_transition();
                    return Err(
                        NativePluginLiveHostLoadingError::RuntimeBridgeMethodBindings {
                            plugin_id: binding_plugin_id,
                            source: Box::new(source),
                        },
                    );
                }
            }
            loaded.insert(key, plugin);
            if module_kind == PluginModuleKind::Runtime {
                self.invalidate_runtime_registration_replay_generation(&plugin_id);
            }
            drop(loaded);
            loaded_plugin_ids.push(plugin_id);
        }

        loaded_plugin_ids.sort();
        loaded_plugin_ids.dedup();
        diagnostics.sort();
        diagnostics.dedup();
        Ok(NativePluginLiveHostLoadReport {
            module_kind,
            loaded_plugin_ids,
            runtime_plugin_registration_reports,
            runtime_plugin_feature_registration_reports,
            bridge_lifecycle_reports: Vec::new(),
            diagnostics,
        })
    }
}

pub(super) fn lock_loaded_native_plugins(
    loaded: &ObservedLoadedNativePlugins,
) -> NativePluginLiveHostLoadingResult<MutexGuard<'_, NativePluginLiveRegistry<LoadedNativePlugin>>>
{
    loaded
        .lock()
        .map_err(|_| NativePluginLiveHostLoadingError::LiveHostLockPoisoned)
}
