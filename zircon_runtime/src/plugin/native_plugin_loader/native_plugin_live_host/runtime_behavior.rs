use std::collections::BTreeMap;

use crate::plugin::PluginModuleKind;

use super::super::{
    LoadedNativePlugin, NativePluginBehaviorCallReport, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::diagnostics::{report_diagnostics, unloaded_plugin_error};
use super::keys::live_key;
use super::loading::lock_loaded_native_plugins;
use super::reports::{
    NativePluginRuntimeBehaviorCall, NativePluginRuntimeBehaviorDescriptor,
    NativePluginRuntimeCommandDispatchReport, NativePluginRuntimePlayModeExitReport,
    NativePluginRuntimePlayModeSnapshot, NativePluginRuntimePluginState,
    NativePluginRuntimeStateRestoreReport, NativePluginRuntimeStateSnapshot,
    NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND, NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND,
};
use super::NativePluginLiveHost;

impl NativePluginLiveHost {
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
                    state_schema_version: plugin.runtime_state_schema_version(),
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
            let loaded_schema = plugin.runtime_state_schema_version();
            if plugin_state.state_schema_version != loaded_schema {
                skipped_plugin_ids.push(plugin_state.plugin_id.clone());
                diagnostics.push(format!(
                    "runtime plugin {} restore-state skipped because snapshot state schema {:?} does not match loaded state schema {:?}",
                    plugin_state.plugin_id, plugin_state.state_schema_version, loaded_schema
                ));
                continue;
            }
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
}

pub(super) fn runtime_plugins<'a>(
    loaded: &'a BTreeMap<String, LoadedNativePlugin>,
) -> impl Iterator<Item = (String, &'a LoadedNativePlugin)> + 'a {
    let prefix = super::keys::live_key_prefix(PluginModuleKind::Runtime);
    loaded.iter().filter_map(move |(key, plugin)| {
        key.strip_prefix(prefix).map(|id| (id.to_string(), plugin))
    })
}

pub(super) fn runtime_behavior_descriptor(
    plugin_id: &str,
    plugin: &LoadedNativePlugin,
) -> NativePluginRuntimeBehaviorDescriptor {
    NativePluginRuntimeBehaviorDescriptor {
        plugin_id: plugin_id.to_string(),
        is_stateless: plugin.runtime_behavior_is_stateless(),
        state_schema_version: plugin.runtime_state_schema_version(),
        command_manifest_schema: plugin.runtime_command_manifest_schema().map(str::to_string),
        event_manifest_schema: plugin.runtime_event_manifest_schema().map(str::to_string),
        registration_manifest_schema: plugin
            .runtime_registration_manifest_schema()
            .map(str::to_string),
        command_manifest: plugin.runtime_command_manifest().map(str::to_string),
        event_manifest: plugin.runtime_event_manifest().map(str::to_string),
        registration_manifest: plugin.runtime_registration_manifest().map(str::to_string),
        validation_report: plugin.runtime_behavior_validation_report().cloned(),
    }
}

pub(super) fn unload_behavior(
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
                super::keys::module_kind_label(module_kind)
            )],
            payload: None,
        },
    };
    allow_missing_unload_callback_to_drop_handle(report)
}

pub(super) fn allow_missing_unload_callback_to_drop_handle(
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
