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
    pub command_manifest: Option<String>,
    pub event_manifest: Option<String>,
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
        if let Some(existing) = loaded.remove(&key) {
            match diagnostics_from_behavior_report(
                &format!(
                    "{} unload before hot reload",
                    module_kind_label(module_kind)
                ),
                unload_behavior(&existing, module_kind),
            ) {
                Ok(unload_diagnostics) => diagnostics.extend(unload_diagnostics),
                Err(error) => {
                    loaded.insert(key, existing);
                    return Err(error);
                }
            }
        }

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
            return Err(format!(
                "plugin {plugin_id} hot reload did not load {} native package from {}: {diagnostic_hint}",
                module_kind_article_label(module_kind),
                root.display()
            ));
        };
        loaded.insert(key, plugin);
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
mod tests {
    use super::*;

    #[test]
    fn native_live_host_reports_missing_editor_package_on_hot_reload() {
        let project_root = std::env::temp_dir().join(format!(
            "zircon-runtime-missing-native-live-host-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let error = NativePluginLiveHost::default()
            .hot_reload_editor_plugin(&project_root, "physics")
            .unwrap_err();
        assert!(error.contains("plugin physics hot reload did not load an editor native package"));
        assert!(error.contains("native plugin root does not exist"));
    }

    #[test]
    fn native_live_host_reports_unloaded_plugin_by_module_kind() {
        let error = NativePluginLiveHost::default()
            .unload_runtime_plugin("physics")
            .unwrap_err();
        assert_eq!(
            error,
            "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
        );
    }

    #[test]
    fn native_live_host_runtime_behavior_calls_report_unloaded_plugin() {
        let host = NativePluginLiveHost::default();
        let expected =
            "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package";
        assert_eq!(
            host.runtime_behavior_descriptor("physics").unwrap_err(),
            expected
        );
        assert!(host
            .runtime_behavior_descriptors()
            .expect("empty runtime live host should list no descriptors")
            .is_empty());
        assert_eq!(
            host.invoke_runtime_plugin_command("physics", "simulate", b"")
                .unwrap_err(),
            expected
        );
        assert_eq!(
            host.save_runtime_plugin_state("physics").unwrap_err(),
            expected
        );
        assert_eq!(
            host.restore_runtime_plugin_state("physics", b"")
                .unwrap_err(),
            expected
        );
    }

    #[test]
    fn native_live_host_runtime_broadcasts_and_snapshots_empty_when_no_plugins_loaded() {
        let host = NativePluginLiveHost::default();

        let dispatch = host
            .dispatch_runtime_plugin_command("play-mode.enter", b"{}")
            .expect("empty runtime live host should still dispatch as an empty report");
        assert_eq!(dispatch.command_name, "play-mode.enter");
        assert!(dispatch.calls.is_empty());
        assert!(dispatch.diagnostics.is_empty());
        assert!(dispatch.is_clean());
        assert_eq!(dispatch.failed_call_count(), 0);
        assert!(dispatch.combined_diagnostics().is_empty());

        let snapshot = host
            .save_runtime_plugin_states()
            .expect("empty runtime live host should still save an empty snapshot");
        assert!(snapshot.plugin_states.is_empty());
        assert!(snapshot.diagnostics.is_empty());
        assert!(snapshot.is_clean());
        assert!(snapshot.combined_diagnostics().is_empty());

        let restore = host
            .restore_runtime_plugin_states(&snapshot)
            .expect("empty runtime live host should still restore an empty snapshot");
        assert!(restore.calls.is_empty());
        assert!(restore.skipped_plugin_ids.is_empty());
        assert!(restore.diagnostics.is_empty());
        assert!(restore.is_clean());
        assert_eq!(restore.failed_call_count(), 0);
        assert!(restore.combined_diagnostics().is_empty());

        let play_snapshot = host
            .enter_runtime_play_mode()
            .expect("empty runtime live host should still enter play mode");
        assert_eq!(
            play_snapshot.enter_report.command_name,
            NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND
        );
        assert!(play_snapshot.state_snapshot.plugin_states.is_empty());
        assert!(play_snapshot.is_clean());
        assert!(play_snapshot.combined_diagnostics().is_empty());
        let play_exit = host
            .exit_runtime_play_mode(&play_snapshot)
            .expect("empty runtime live host should still exit play mode");
        assert_eq!(
            play_exit.exit_report.command_name,
            NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND
        );
        assert!(play_exit.restore_report.calls.is_empty());
        assert!(play_exit.is_clean());
        assert!(play_exit.combined_diagnostics().is_empty());
    }

    #[test]
    fn native_live_host_runtime_snapshot_restore_reports_unloaded_plugins() {
        let host = NativePluginLiveHost::default();
        let snapshot = NativePluginRuntimeStateSnapshot {
            plugin_states: vec![NativePluginRuntimePluginState {
                plugin_id: "physics".to_string(),
                state: b"state".to_vec(),
            }],
            diagnostics: Vec::new(),
        };

        let restore = host
            .restore_runtime_plugin_states(&snapshot)
            .expect("unloaded plugins should be restore diagnostics, not host failures");
        assert!(restore.calls.is_empty());
        assert_eq!(restore.skipped_plugin_ids, vec!["physics".to_string()]);
        assert!(!restore.is_clean());
        assert_eq!(restore.failed_call_count(), 0);
        assert_eq!(
            restore.diagnostics,
            vec![
                "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
                    .to_string()
            ]
        );
        assert_eq!(restore.combined_diagnostics(), restore.diagnostics);
    }

    #[test]
    fn native_runtime_reports_synthesize_callback_status_diagnostics() {
        let failed_call = NativePluginRuntimeBehaviorCall {
            plugin_id: "physics".to_string(),
            report: NativePluginBehaviorCallReport {
                status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                diagnostics: Vec::new(),
                payload: None,
            },
        };
        let dispatch = NativePluginRuntimeCommandDispatchReport {
            command_name: "simulate".to_string(),
            calls: vec![failed_call.clone()],
            diagnostics: Vec::new(),
        };
        assert!(!dispatch.is_clean());
        assert_eq!(dispatch.failed_call_count(), 1);
        assert_eq!(
            dispatch.combined_diagnostics(),
            vec!["runtime plugin physics simulate returned status 1".to_string()]
        );

        let restore = NativePluginRuntimeStateRestoreReport {
            calls: vec![failed_call],
            skipped_plugin_ids: Vec::new(),
            diagnostics: Vec::new(),
        };
        assert!(!restore.is_clean());
        assert_eq!(restore.failed_call_count(), 1);
        assert_eq!(
            restore.combined_diagnostics(),
            vec!["runtime plugin physics restore-state returned status 1".to_string()]
        );
    }

    #[test]
    fn native_live_host_loads_runtime_export_diagnostics_without_handles() {
        let export_root = std::env::temp_dir().join(format!(
            "zircon-runtime-missing-native-live-host-export-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let report = NativePluginLiveHost::default()
            .load_runtime_plugins_from_export_root(&export_root)
            .expect("missing manifest should be reported as diagnostics, not a host failure");
        assert_eq!(report.module_kind, PluginModuleKind::Runtime);
        assert!(report.loaded_plugin_ids.is_empty());
        assert!(report.runtime_plugin_registration_reports.is_empty());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("failed to read native plugin load manifest")));
    }

    #[test]
    fn native_live_host_treats_missing_unload_hook_as_noop_unload() {
        let report = allow_missing_unload_callback_to_drop_handle(NativePluginBehaviorCallReport {
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            diagnostics: vec!["native plugin behavior callback unload is missing".to_string()],
            payload: None,
        });
        assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(
            report.diagnostics,
            vec!["native plugin behavior callback unload is missing".to_string()]
        );
    }
}
