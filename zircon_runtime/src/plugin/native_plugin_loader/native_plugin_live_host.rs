use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use crate::plugin::PluginModuleKind;

use super::super::runtime_plugin::{
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
use super::{
    LoadedNativePlugin, NativePluginBehaviorCallReport, NativePluginLoadReport, NativePluginLoader,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};

pub const NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND: &str = "play-mode.enter";
pub const NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND: &str = "play-mode.exit";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePluginLiveHostCommand {
    Unload,
    HotReload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginLiveHostOutcome {
    pub plugin_id: String,
    pub module_kind: PluginModuleKind,
    pub command: NativePluginLiveHostCommand,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct NativePluginLiveHostLoadReport {
    pub module_kind: PluginModuleKind,
    pub loaded_plugin_ids: Vec<String>,
    pub runtime_plugin_registration_reports: Vec<RuntimePluginRegistrationReport>,
    pub runtime_plugin_feature_registration_reports: Vec<RuntimePluginFeatureRegistrationReport>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeBehaviorDescriptor {
    pub plugin_id: String,
    pub is_stateless: Option<bool>,
    pub state_schema_version: Option<u32>,
    pub command_manifest_schema: Option<String>,
    pub event_manifest_schema: Option<String>,
    pub command_manifest: Option<String>,
    pub event_manifest: Option<String>,
}

#[derive(Debug)]
struct NativePluginHotReloadState {
    module_kind: PluginModuleKind,
    key: String,
    existing: Option<LoadedNativePlugin>,
    previous_unloaded: bool,
    diagnostics: Vec<String>,
}

impl NativePluginHotReloadState {
    fn new(
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
        }
    }

    fn take_existing_for_unload(&mut self) -> Option<LoadedNativePlugin> {
        self.existing.take()
    }

    fn mark_existing_unloaded(&mut self, diagnostics: Vec<String>) {
        self.previous_unloaded = true;
        self.diagnostics.extend(diagnostics);
    }

    fn rollback_error(&mut self, error: String) -> String {
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

    fn into_rollback_plugin(self) -> Option<LoadedNativePlugin> {
        self.existing
    }
}

/// One ABI v2 runtime behavior callback result tied to its native package id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeBehaviorCall {
    pub plugin_id: String,
    pub report: NativePluginBehaviorCallReport,
}

impl NativePluginRuntimeBehaviorCall {
    pub fn is_success(&self) -> bool {
        self.report.status_code == ZIRCON_NATIVE_PLUGIN_STATUS_OK
    }
}

/// Aggregate report for broadcasting a command to every loaded runtime native plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeCommandDispatchReport {
    pub command_name: String,
    pub calls: Vec<NativePluginRuntimeBehaviorCall>,
    pub diagnostics: Vec<String>,
}

impl NativePluginRuntimeCommandDispatchReport {
    pub fn failed_call_count(&self) -> usize {
        self.calls.iter().filter(|call| !call.is_success()).count()
    }

    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty() && self.failed_call_count() == 0
    }

    pub fn combined_diagnostics(&self) -> Vec<String> {
        let mut diagnostics = self.diagnostics.clone();
        for call in &self.calls {
            diagnostics.extend(report_diagnostics(
                &call.plugin_id,
                &self.command_name,
                &call.report,
            ));
        }
        sorted_unique_diagnostics(diagnostics)
    }
}

/// Serialized state captured from one loaded runtime native plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimePluginState {
    pub plugin_id: String,
    pub state: Vec<u8>,
}

/// Play-mode friendly snapshot of every stateful loaded runtime native plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeStateSnapshot {
    pub plugin_states: Vec<NativePluginRuntimePluginState>,
    pub diagnostics: Vec<String>,
}

impl NativePluginRuntimeStateSnapshot {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn combined_diagnostics(&self) -> Vec<String> {
        sorted_unique_diagnostics(self.diagnostics.clone())
    }
}

/// Aggregate restore report for a previously captured native runtime snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeStateRestoreReport {
    pub calls: Vec<NativePluginRuntimeBehaviorCall>,
    pub skipped_plugin_ids: Vec<String>,
    pub diagnostics: Vec<String>,
}

impl NativePluginRuntimeStateRestoreReport {
    pub fn failed_call_count(&self) -> usize {
        self.calls.iter().filter(|call| !call.is_success()).count()
    }

    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
            && self.skipped_plugin_ids.is_empty()
            && self.failed_call_count() == 0
    }

    pub fn combined_diagnostics(&self) -> Vec<String> {
        let mut diagnostics = self.diagnostics.clone();
        for call in &self.calls {
            diagnostics.extend(report_diagnostics(
                &call.plugin_id,
                "restore-state",
                &call.report,
            ));
        }
        sorted_unique_diagnostics(diagnostics)
    }
}

/// Native runtime plugin state captured when editor play mode begins.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimePlayModeSnapshot {
    pub state_snapshot: NativePluginRuntimeStateSnapshot,
    pub enter_report: NativePluginRuntimeCommandDispatchReport,
}

impl NativePluginRuntimePlayModeSnapshot {
    pub fn is_clean(&self) -> bool {
        self.state_snapshot.is_clean() && self.enter_report.is_clean()
    }

    pub fn combined_diagnostics(&self) -> Vec<String> {
        combine_diagnostics([
            self.state_snapshot.combined_diagnostics(),
            self.enter_report.combined_diagnostics(),
        ])
    }
}

/// Exit report that pairs the play-mode exit broadcast with state restoration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimePlayModeExitReport {
    pub exit_report: NativePluginRuntimeCommandDispatchReport,
    pub restore_report: NativePluginRuntimeStateRestoreReport,
}

impl NativePluginRuntimePlayModeExitReport {
    pub fn is_clean(&self) -> bool {
        self.exit_report.is_clean() && self.restore_report.is_clean()
    }

    pub fn combined_diagnostics(&self) -> Vec<String> {
        combine_diagnostics([
            self.exit_report.combined_diagnostics(),
            self.restore_report.combined_diagnostics(),
        ])
    }
}

#[derive(Debug, Default)]
pub struct NativePluginLiveHost {
    loader: NativePluginLoader,
    loaded: Mutex<BTreeMap<String, LoadedNativePlugin>>,
}

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

    pub fn load_runtime_plugins_from_export_root(
        &self,
        export_root: impl AsRef<Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_runtime_from_load_manifest(export_root);
        self.load_reported_plugins(report, PluginModuleKind::Runtime)
    }

    pub fn load_editor_plugins_from_export_root(
        &self,
        export_root: impl AsRef<Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_editor_from_load_manifest(export_root);
        self.load_reported_plugins(report, PluginModuleKind::Editor)
    }

    pub fn load_runtime_plugins_from_project_root(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_discovered_runtime(root);
        self.load_reported_plugins(report, PluginModuleKind::Runtime)
    }

    pub fn load_editor_plugins_from_project_root(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_discovered_editor(root);
        self.load_reported_plugins(report, PluginModuleKind::Editor)
    }

    pub fn loaded_plugin_ids(&self, module_kind: PluginModuleKind) -> Result<Vec<String>, String> {
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let prefix = live_key_prefix(module_kind);
        Ok(loaded
            .keys()
            .filter_map(|key| key.strip_prefix(prefix))
            .map(str::to_string)
            .collect())
    }

    pub fn runtime_behavior_descriptor(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginRuntimeBehaviorDescriptor, String> {
        let plugin_id = plugin_id.as_ref();
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| unloaded_plugin_error(plugin_id, PluginModuleKind::Runtime))?;
        Ok(runtime_behavior_descriptor(plugin_id, plugin))
    }

    pub fn runtime_behavior_descriptors(
        &self,
    ) -> Result<Vec<NativePluginRuntimeBehaviorDescriptor>, String> {
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        Ok(runtime_plugins(&loaded)
            .map(|(plugin_id, plugin)| runtime_behavior_descriptor(&plugin_id, plugin))
            .collect())
    }

    pub fn invoke_runtime_plugin_command(
        &self,
        plugin_id: impl AsRef<str>,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        let plugin_id = plugin_id.as_ref();
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| unloaded_plugin_error(plugin_id, PluginModuleKind::Runtime))?;
        Ok(plugin.invoke_runtime_command(command_name.as_ref(), payload.as_ref()))
    }

    pub fn dispatch_runtime_plugin_command(
        &self,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> Result<NativePluginRuntimeCommandDispatchReport, String> {
        let command_name = command_name.as_ref();
        let payload = payload.as_ref();
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let mut calls = Vec::new();
        let mut diagnostics = Vec::new();
        for (plugin_id, plugin) in runtime_plugins(&loaded) {
            let report = plugin.invoke_runtime_command(command_name, payload);
            diagnostics.extend(report_diagnostics(&plugin_id, command_name, &report));
            calls.push(NativePluginRuntimeBehaviorCall { plugin_id, report });
        }
        Ok(NativePluginRuntimeCommandDispatchReport {
            command_name: command_name.to_string(),
            calls,
            diagnostics,
        })
    }

    pub fn save_runtime_plugin_state(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        let plugin_id = plugin_id.as_ref();
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| unloaded_plugin_error(plugin_id, PluginModuleKind::Runtime))?;
        Ok(plugin.save_runtime_state())
    }

    pub fn save_runtime_plugin_states(&self) -> Result<NativePluginRuntimeStateSnapshot, String> {
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let mut plugin_states = Vec::new();
        let mut diagnostics = Vec::new();
        for (plugin_id, plugin) in runtime_plugins(&loaded) {
            let report = plugin.save_runtime_state();
            diagnostics.extend(report_diagnostics(&plugin_id, "save-state", &report));
            if report.status_code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
                continue;
            }
            match report.payload {
                Some(state) => plugin_states.push(NativePluginRuntimePluginState {
                    plugin_id,
                    state,
                }),
                None => diagnostics.push(format!(
                    "runtime plugin {plugin_id} save-state returned no payload; treating it as stateless for this snapshot"
                )),
            }
        }
        Ok(NativePluginRuntimeStateSnapshot {
            plugin_states,
            diagnostics,
        })
    }

    pub fn restore_runtime_plugin_state(
        &self,
        plugin_id: impl AsRef<str>,
        state: impl AsRef<[u8]>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        let plugin_id = plugin_id.as_ref();
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| unloaded_plugin_error(plugin_id, PluginModuleKind::Runtime))?;
        Ok(plugin.restore_runtime_state(state.as_ref()))
    }

    pub fn restore_runtime_plugin_states(
        &self,
        snapshot: &NativePluginRuntimeStateSnapshot,
    ) -> Result<NativePluginRuntimeStateRestoreReport, String> {
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let mut calls = Vec::new();
        let mut skipped_plugin_ids = Vec::new();
        let mut diagnostics = Vec::new();
        for plugin_state in &snapshot.plugin_states {
            let Some(plugin) = loaded.get(&live_key(
                PluginModuleKind::Runtime,
                &plugin_state.plugin_id,
            )) else {
                skipped_plugin_ids.push(plugin_state.plugin_id.clone());
                diagnostics.push(unloaded_plugin_error(
                    &plugin_state.plugin_id,
                    PluginModuleKind::Runtime,
                ));
                continue;
            };
            let report = plugin.restore_runtime_state(&plugin_state.state);
            diagnostics.extend(report_diagnostics(
                &plugin_state.plugin_id,
                "restore-state",
                &report,
            ));
            calls.push(NativePluginRuntimeBehaviorCall {
                plugin_id: plugin_state.plugin_id.clone(),
                report,
            });
        }
        Ok(NativePluginRuntimeStateRestoreReport {
            calls,
            skipped_plugin_ids,
            diagnostics,
        })
    }

    pub fn enter_runtime_play_mode(&self) -> Result<NativePluginRuntimePlayModeSnapshot, String> {
        let state_snapshot = self.save_runtime_plugin_states()?;
        let enter_report =
            self.dispatch_runtime_plugin_command(NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND, b"")?;
        Ok(NativePluginRuntimePlayModeSnapshot {
            state_snapshot,
            enter_report,
        })
    }

    pub fn exit_runtime_play_mode(
        &self,
        snapshot: &NativePluginRuntimePlayModeSnapshot,
    ) -> Result<NativePluginRuntimePlayModeExitReport, String> {
        let exit_report =
            self.dispatch_runtime_plugin_command(NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND, b"")?;
        let restore_report = self.restore_runtime_plugin_states(&snapshot.state_snapshot)?;
        Ok(NativePluginRuntimePlayModeExitReport {
            exit_report,
            restore_report,
        })
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
            Ok(diagnostics) => Ok(NativePluginLiveHostOutcome {
                plugin_id: plugin_id.to_string(),
                module_kind,
                command: NativePluginLiveHostCommand::Unload,
                diagnostics,
            }),
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
                }
                Err(error) => {
                    loaded.insert(reload_state.key.clone(), existing);
                    return Err(error);
                }
            }
        }
        loaded.insert(reload_state.key, plugin);
        Ok(NativePluginLiveHostOutcome {
            plugin_id: plugin_id.to_string(),
            module_kind,
            command: NativePluginLiveHostCommand::HotReload,
            diagnostics,
        })
    }

    fn load_reported_plugins(
        &self,
        mut report: NativePluginLoadReport,
        module_kind: PluginModuleKind,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let runtime_plugin_registration_reports = match module_kind {
            PluginModuleKind::Runtime => report.runtime_plugin_registration_reports(),
            PluginModuleKind::Editor | PluginModuleKind::Native | PluginModuleKind::Vm => {
                Vec::new()
            }
        };
        let runtime_plugin_feature_registration_reports = match module_kind {
            PluginModuleKind::Runtime => report.runtime_plugin_feature_registration_reports(),
            PluginModuleKind::Editor | PluginModuleKind::Native | PluginModuleKind::Vm => {
                Vec::new()
            }
        };
        let mut diagnostics = load_report_diagnostics(&report);
        let mut loaded = lock_loaded_native_plugins(&self.loaded)?;
        let mut loaded_plugin_ids = Vec::new();

        for plugin in std::mem::take(&mut report.loaded) {
            let plugin_id = plugin.plugin_id.clone();
            let key = live_key(module_kind, &plugin_id);
            if let Some(existing) = loaded.remove(&key) {
                match diagnostics_from_behavior_report(
                    &format!("{} unload before reload", module_kind_label(module_kind)),
                    unload_behavior(&existing, module_kind),
                ) {
                    Ok(unload_diagnostics) => diagnostics.extend(unload_diagnostics),
                    Err(error) => {
                        loaded.insert(key, existing);
                        return Err(error);
                    }
                }
            }
            loaded.insert(key, plugin);
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
            diagnostics,
        })
    }
}

fn load_report_diagnostics(report: &NativePluginLoadReport) -> Vec<String> {
    let mut diagnostics = report.diagnostics.clone();
    diagnostics.extend(report.descriptor_diagnostics());
    diagnostics.extend(report.entry_diagnostics());
    diagnostics
}

fn lock_loaded_native_plugins(
    loaded: &Mutex<BTreeMap<String, LoadedNativePlugin>>,
) -> Result<MutexGuard<'_, BTreeMap<String, LoadedNativePlugin>>, String> {
    loaded
        .lock()
        .map_err(|_| "native plugin live host lock is poisoned".to_string())
}

fn unloaded_plugin_error(plugin_id: &str, module_kind: PluginModuleKind) -> String {
    format!(
        "plugin {plugin_id} is not loaded in the {} live host; run Hot Reload after building its native dynamic package",
        module_kind_label(module_kind)
    )
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

fn diagnostics_for_plugin(
    report: &NativePluginLoadReport,
    plugin_id: &str,
    module_kind: PluginModuleKind,
) -> Vec<String> {
    match module_kind {
        PluginModuleKind::Runtime => report.diagnostics_for_runtime_plugin(plugin_id),
        PluginModuleKind::Editor => report.diagnostics_for_editor_plugin(plugin_id),
        PluginModuleKind::Native | PluginModuleKind::Vm => report.diagnostics_for_plugin(plugin_id),
    }
}

fn runtime_plugins<'a>(
    loaded: &'a BTreeMap<String, LoadedNativePlugin>,
) -> impl Iterator<Item = (String, &'a LoadedNativePlugin)> + 'a {
    let prefix = live_key_prefix(PluginModuleKind::Runtime);
    loaded.iter().filter_map(move |(key, plugin)| {
        key.strip_prefix(prefix).map(|id| (id.to_string(), plugin))
    })
}

fn runtime_behavior_descriptor(
    plugin_id: &str,
    plugin: &LoadedNativePlugin,
) -> NativePluginRuntimeBehaviorDescriptor {
    NativePluginRuntimeBehaviorDescriptor {
        plugin_id: plugin_id.to_string(),
        is_stateless: plugin.runtime_behavior_is_stateless(),
        state_schema_version: plugin.runtime_state_schema_version(),
        command_manifest_schema: plugin.runtime_command_manifest_schema().map(str::to_string),
        event_manifest_schema: plugin.runtime_event_manifest_schema().map(str::to_string),
        command_manifest: plugin.runtime_command_manifest().map(str::to_string),
        event_manifest: plugin.runtime_event_manifest().map(str::to_string),
    }
}

fn unload_behavior(
    plugin: &LoadedNativePlugin,
    module_kind: PluginModuleKind,
) -> NativePluginBehaviorCallReport {
    let report = match module_kind {
        PluginModuleKind::Runtime => plugin.unload_runtime_behavior(),
        PluginModuleKind::Editor => plugin.unload_editor_behavior(),
        PluginModuleKind::Native | PluginModuleKind::Vm => NativePluginBehaviorCallReport {
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            diagnostics: vec![format!(
                "native plugin live host does not manage {} behavior",
                module_kind_label(module_kind)
            )],
            payload: None,
        },
    };
    allow_missing_unload_callback_to_drop_handle(report)
}

fn allow_missing_unload_callback_to_drop_handle(
    report: NativePluginBehaviorCallReport,
) -> NativePluginBehaviorCallReport {
    if report.status_code == ZIRCON_NATIVE_PLUGIN_STATUS_OK
        || !report.diagnostics.iter().any(|diagnostic| {
            diagnostic == "native plugin runtime behavior is missing"
                || diagnostic == "native plugin editor behavior is missing"
                || diagnostic == "native plugin behavior callback unload is missing"
        })
    {
        return report;
    }

    NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: report.diagnostics,
        payload: None,
    }
}

fn diagnostics_from_behavior_report(
    label: &str,
    report: NativePluginBehaviorCallReport,
) -> Result<Vec<String>, String> {
    if report.status_code == ZIRCON_NATIVE_PLUGIN_STATUS_OK {
        return Ok(report.diagnostics);
    }
    let diagnostics = if report.diagnostics.is_empty() {
        vec![format!("{label} returned status {}", report.status_code)]
    } else {
        report
            .diagnostics
            .into_iter()
            .map(|message| format!("{label}: {message}"))
            .collect()
    };
    Err(diagnostics.join("; "))
}

fn report_diagnostics(
    plugin_id: &str,
    operation: &str,
    report: &NativePluginBehaviorCallReport,
) -> Vec<String> {
    let mut diagnostics = report
        .diagnostics
        .iter()
        .map(|diagnostic| format!("runtime plugin {plugin_id} {operation}: {diagnostic}"))
        .collect::<Vec<_>>();
    if report.status_code != ZIRCON_NATIVE_PLUGIN_STATUS_OK && diagnostics.is_empty() {
        diagnostics.push(format!(
            "runtime plugin {plugin_id} {operation} returned status {}",
            report.status_code
        ));
    }
    diagnostics
}

fn combine_diagnostics<const N: usize>(diagnostic_groups: [Vec<String>; N]) -> Vec<String> {
    sorted_unique_diagnostics(diagnostic_groups.into_iter().flatten().collect::<Vec<_>>())
}

fn sorted_unique_diagnostics(mut diagnostics: Vec<String>) -> Vec<String> {
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn live_key(module_kind: PluginModuleKind, plugin_id: &str) -> String {
    format!("{}{plugin_id}", live_key_prefix(module_kind))
}

fn live_key_prefix(module_kind: PluginModuleKind) -> &'static str {
    match module_kind {
        PluginModuleKind::Runtime => "runtime:",
        PluginModuleKind::Editor => "editor:",
        PluginModuleKind::Native => "native:",
        PluginModuleKind::Vm => "vm:",
    }
}

fn module_kind_label(module_kind: PluginModuleKind) -> &'static str {
    match module_kind {
        PluginModuleKind::Runtime => "runtime",
        PluginModuleKind::Editor => "editor",
        PluginModuleKind::Native => "native",
        PluginModuleKind::Vm => "vm",
    }
}

fn module_kind_article_label(module_kind: PluginModuleKind) -> &'static str {
    match module_kind {
        PluginModuleKind::Runtime => "a runtime",
        PluginModuleKind::Editor => "an editor",
        PluginModuleKind::Native => "a native",
        PluginModuleKind::Vm => "a vm",
    }
}

#[cfg(test)]
#[path = "native_plugin_live_host/tests.rs"]
mod tests;
