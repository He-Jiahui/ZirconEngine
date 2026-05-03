use std::path::Path;

use zircon_runtime::core::{CoreError, CoreHandle};
use zircon_runtime::{
    plugin::NativePluginBehaviorCallReport, plugin::NativePluginLiveHost,
    plugin::NativePluginRuntimeBehaviorDescriptor,
    plugin::NativePluginRuntimeCommandDispatchReport,
    plugin::NativePluginRuntimePlayModeExitReport, plugin::NativePluginRuntimePlayModeSnapshot,
    plugin::NativePluginRuntimeStateRestoreReport, plugin::NativePluginRuntimeStateSnapshot,
    plugin::RuntimePluginFeatureRegistrationReport, plugin::RuntimePluginRegistrationReport,
};

use crate::entry::{BuiltinEngineEntry, EngineEntry, EntryConfig};

use super::EntryRunner;

#[derive(Debug)]
pub struct NativePluginRuntimeBootstrap {
    // Native dynamic library handles must outlive the runtime graph that was
    // registered from their package manifests.
    core: CoreHandle,
    native_plugin_host: NativePluginLiveHost,
    diagnostics: Vec<String>,
}

impl NativePluginRuntimeBootstrap {
    pub fn core(&self) -> &CoreHandle {
        &self.core
    }

    pub fn clone_core(&self) -> CoreHandle {
        self.core.clone()
    }

    pub fn into_core(self) -> CoreHandle {
        self.core
    }

    pub fn into_parts(self) -> (CoreHandle, NativePluginLiveHost, Vec<String>) {
        (self.core, self.native_plugin_host, self.diagnostics)
    }

    pub fn native_plugin_host(&self) -> &NativePluginLiveHost {
        &self.native_plugin_host
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn runtime_behavior_descriptor(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginRuntimeBehaviorDescriptor, String> {
        self.native_plugin_host
            .runtime_behavior_descriptor(plugin_id)
    }

    pub fn runtime_behavior_descriptors(
        &self,
    ) -> Result<Vec<NativePluginRuntimeBehaviorDescriptor>, String> {
        self.native_plugin_host.runtime_behavior_descriptors()
    }

    pub fn invoke_runtime_plugin_command(
        &self,
        plugin_id: impl AsRef<str>,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        self.native_plugin_host
            .invoke_runtime_plugin_command(plugin_id, command_name, payload)
    }

    pub fn dispatch_runtime_plugin_command(
        &self,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> Result<NativePluginRuntimeCommandDispatchReport, String> {
        self.native_plugin_host
            .dispatch_runtime_plugin_command(command_name, payload)
    }

    pub fn save_runtime_plugin_state(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        self.native_plugin_host.save_runtime_plugin_state(plugin_id)
    }

    pub fn save_runtime_plugin_states(&self) -> Result<NativePluginRuntimeStateSnapshot, String> {
        self.native_plugin_host.save_runtime_plugin_states()
    }

    pub fn restore_runtime_plugin_state(
        &self,
        plugin_id: impl AsRef<str>,
        state: impl AsRef<[u8]>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        self.native_plugin_host
            .restore_runtime_plugin_state(plugin_id, state)
    }

    pub fn restore_runtime_plugin_states(
        &self,
        snapshot: &NativePluginRuntimeStateSnapshot,
    ) -> Result<NativePluginRuntimeStateRestoreReport, String> {
        self.native_plugin_host
            .restore_runtime_plugin_states(snapshot)
    }

    pub fn enter_runtime_play_mode(&self) -> Result<NativePluginRuntimePlayModeSnapshot, String> {
        self.native_plugin_host.enter_runtime_play_mode()
    }

    pub fn exit_runtime_play_mode(
        &self,
        snapshot: &NativePluginRuntimePlayModeSnapshot,
    ) -> Result<NativePluginRuntimePlayModeExitReport, String> {
        self.native_plugin_host.exit_runtime_play_mode(snapshot)
    }
}

impl EntryRunner {
    pub fn bootstrap(config: EntryConfig) -> Result<CoreHandle, CoreError> {
        BuiltinEngineEntry::for_config(&config)?.bootstrap()
    }

    pub fn bootstrap_with_runtime_plugin_registrations(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<CoreHandle, CoreError> {
        BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(&config, registrations)?
            .bootstrap()
    }

    pub fn bootstrap_with_runtime_plugin_and_feature_registrations(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Result<CoreHandle, CoreError> {
        BuiltinEngineEntry::for_config_with_runtime_plugin_and_feature_registrations(
            &config,
            registrations,
            feature_registrations,
        )?
        .bootstrap()
    }

    pub fn bootstrap_with_native_plugins_from_export_root(
        config: EntryConfig,
        export_root: impl AsRef<Path>,
    ) -> Result<NativePluginRuntimeBootstrap, CoreError> {
        let native_plugin_host = NativePluginLiveHost::default();
        let native_report = native_plugin_host
            .load_runtime_plugins_from_export_root(export_root)
            .map_err(|error| {
                CoreError::Initialization("NativePluginLiveHost".to_string(), error)
            })?;
        for diagnostic in &native_report.diagnostics {
            eprintln!("[zircon_app] native plugin warning: {diagnostic}");
        }
        let core = BuiltinEngineEntry::for_config_with_runtime_plugin_and_feature_registrations(
            &config,
            native_report.runtime_plugin_registration_reports.clone(),
            native_report
                .runtime_plugin_feature_registration_reports
                .clone(),
        )?
        .bootstrap()?;
        Ok(NativePluginRuntimeBootstrap {
            core,
            native_plugin_host,
            diagnostics: native_report.diagnostics,
        })
    }
}
