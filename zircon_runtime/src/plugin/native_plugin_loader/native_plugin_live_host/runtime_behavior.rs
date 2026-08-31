use crate::plugin::PluginModuleKind;

use super::super::loaded_native_plugin::NativePluginCallbackLeaseError;
use super::super::{
    LoadedNativePlugin, NativePluginBehaviorCallReport, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::diagnostics::report_diagnostics;
use super::keys::live_key;
use super::loading::{lock_loaded_native_plugins, NativePluginLiveHostLoadingError};
use super::reports::{
    NativePluginRuntimeBehaviorCall, NativePluginRuntimeBehaviorDescriptor,
    NativePluginRuntimeCommandDispatchReport, NativePluginRuntimePlayModeExitReport,
    NativePluginRuntimePlayModeSnapshot, NativePluginRuntimePluginState,
    NativePluginRuntimeStateRestoreReport, NativePluginRuntimeStateSnapshot,
    NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND, NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND,
};
use super::NativePluginLiveHost;

pub(super) type NativePluginRuntimeBehaviorResult<T> =
    std::result::Result<T, NativePluginRuntimeBehaviorError>;

#[derive(Debug)]
pub(super) enum NativePluginRuntimeBehaviorError {
    LiveHostLock(NativePluginLiveHostLoadingError),
    RuntimePluginNotLoaded {
        plugin_id: String,
    },
    CallbackSnapshot {
        plugin_id: String,
        source: NativePluginCallbackLeaseError,
    },
}

impl std::fmt::Display for NativePluginRuntimeBehaviorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LiveHostLock(error) => write!(formatter, "{error}"),
            Self::RuntimePluginNotLoaded { plugin_id } => write!(
                formatter,
                "plugin {plugin_id} is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
            ),
            Self::CallbackSnapshot { plugin_id, source } => write!(
                formatter,
                "runtime plugin {plugin_id} callback snapshot rejected: {source}"
            ),
        }
    }
}

impl std::error::Error for NativePluginRuntimeBehaviorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LiveHostLock(error) => Some(error),
            Self::CallbackSnapshot { source, .. } => Some(source),
            Self::RuntimePluginNotLoaded { .. } => None,
        }
    }
}

impl NativePluginLiveHost {
    pub fn runtime_behavior_descriptor(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginRuntimeBehaviorDescriptor, String> {
        self.runtime_behavior_descriptor_result(plugin_id)
            .map_err(|error| error.to_string())
    }

    pub(super) fn runtime_behavior_descriptor_result(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> NativePluginRuntimeBehaviorResult<NativePluginRuntimeBehaviorDescriptor> {
        let plugin_id = plugin_id.as_ref();
        let loaded = lock_loaded_native_plugins(&self.loaded)
            .map_err(NativePluginRuntimeBehaviorError::LiveHostLock)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(
                || NativePluginRuntimeBehaviorError::RuntimePluginNotLoaded {
                    plugin_id: plugin_id.to_string(),
                },
            )?;
        Ok(runtime_behavior_descriptor(plugin_id, plugin))
    }

    pub fn runtime_behavior_descriptors(
        &self,
    ) -> Result<Vec<NativePluginRuntimeBehaviorDescriptor>, String> {
        self.runtime_behavior_descriptors_result()
            .map_err(|error| error.to_string())
    }

    pub(super) fn runtime_behavior_descriptors_result(
        &self,
    ) -> NativePluginRuntimeBehaviorResult<Vec<NativePluginRuntimeBehaviorDescriptor>> {
        let loaded = lock_loaded_native_plugins(&self.loaded)
            .map_err(NativePluginRuntimeBehaviorError::LiveHostLock)?;
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
        self.invoke_runtime_plugin_command_result(plugin_id, command_name, payload)
            .map_err(|error| error.to_string())
    }

    pub(super) fn invoke_runtime_plugin_command_result(
        &self,
        plugin_id: impl AsRef<str>,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> NativePluginRuntimeBehaviorResult<NativePluginBehaviorCallReport> {
        let plugin_id = plugin_id.as_ref();
        let snapshot = {
            let loaded = lock_loaded_native_plugins(&self.loaded)
                .map_err(NativePluginRuntimeBehaviorError::LiveHostLock)?;
            let plugin = loaded
                .get(&live_key(PluginModuleKind::Runtime, plugin_id))
                .ok_or_else(
                    || NativePluginRuntimeBehaviorError::RuntimePluginNotLoaded {
                        plugin_id: plugin_id.to_string(),
                    },
                )?;
            plugin.runtime_behavior_snapshot().map_err(|source| {
                NativePluginRuntimeBehaviorError::CallbackSnapshot {
                    plugin_id: plugin_id.to_string(),
                    source,
                }
            })?
        };
        Ok(snapshot.invoke_command(command_name.as_ref(), payload.as_ref()))
    }

    pub fn dispatch_runtime_plugin_command(
        &self,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> Result<NativePluginRuntimeCommandDispatchReport, String> {
        self.dispatch_runtime_plugin_command_result(command_name, payload)
            .map_err(|error| error.to_string())
    }

    pub(super) fn dispatch_runtime_plugin_command_result(
        &self,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> NativePluginRuntimeBehaviorResult<NativePluginRuntimeCommandDispatchReport> {
        let command_name = command_name.as_ref();
        let payload = payload.as_ref();
        let snapshots = {
            let loaded = lock_loaded_native_plugins(&self.loaded)
                .map_err(NativePluginRuntimeBehaviorError::LiveHostLock)?;
            runtime_plugins(&loaded)
                .map(|(plugin_id, plugin)| {
                    let snapshot = plugin.runtime_behavior_snapshot().map_err(|source| {
                        NativePluginRuntimeBehaviorError::CallbackSnapshot {
                            plugin_id: plugin_id.clone(),
                            source,
                        }
                    })?;
                    Ok((plugin_id, snapshot))
                })
                .collect::<NativePluginRuntimeBehaviorResult<Vec<_>>>()?
        };
        let mut calls = Vec::with_capacity(snapshots.len());
        let mut diagnostics = Vec::new();
        for (plugin_id, snapshot) in snapshots {
            let report = snapshot.invoke_command(command_name, payload);
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
        self.save_runtime_plugin_state_result(plugin_id)
            .map_err(|error| error.to_string())
    }

    pub(super) fn save_runtime_plugin_state_result(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> NativePluginRuntimeBehaviorResult<NativePluginBehaviorCallReport> {
        let plugin_id = plugin_id.as_ref();
        let snapshot = {
            let loaded = lock_loaded_native_plugins(&self.loaded)
                .map_err(NativePluginRuntimeBehaviorError::LiveHostLock)?;
            let plugin = loaded
                .get(&live_key(PluginModuleKind::Runtime, plugin_id))
                .ok_or_else(
                    || NativePluginRuntimeBehaviorError::RuntimePluginNotLoaded {
                        plugin_id: plugin_id.to_string(),
                    },
                )?;
            plugin.runtime_behavior_snapshot().map_err(|source| {
                NativePluginRuntimeBehaviorError::CallbackSnapshot {
                    plugin_id: plugin_id.to_string(),
                    source,
                }
            })?
        };
        Ok(snapshot.save_state())
    }

    pub fn save_runtime_plugin_states(&self) -> Result<NativePluginRuntimeStateSnapshot, String> {
        self.save_runtime_plugin_states_result()
            .map_err(|error| error.to_string())
    }

    pub(super) fn save_runtime_plugin_states_result(
        &self,
    ) -> NativePluginRuntimeBehaviorResult<NativePluginRuntimeStateSnapshot> {
        let snapshots = {
            let loaded = lock_loaded_native_plugins(&self.loaded)
                .map_err(NativePluginRuntimeBehaviorError::LiveHostLock)?;
            runtime_plugins(&loaded)
                .map(|(plugin_id, plugin)| {
                    let state_schema_version = plugin.runtime_state_schema_version();
                    let snapshot = plugin.runtime_behavior_snapshot().map_err(|source| {
                        NativePluginRuntimeBehaviorError::CallbackSnapshot {
                            plugin_id: plugin_id.clone(),
                            source,
                        }
                    })?;
                    Ok((plugin_id, state_schema_version, snapshot))
                })
                .collect::<NativePluginRuntimeBehaviorResult<Vec<_>>>()?
        };
        let mut plugin_states = Vec::new();
        let mut diagnostics = Vec::new();
        for (plugin_id, state_schema_version, snapshot) in snapshots {
            let report = snapshot.save_state();
            diagnostics.extend(report_diagnostics(&plugin_id, "save-state", &report));
            if report.status_code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
                continue;
            }
            match report.payload {
                Some(state) => plugin_states.push(NativePluginRuntimePluginState {
                    plugin_id,
                    state_schema_version,
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
        self.restore_runtime_plugin_state_result(plugin_id, state)
            .map_err(|error| error.to_string())
    }

    pub(super) fn restore_runtime_plugin_state_result(
        &self,
        plugin_id: impl AsRef<str>,
        state: impl AsRef<[u8]>,
    ) -> NativePluginRuntimeBehaviorResult<NativePluginBehaviorCallReport> {
        let plugin_id = plugin_id.as_ref();
        let snapshot = {
            let loaded = lock_loaded_native_plugins(&self.loaded)
                .map_err(NativePluginRuntimeBehaviorError::LiveHostLock)?;
            let plugin = loaded
                .get(&live_key(PluginModuleKind::Runtime, plugin_id))
                .ok_or_else(
                    || NativePluginRuntimeBehaviorError::RuntimePluginNotLoaded {
                        plugin_id: plugin_id.to_string(),
                    },
                )?;
            plugin.runtime_behavior_snapshot().map_err(|source| {
                NativePluginRuntimeBehaviorError::CallbackSnapshot {
                    plugin_id: plugin_id.to_string(),
                    source,
                }
            })?
        };
        Ok(snapshot.restore_state(state.as_ref()))
    }

    pub fn restore_runtime_plugin_states(
        &self,
        snapshot: &NativePluginRuntimeStateSnapshot,
    ) -> Result<NativePluginRuntimeStateRestoreReport, String> {
        self.restore_runtime_plugin_states_result(snapshot)
            .map_err(|error| error.to_string())
    }

    pub(super) fn restore_runtime_plugin_states_result(
        &self,
        snapshot: &NativePluginRuntimeStateSnapshot,
    ) -> NativePluginRuntimeBehaviorResult<NativePluginRuntimeStateRestoreReport> {
        let mut calls = Vec::new();
        let mut skipped_plugin_ids = Vec::new();
        let mut diagnostics = Vec::new();
        let pending = {
            let loaded = lock_loaded_native_plugins(&self.loaded)
                .map_err(NativePluginRuntimeBehaviorError::LiveHostLock)?;
            let mut pending = Vec::new();
            for plugin_state in &snapshot.plugin_states {
                let Some(plugin) = loaded.get(&live_key(
                    PluginModuleKind::Runtime,
                    &plugin_state.plugin_id,
                )) else {
                    skipped_plugin_ids.push(plugin_state.plugin_id.clone());
                    diagnostics.push(
                        NativePluginRuntimeBehaviorError::RuntimePluginNotLoaded {
                            plugin_id: plugin_state.plugin_id.clone(),
                        }
                        .to_string(),
                    );
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
                let callback = plugin.runtime_behavior_snapshot().map_err(|source| {
                    NativePluginRuntimeBehaviorError::CallbackSnapshot {
                        plugin_id: plugin_state.plugin_id.clone(),
                        source,
                    }
                })?;
                pending.push((
                    plugin_state.plugin_id.clone(),
                    plugin_state.state.as_slice(),
                    callback,
                ));
            }
            pending
        };
        for (plugin_id, state, callback) in pending {
            let report = callback.restore_state(state);
            diagnostics.extend(report_diagnostics(&plugin_id, "restore-state", &report));
            calls.push(NativePluginRuntimeBehaviorCall { plugin_id, report });
        }
        Ok(NativePluginRuntimeStateRestoreReport {
            calls,
            skipped_plugin_ids,
            diagnostics,
        })
    }

    pub fn enter_runtime_play_mode(&self) -> Result<NativePluginRuntimePlayModeSnapshot, String> {
        self.enter_runtime_play_mode_result()
            .map_err(|error| error.to_string())
    }

    pub(super) fn enter_runtime_play_mode_result(
        &self,
    ) -> NativePluginRuntimeBehaviorResult<NativePluginRuntimePlayModeSnapshot> {
        let state_snapshot = self.save_runtime_plugin_states_result()?;
        let enter_report = self
            .dispatch_runtime_plugin_command_result(NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND, b"")?;
        Ok(NativePluginRuntimePlayModeSnapshot {
            state_snapshot,
            enter_report,
        })
    }

    pub fn exit_runtime_play_mode(
        &self,
        snapshot: &NativePluginRuntimePlayModeSnapshot,
    ) -> Result<NativePluginRuntimePlayModeExitReport, String> {
        self.exit_runtime_play_mode_result(snapshot)
            .map_err(|error| error.to_string())
    }

    pub(super) fn exit_runtime_play_mode_result(
        &self,
        snapshot: &NativePluginRuntimePlayModeSnapshot,
    ) -> NativePluginRuntimeBehaviorResult<NativePluginRuntimePlayModeExitReport> {
        let exit_report = self
            .dispatch_runtime_plugin_command_result(NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND, b"")?;
        let restore_report = self.restore_runtime_plugin_states_result(&snapshot.state_snapshot)?;
        Ok(NativePluginRuntimePlayModeExitReport {
            exit_report,
            restore_report,
        })
    }
}

pub(super) fn runtime_plugins<'a>(
    loaded: &'a super::keys::NativePluginLiveRegistry<LoadedNativePlugin>,
) -> impl Iterator<Item = (String, &'a LoadedNativePlugin)> + 'a {
    loaded
        .entries(PluginModuleKind::Runtime)
        .map(|(plugin_id, plugin)| (plugin_id.to_string(), plugin))
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
        PluginModuleKind::Runtime | PluginModuleKind::Editor => {
            plugin.unload_behavior_during_transition(module_kind)
        }
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
