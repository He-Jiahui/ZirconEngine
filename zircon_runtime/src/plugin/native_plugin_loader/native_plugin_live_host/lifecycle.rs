use std::path::Path;

use crate::plugin::PluginModuleKind;

use super::super::loaded_native_plugin::NativePluginLifecycleTransitionError;
use super::super::{NativePluginLoadReport, NativePluginLoader};
use super::bridge_methods::{
    discovered_runtime_bridge_method_binding_diagnostics,
    discovered_runtime_bridge_method_binding_error_diagnostic,
    discovered_runtime_bridge_method_bindings_result, NativePluginBridgeMethodError,
};
use super::diagnostics::{
    diagnostics_from_behavior_report, load_projected_report_diagnostics,
    projected_diagnostics_for_plugin, NativePluginBehaviorDiagnosticError,
};
use super::hot_reload::{
    restore_runtime_snapshot, NativePluginHotReloadState, PluginStateSnapshot,
};
use super::keys::{live_key, module_kind_article_label, module_kind_label};
use super::loading::{lock_loaded_native_plugins, NativePluginLiveHostLoadingError};
use super::reports::{NativePluginLiveHostCommand, NativePluginLiveHostOutcome};
use super::runtime_behavior::unload_behavior;
use super::NativePluginLiveHost;

pub(super) type NativePluginLiveHostLifecycleResult<T> =
    std::result::Result<T, NativePluginLiveHostLifecycleError>;

#[derive(Debug)]
pub(super) enum NativePluginHotReloadPublicationError {
    LiveHostLock(NativePluginLiveHostLoadingError),
    RuntimeBridgeMethodBindings {
        plugin_id: String,
        source: NativePluginBridgeMethodError,
    },
}

impl std::fmt::Display for NativePluginHotReloadPublicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LiveHostLock(error) => write!(formatter, "{error}"),
            Self::RuntimeBridgeMethodBindings { plugin_id, source } => write!(
                formatter,
                "runtime plugin {plugin_id} bridge method binding install failed during hot reload: {source}"
            ),
        }
    }
}

impl std::error::Error for NativePluginHotReloadPublicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LiveHostLock(error) => Some(error),
            Self::RuntimeBridgeMethodBindings { source, .. } => Some(source),
        }
    }
}

#[derive(Debug)]
pub(super) enum NativePluginLiveHostLifecycleError {
    LiveHostLock(NativePluginLiveHostLoadingError),
    RuntimePluginNotLoaded {
        plugin_id: String,
        module_kind: PluginModuleKind,
    },
    PluginBusy {
        plugin_id: String,
        module_kind: PluginModuleKind,
        source: NativePluginLifecycleTransitionError,
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
    HotReloadPublication {
        source: NativePluginHotReloadPublicationError,
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
            Self::PluginBusy {
                plugin_id,
                module_kind,
                source,
            } => write!(
                formatter,
                "plugin {plugin_id} {} lifecycle is busy: {source}",
                module_kind_label(*module_kind)
            ),
            Self::UnloadBehavior { source, .. } => write!(formatter, "{source}"),
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
            Self::HotReloadUnloadBeforeReload { source, .. } => write!(formatter, "{source}"),
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
            Self::HotReloadPublication {
                source,
                rollback_diagnostics,
                rollback_diagnostic,
            } => {
                write!(formatter, "{source}")?;
                if !rollback_diagnostics.is_empty() {
                    write!(formatter, "; {}", rollback_diagnostics.join("; "))?;
                }
                write!(formatter, "; {rollback_diagnostic}")
            }
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
            Self::HotReloadPublication { source, .. } => Some(source),
            Self::RuntimeBridgeMethodBindings { source, .. } => Some(source),
            Self::PluginBusy { source, .. } => Some(source),
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
        let key = live_key(module_kind, plugin_id);
        let plugin = {
            let loaded = lock_loaded_native_plugins(&self.loaded)
                .map_err(NativePluginLiveHostLifecycleError::LiveHostLock)?;
            let Some(plugin) = loaded.get(&key) else {
                return Err(NativePluginLiveHostLifecycleError::RuntimePluginNotLoaded {
                    plugin_id: plugin_id.to_string(),
                    module_kind,
                });
            };
            if let Err(source) = plugin.begin_lifecycle_transition() {
                return Err(NativePluginLiveHostLifecycleError::PluginBusy {
                    plugin_id: plugin_id.to_string(),
                    module_kind,
                    source,
                });
            }
            plugin.clone()
        };
        match diagnostics_from_behavior_report(
            &format!("{} unload", module_kind_label(module_kind)),
            unload_behavior(&plugin, module_kind),
        ) {
            Ok(diagnostics) => {
                let mut loaded = match lock_loaded_native_plugins(&self.loaded) {
                    Ok(loaded) => loaded,
                    Err(error) => {
                        plugin.cancel_lifecycle_transition();
                        return Err(NativePluginLiveHostLifecycleError::LiveHostLock(error));
                    }
                };
                loaded.remove(&key);
                if module_kind == PluginModuleKind::Runtime {
                    self.publish_runtime_bridge_method_bindings_under_loaded_lock_result(
                        &loaded, plugin_id, None,
                    )
                    .map_err(|source| {
                        NativePluginLiveHostLifecycleError::RuntimeBridgeMethodBindings {
                            plugin_id: plugin_id.to_string(),
                            source,
                        }
                    })?;
                }
                if module_kind == PluginModuleKind::Runtime {
                    self.invalidate_runtime_registration_replay_generation(plugin_id);
                }
                drop(loaded);
                Ok(NativePluginLiveHostOutcome {
                    plugin_id: plugin_id.to_string(),
                    module_kind,
                    command: NativePluginLiveHostCommand::Unload,
                    bridge_lifecycle_report: None,
                    diagnostics,
                })
            }
            Err(error) => {
                plugin.cancel_lifecycle_transition();
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
        report: NativePluginLoadReport,
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
        let key = live_key(module_kind, plugin_id);
        let mut diagnostics = Vec::new();
        let existing = {
            let loaded = lock_loaded_native_plugins(&self.loaded)
                .map_err(NativePluginLiveHostLifecycleError::LiveHostLock)?;
            let existing = loaded.get(&key);
            if let Some(existing_plugin) = existing {
                if let Err(source) = existing_plugin.begin_lifecycle_transition() {
                    return Err(NativePluginLiveHostLifecycleError::PluginBusy {
                        plugin_id: plugin_id.to_string(),
                        module_kind,
                        source,
                    });
                }
            }
            existing.cloned()
        };
        // The registry borrows `key` only for lookup. The reload transition can outlive that
        // borrow, so it owns the plugin id and reconstructs the typed lookup key at reinsertion.
        let mut reload_state =
            NativePluginHotReloadState::new(module_kind, plugin_id.to_owned(), existing);

        let projection = report.projection();
        diagnostics.extend(load_projected_report_diagnostics(&report, &projection));
        diagnostics.extend(projected_diagnostics_for_plugin(
            &projection,
            plugin_id,
            module_kind,
        ));
        diagnostics.sort();
        diagnostics.dedup();
        let mut reloaded = None;
        for plugin in report.take_loaded() {
            if plugin.plugin_id == plugin_id {
                reloaded = Some(plugin);
            }
        }
        let Some(plugin) = reloaded else {
            let discovery_hint = native_plugin_discovery_hint(
                report
                    .discovered()
                    .iter()
                    .map(|candidate| candidate.plugin_id.as_str()),
            );
            let diagnostic_hint = if diagnostics.is_empty() {
                discovery_hint
            } else {
                format!("{discovery_hint}; {}", diagnostics.join("; "))
            };
            let rollback_diagnostic = reload_state.rollback_diagnostic();
            if let Some(existing) = reload_state.into_rollback_plugin() {
                existing.cancel_lifecycle_transition();
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
                existing.cancel_lifecycle_transition();
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
                    existing.cancel_lifecycle_transition();
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
        let runtime_snapshot = reload_state.take_runtime_snapshot();
        if let Some(snapshot) = runtime_snapshot.as_ref() {
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
                        match restore_runtime_snapshot(snapshot, &existing) {
                            Ok(restore_diagnostics) => {
                                existing.cancel_lifecycle_transition();
                                rollback_diagnostics.extend(restore_diagnostics);
                                reload_state.mark_existing_restored();
                            }
                            Err(restore_error) => {
                                rollback_diagnostics.push(restore_error.to_string());
                            }
                        }
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
                        plugin_id,
                        bindings.len(),
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
        let mut loaded = match lock_loaded_native_plugins(&self.loaded) {
            Ok(loaded) => loaded,
            Err(error) => {
                let had_unloaded_existing = unloaded_existing.is_some();
                let mut rollback_diagnostics =
                    unload_replacement_after_failed_publication(&plugin, module_kind);
                match rollback_unloaded_existing_runtime_snapshot(
                    runtime_snapshot.as_ref(),
                    unloaded_existing.as_ref(),
                ) {
                    Ok(()) => {
                        if had_unloaded_existing {
                            reload_state.mark_existing_restored();
                        }
                        return Err(NativePluginLiveHostLifecycleError::HotReloadPublication {
                            source: NativePluginHotReloadPublicationError::LiveHostLock(error),
                            rollback_diagnostics,
                            rollback_diagnostic: reload_state.rollback_diagnostic(),
                        });
                    }
                    Err(source) => {
                        rollback_diagnostics.push(format!(
                            "native plugin live host publication failed before the retained generation could be restored: {error}"
                        ));
                        return Err(NativePluginLiveHostLifecycleError::HotReloadRestore {
                            source,
                            rollback_diagnostics,
                            rollback_diagnostic: reload_state.rollback_diagnostic(),
                        });
                    }
                }
            }
        };
        if let Some(bindings) = bridge_binding_update {
            if let Err(source) = self
                .publish_runtime_bridge_method_bindings_under_loaded_lock_result(
                    &loaded, plugin_id, bindings,
                )
            {
                drop(loaded);
                let had_unloaded_existing = unloaded_existing.is_some();
                let mut rollback_diagnostics =
                    unload_replacement_after_failed_publication(&plugin, module_kind);
                match rollback_unloaded_existing_runtime_snapshot(
                    runtime_snapshot.as_ref(),
                    unloaded_existing.as_ref(),
                ) {
                    Ok(()) => {
                        if had_unloaded_existing {
                            reload_state.mark_existing_restored();
                        }
                        return Err(NativePluginLiveHostLifecycleError::HotReloadPublication {
                            source:
                                NativePluginHotReloadPublicationError::RuntimeBridgeMethodBindings {
                                    plugin_id: plugin_id.to_string(),
                                    source,
                                },
                            rollback_diagnostics,
                            rollback_diagnostic: reload_state.rollback_diagnostic(),
                        });
                    }
                    Err(restore_error) => {
                        rollback_diagnostics.push(format!(
                            "runtime bridge method binding publication failed before the retained generation could be restored: {source}"
                        ));
                        return Err(NativePluginLiveHostLifecycleError::HotReloadRestore {
                            source: restore_error,
                            rollback_diagnostics,
                            rollback_diagnostic: reload_state.rollback_diagnostic(),
                        });
                    }
                }
            }
        }
        loaded.insert(
            live_key(reload_state.module_kind, &reload_state.key),
            plugin,
        );
        if module_kind == PluginModuleKind::Runtime {
            self.invalidate_runtime_registration_replay_generation(plugin_id);
        }
        drop(loaded);
        Ok(NativePluginLiveHostOutcome {
            plugin_id: plugin_id.to_string(),
            module_kind,
            command: NativePluginLiveHostCommand::HotReload,
            bridge_lifecycle_report: None,
            diagnostics,
        })
    }
}

fn native_plugin_discovery_hint<'a>(plugin_ids: impl Iterator<Item = &'a str> + Clone) -> String {
    const PREFIX: &str = "discovered native plugins: ";
    const SEPARATOR: &str = ", ";

    let mut plugin_ids = plugin_ids.peekable();
    if plugin_ids.peek().is_none() {
        return "no native plugin manifests were discovered".to_string();
    }
    let plugin_count = plugin_ids.clone().count();
    let capacity = PREFIX.len()
        + plugin_ids.clone().map(str::len).sum::<usize>()
        + plugin_count
            .saturating_sub(1)
            .saturating_mul(SEPARATOR.len());
    let mut hint = String::with_capacity(capacity);
    hint.push_str(PREFIX);
    for (index, plugin_id) in plugin_ids.enumerate() {
        if index != 0 {
            hint.push_str(SEPARATOR);
        }
        hint.push_str(plugin_id);
    }
    hint
}

#[cfg(test)]
#[path = "lifecycle/discovery_hint_tests.rs"]
mod discovery_hint_tests;

// A retained generation may resume callback admission only after its saved state restores.
fn rollback_unloaded_existing_runtime_snapshot(
    runtime_snapshot: Option<&PluginStateSnapshot>,
    unloaded_existing: Option<&super::super::LoadedNativePlugin>,
) -> super::hot_reload::NativePluginHotReloadResult<()> {
    let Some(existing) = unloaded_existing else {
        return Ok(());
    };
    if let Some(snapshot) = runtime_snapshot {
        restore_runtime_snapshot(snapshot, existing)?;
    }
    existing.cancel_lifecycle_transition();
    Ok(())
}

fn unload_replacement_after_failed_publication(
    plugin: &super::super::LoadedNativePlugin,
    module_kind: PluginModuleKind,
) -> Vec<String> {
    diagnostics_from_behavior_report(
        &format!(
            "{} unload after failed hot reload publication",
            module_kind_label(module_kind)
        ),
        unload_behavior(plugin, module_kind),
    )
    .unwrap_or_else(|unload_error| vec![unload_error.to_string()])
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
