use std::path::Path;

use crate::plugin::PluginModuleKind;

use super::super::{NativePluginLoadReport, NativePluginLoader};
use super::bridge_methods::{
    discovered_runtime_bridge_method_binding_diagnostics,
    discovered_runtime_bridge_method_binding_error_diagnostic,
    discovered_runtime_bridge_method_bindings,
};
use super::diagnostics::{
    diagnostics_for_plugin, diagnostics_from_behavior_report, load_report_diagnostics,
    unloaded_plugin_error,
};
use super::hot_reload::{restore_runtime_snapshot, NativePluginHotReloadState};
use super::keys::{live_key, module_kind_article_label, module_kind_label};
use super::loading::lock_loaded_native_plugins;
use super::reports::{NativePluginLiveHostCommand, NativePluginLiveHostOutcome};
use super::runtime_behavior::unload_behavior;
use super::NativePluginLiveHost;

impl NativePluginLiveHost {
    pub fn unload_runtime_plugin(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.unload_plugin(plugin_id.as_ref(), PluginModuleKind::Runtime)
    }

    pub fn unload_editor_plugin(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.unload_plugin(plugin_id.as_ref(), PluginModuleKind::Editor)
    }

    pub fn hot_reload_runtime_plugin(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.hot_reload_plugin(root.as_ref(), plugin_id.as_ref(), PluginModuleKind::Runtime)
    }

    pub fn hot_reload_editor_plugin(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.hot_reload_plugin(root.as_ref(), plugin_id.as_ref(), PluginModuleKind::Editor)
    }

    fn unload_plugin(
        &self,
        plugin_id: &str,
        module_kind: PluginModuleKind,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        let mut loaded = lock_loaded_native_plugins(&self.loaded)?;
        let key = live_key(module_kind, plugin_id);
        let Some(plugin) = loaded.remove(&key) else {
            return Err(unloaded_plugin_error(plugin_id, module_kind));
        };
        match diagnostics_from_behavior_report(
            &format!("{} unload", module_kind_label(module_kind)),
            unload_behavior(&plugin, module_kind),
        ) {
            Ok(diagnostics) => {
                drop(loaded);
                if module_kind == PluginModuleKind::Runtime {
                    self.replace_runtime_bridge_method_bindings(plugin_id, None)?;
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
                Err(error)
            }
        }
    }

    fn hot_reload_plugin(
        &self,
        root: &Path,
        plugin_id: &str,
        module_kind: PluginModuleKind,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        let mut loaded = lock_loaded_native_plugins(&self.loaded)?;
        let key = live_key(module_kind, plugin_id);
        let mut diagnostics = Vec::new();
        let existing = loaded.remove(&key);
        let mut reload_state = NativePluginHotReloadState::new(module_kind, key, existing);

        let mut report = load_for_module_kind(&self.loader, root, module_kind)?;
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
            let error = format!(
                "plugin {plugin_id} hot reload did not load {} native package from {}: {diagnostic_hint}",
                module_kind_article_label(module_kind),
                root.display()
            );
            let error = reload_state.rollback_error(error);
            if let Some(existing) = reload_state.into_rollback_plugin() {
                loaded.insert(live_key(module_kind, plugin_id), existing);
            }
            return Err(error);
        };
        if let Err(error) = reload_state.save_existing_runtime_snapshot(plugin_id) {
            if let Some(existing) = reload_state.into_rollback_plugin() {
                loaded.insert(live_key(module_kind, plugin_id), existing);
            }
            return Err(error);
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
                    return Err(error);
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
                        .unwrap_or_else(|unload_error| vec![unload_error]),
                    );
                    if let Some(existing) = unloaded_existing.take() {
                        rollback_diagnostics.extend(
                            restore_runtime_snapshot(snapshot, &existing)
                                .unwrap_or_else(|restore_error| vec![restore_error]),
                        );
                        loaded.insert(reload_state.key.clone(), existing);
                    }
                    return Err(reload_state
                        .rollback_error(format!("{error}; {}", rollback_diagnostics.join("; "))));
                }
            }
        }
        let bridge_binding_update = if module_kind == PluginModuleKind::Runtime {
            match discovered_runtime_bridge_method_bindings(&plugin) {
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
            self.replace_runtime_bridge_method_bindings(plugin_id, bindings)?;
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

fn load_for_module_kind(
    loader: &NativePluginLoader,
    root: &Path,
    module_kind: PluginModuleKind,
) -> Result<NativePluginLoadReport, String> {
    match module_kind {
        PluginModuleKind::Runtime => Ok(loader.load_discovered_runtime(root)),
        PluginModuleKind::Editor => Ok(loader.load_discovered_editor(root)),
        PluginModuleKind::Native | PluginModuleKind::Vm => Err(format!(
            "native plugin live host does not manage {} module handles",
            module_kind_label(module_kind)
        )),
    }
}
