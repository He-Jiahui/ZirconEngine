use std::path::Path;

use crate::plugin::PluginModuleKind;

use super::super::{NativePluginLoadReport, NativePluginLoader};
use super::bridge_methods::{
    discovered_runtime_bridge_method_binding_diagnostics,
    discovered_runtime_bridge_method_binding_error_diagnostic,
    discovered_runtime_bridge_method_bindings_result, NativePluginBridgeMethodError,
};
use super::diagnostics::{
    diagnostics_for_plugin, diagnostics_from_behavior_report, load_report_diagnostics,
    NativePluginBehaviorDiagnosticError,
};
use super::hot_reload::{restore_runtime_snapshot, NativePluginHotReloadState};
use super::keys::{live_key, module_kind_article_label, module_kind_label};
use super::loading::{lock_loaded_native_plugins, NativePluginLiveHostLoadingError};
use super::reports::{NativePluginLiveHostCommand, NativePluginLiveHostOutcome};
use super::runtime_behavior::unload_behavior;
use super::NativePluginLiveHost;

pub(super) type NativePluginLiveHostLifecycleResult<T> =
    std::result::Result<T, NativePluginLiveHostLifecycleError>;

#[derive(Debug)]
pub(super) enum NativePluginLiveHostLifecycleError {
    LiveHostLock(NativePluginLiveHostLoadingError),
    RuntimePluginNotLoaded {
        plugin_id: String,
        module_kind: PluginModuleKind,
    },
    UnloadBehavior {
        plugin_id: String,
        module_kind: PluginModuleKind,
        source: NativePluginBehaviorDiagnosticError,
    },
    HotReloadDidNotLoad {
        plugin_id: String,
        module_kind: PluginModuleKind,
        root: std::path::PathBuf,
        diagnostic_hint: String,
        rollback_diagnostic: String,
    },
    HotReloadSnapshot {
        source: super::hot_reload::NativePluginHotReloadError,
    },
    HotReloadUnloadBeforeReload {
        plugin_id: String,
        module_kind: PluginModuleKind,
        source: NativePluginBehaviorDiagnosticError,
    },
    HotReloadRestore {
        source: super::hot_reload::NativePluginHotReloadError,
        rollback_diagnostics: Vec<String>,
        rollback_diagnostic: String,
    },
    RuntimeBridgeMethodBindings {
        plugin_id: String,
        source: NativePluginBridgeMethodError,
    },
    UnsupportedLiveHostModuleKind {
        module_kind: PluginModuleKind,
    },
}

impl std::fmt::Display for NativePluginLiveHostLifecycleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LiveHostLock(error) => write!(formatter, "{error}"),
            Self::RuntimePluginNotLoaded {
                plugin_id,
                module_kind,
            } => write!(
                formatter,
                "plugin {plugin_id} is not loaded in the {} live host; run Hot Reload after building its native dynamic package",
                module_kind_label(*module_kind)
            ),
            Self::UnloadBehavior {
                source,
                ..
            } => write!(formatter, "{source}"),
            Self::HotReloadDidNotLoad {
                plugin_id,
                module_kind,
                root,
                diagnostic_hint,
                rollback_diagnostic,
            } => write!(
                formatter,
                "plugin {plugin_id} hot reload did not load {} native package from {}: {diagnostic_hint}; {rollback_diagnostic}",
                module_kind_article_label(*module_kind),
                root.display()
            ),
            Self::HotReloadSnapshot { source } => write!(formatter, "{source}"),
            Self::HotReloadUnloadBeforeReload {
                source,
                ..
            } => write!(formatter, "{source}"),
            Self::HotReloadRestore {
                source,
                rollback_diagnostics,
                rollback_diagnostic,
            } => write!(
                formatter,
                "{}; {}; {rollback_diagnostic}",
                source,
                rollback_diagnostics.join("; ")
            ),
            Self::RuntimeBridgeMethodBindings { plugin_id, source } => write!(
                formatter,
                "runtime plugin {plugin_id} bridge method binding install failed during hot reload: {source}"
            ),
            Self::UnsupportedLiveHostModuleKind { module_kind } => write!(
                formatter,
                "native plugin live host does not manage {} module handles",
                module_kind_label(*module_kind)
            ),
        }
    }
}

impl std::error::Error for NativePluginLiveHostLifecycleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LiveHostLock(error) => Some(error),
            Self::HotReloadSnapshot { source } | Self::HotReloadRestore { source, .. } => {
                Some(source)
            }
            Self::RuntimeBridgeMethodBindings { source, .. } => Some(source),
            Self::UnloadBehavior { source, .. }
            | Self::HotReloadUnloadBeforeReload { source, .. } => Some(source),
            Self::RuntimePluginNotLoaded { .. }
            | Self::HotReloadDidNotLoad { .. }
            | Self::UnsupportedLiveHostModuleKind { .. } => None,
        }
    }
}

impl NativePluginLiveHost {
    pub fn unload_runtime_plugin(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.unload_plugin_result(plugin_id.as_ref(), PluginModuleKind::Runtime)
            .map_err(|error| error.to_string())
    }

    pub fn unload_editor_plugin(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.unload_plugin_result(plugin_id.as_ref(), PluginModuleKind::Editor)
            .map_err(|error| error.to_string())
    }

    pub fn hot_reload_runtime_plugin(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.hot_reload_plugin_result(root.as_ref(), plugin_id.as_ref(), PluginModuleKind::Runtime)
            .map_err(|error| error.to_string())
    }

    pub fn hot_reload_editor_plugin(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.hot_reload_plugin_result(root.as_ref(), plugin_id.as_ref(), PluginModuleKind::Editor)
            .map_err(|error| error.to_string())
    }

    pub(super) fn unload_plugin_result(
        &self,
        plugin_id: &str,
        module_kind: PluginModuleKind,
    ) -> NativePluginLiveHostLifecycleResult<NativePluginLiveHostOutcome> {
        let mut loaded = lock_loaded_native_plugins(&self.loaded)
            .map_err(NativePluginLiveHostLifecycleError::LiveHostLock)?;
        let key = live_key(module_kind, plugin_id);
        let Some(plugin) = loaded.remove(&key) else {
            return Err(NativePluginLiveHostLifecycleError::RuntimePluginNotLoaded {
                plugin_id: plugin_id.to_string(),
                module_kind,
            });
        };
        match diagnostics_from_behavior_report(
            &format!("{} unload", module_kind_label(module_kind)),
            unload_behavior(&plugin, module_kind),
        ) {
            Ok(diagnostics) => {
                drop(loaded);
                if module_kind == PluginModuleKind::Runtime {
                    self.replace_runtime_bridge_method_bindings_result(plugin_id, None)
                        .map_err(|source| {
                            NativePluginLiveHostLifecycleError::RuntimeBridgeMethodBindings {
                                plugin_id: plugin_id.to_string(),
                                source,
                            }
                        })?;
                }
                Ok(NativePluginLiveHostOutcome {
                    plugin_id: plugin_id.to_string(),
                    module_kind,
                    command: NativePluginLiveHostCommand::Unload,
                    bridge_lifecycle_report: None,
                    diagnostics,
                })
            }
            Err(error) => {
                loaded.insert(key, plugin);
                Err(NativePluginLiveHostLifecycleError::UnloadBehavior {
                    plugin_id: plugin_id.to_string(),
                    module_kind,
                    source: error,
                })
            }
        }
    }

    fn hot_reload_plugin_result(
        &self,
        root: &Path,
        plugin_id: &str,
        module_kind: PluginModuleKind,
    ) -> NativePluginLiveHostLifecycleResult<NativePluginLiveHostOutcome> {
        let report = load_for_module_kind(&self.loader, root, module_kind)?;
        self.hot_reload_reported_plugin_result(report, root, plugin_id, module_kind)
    }

    pub(super) fn hot_reload_reported_plugin(
        &self,
        mut report: NativePluginLoadReport,
        root: &Path,
        plugin_id: &str,
        module_kind: PluginModuleKind,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.hot_reload_reported_plugin_result(report, root, plugin_id, module_kind)
            .map_err(|error| error.to_string())
    }

    pub(super) fn hot_reload_reported_plugin_result(
        &self,
        mut report: NativePluginLoadReport,
        root: &Path,
        plugin_id: &str,
        module_kind: PluginModuleKind,
    ) -> NativePluginLiveHostLifecycleResult<NativePluginLiveHostOutcome> {
        let mut loaded = lock_loaded_native_plugins(&self.loaded)
            .map_err(NativePluginLiveHostLifecycleError::LiveHostLock)?;
        let key = live_key(module_kind, plugin_id);
        let mut diagnostics = Vec::new();
        let existing = loaded.remove(&key);
        let mut reload_state = NativePluginHotReloadState::new(module_kind, key, existing);

        diagnostics.extend(load_report_diagnostics(&report));
        diagnostics.extend(diagnostics_for_plugin(&report, plugin_id, module_kind));
        diagnostics.sort();
        diagnostics.dedup();
        let mut reloaded = None;
        for plugin in std::mem::take(&mut report.loaded) {
            if plugin.plugin_id == plugin_id {
                reloaded = Some(plugin);
            }
        }
        let Some(plugin) = reloaded else {
            let discovered = report
                .discovered
                .iter()
                .map(|candidate| candidate.plugin_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let discovery_hint = if discovered.is_empty() {
                "no native plugin manifests were discovered".to_string()
            } else {
                format!("discovered native plugins: {discovered}")
            };
            let diagnostic_hint = if diagnostics.is_empty() {
                discovery_hint
            } else {
                format!("{discovery_hint}; {}", diagnostics.join("; "))
            };
            let rollback_diagnostic = reload_state.rollback_diagnostic();
            if let Some(existing) = reload_state.into_rollback_plugin() {
                loaded.insert(live_key(module_kind, plugin_id), existing);
            }
            return Err(NativePluginLiveHostLifecycleError::HotReloadDidNotLoad {
                plugin_id: plugin_id.to_string(),
                module_kind,
                root: root.to_path_buf(),
                diagnostic_hint,
                rollback_diagnostic,
            });
        };
        if let Err(error) = reload_state.save_existing_runtime_snapshot(plugin_id) {
            if let Some(existing) = reload_state.into_rollback_plugin() {
                loaded.insert(live_key(module_kind, plugin_id), existing);
            }
            return Err(NativePluginLiveHostLifecycleError::HotReloadSnapshot { source: error });
        }
        let mut unloaded_existing = None;
        if let Some(existing) = reload_state.take_existing_for_unload() {
            match diagnostics_from_behavior_report(
                &format!(
                    "{} unload before hot reload",
                    module_kind_label(module_kind)
                ),
                unload_behavior(&existing, module_kind),
            ) {
                Ok(unload_diagnostics) => {
                    diagnostics.extend(unload_diagnostics.clone());
                    reload_state.mark_existing_unloaded(unload_diagnostics);
                    unloaded_existing = Some(existing);
                }
                Err(error) => {
                    loaded.insert(reload_state.key.clone(), existing);
                    return Err(
                        NativePluginLiveHostLifecycleError::HotReloadUnloadBeforeReload {
                            plugin_id: plugin_id.to_string(),
                            module_kind,
                            source: error,
                        },
                    );
                }
            }
        }
        if let Some(snapshot) = reload_state.runtime_snapshot() {
            match restore_runtime_snapshot(snapshot, &plugin) {
                Ok(restore_diagnostics) => diagnostics.extend(restore_diagnostics),
                Err(error) => {
                    let mut rollback_diagnostics = Vec::new();
                    rollback_diagnostics.extend(
                        diagnostics_from_behavior_report(
                            &format!(
                                "{} unload after failed hot reload",
                                module_kind_label(module_kind)
                            ),
                            unload_behavior(&plugin, module_kind),
                        )
                        .unwrap_or_else(|unload_error| vec![unload_error.to_string()]),
                    );
                    if let Some(existing) = unloaded_existing.take() {
                        rollback_diagnostics.extend(
                            restore_runtime_snapshot(snapshot, &existing)
                                .unwrap_or_else(|restore_error| vec![restore_error.to_string()]),
                        );
                        loaded.insert(reload_state.key.clone(), existing);
                        reload_state.mark_existing_restored();
                    }
                    return Err(NativePluginLiveHostLifecycleError::HotReloadRestore {
                        source: error,
                        rollback_diagnostics,
                        rollback_diagnostic: reload_state.rollback_diagnostic(),
                    });
                }
            }
        }
        let bridge_binding_update = if module_kind == PluginModuleKind::Runtime {
            match discovered_runtime_bridge_method_bindings_result(&plugin) {
                Ok(Some(bindings)) => {
                    diagnostics.push(discovered_runtime_bridge_method_binding_diagnostics(
                        plugin_id, &bindings,
                    ));
                    Some(Some(bindings))
                }
                Ok(None) => Some(None),
                Err(error) => {
                    diagnostics.push(discovered_runtime_bridge_method_binding_error_diagnostic(
                        plugin_id, &error,
                    ));
                    Some(None)
                }
            }
        } else {
            None
        };
        loaded.insert(reload_state.key, plugin);
        drop(loaded);
        if let Some(bindings) = bridge_binding_update {
            self.replace_runtime_bridge_method_bindings_result(plugin_id, bindings)
                .map_err(|source| {
                    NativePluginLiveHostLifecycleError::RuntimeBridgeMethodBindings {
                        plugin_id: plugin_id.to_string(),
                        source,
                    }
                })?;
        }
        Ok(NativePluginLiveHostOutcome {
            plugin_id: plugin_id.to_string(),
            module_kind,
            command: NativePluginLiveHostCommand::HotReload,
            bridge_lifecycle_report: None,
            diagnostics,
        })
    }
}

pub(super) fn load_for_module_kind(
    loader: &NativePluginLoader,
    root: &Path,
    module_kind: PluginModuleKind,
) -> NativePluginLiveHostLifecycleResult<NativePluginLoadReport> {
    match module_kind {
        PluginModuleKind::Runtime => Ok(loader.load_discovered_runtime(root)),
        PluginModuleKind::Editor => Ok(loader.load_discovered_editor(root)),
        PluginModuleKind::Native | PluginModuleKind::Vm => {
            Err(NativePluginLiveHostLifecycleError::UnsupportedLiveHostModuleKind { module_kind })
        }
    }
}
