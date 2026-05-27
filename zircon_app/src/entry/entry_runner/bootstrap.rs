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

use crate::entry::{BuiltinEngineEntry, EngineEntry, EntryConfig, EntryModuleSelectionReport};

use super::EntryRunner;

#[derive(Debug)]
pub struct EntryRuntimeBootstrap {
    core: CoreHandle,
    module_selection_report: EntryModuleSelectionReport,
}

impl EntryRuntimeBootstrap {
    pub fn core(&self) -> &CoreHandle {
        &self.core
    }

    pub fn clone_core(&self) -> CoreHandle {
        self.core.clone()
    }

    pub fn into_core(self) -> CoreHandle {
        self.core
    }

    pub fn module_selection_report(&self) -> &EntryModuleSelectionReport {
        &self.module_selection_report
    }

    pub fn into_parts(self) -> (CoreHandle, EntryModuleSelectionReport) {
        (self.core, self.module_selection_report)
    }
}

#[derive(Debug)]
pub struct NativePluginRuntimeBootstrap {
    // Native dynamic library handles must outlive the runtime graph that was
    // registered from their package manifests.
    core: CoreHandle,
    native_plugin_host: NativePluginLiveHost,
    module_selection_report: EntryModuleSelectionReport,
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

    pub fn into_parts_with_report(
        self,
    ) -> (
        CoreHandle,
        NativePluginLiveHost,
        EntryModuleSelectionReport,
        Vec<String>,
    ) {
        (
            self.core,
            self.native_plugin_host,
            self.module_selection_report,
            self.diagnostics,
        )
    }

    pub fn native_plugin_host(&self) -> &NativePluginLiveHost {
        &self.native_plugin_host
    }

    pub fn module_selection_report(&self) -> &EntryModuleSelectionReport {
        &self.module_selection_report
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
    pub fn module_selection_report(
        config: EntryConfig,
    ) -> Result<EntryModuleSelectionReport, CoreError> {
        Ok(BuiltinEngineEntry::for_config(&config)?.module_selection_report())
    }

    pub fn module_selection_diagnostics(config: EntryConfig) -> Result<String, CoreError> {
        Ok(Self::module_selection_report(config)?.format_diagnostics())
    }

    pub fn module_selection_report_with_first_party_runtime_plugin_registrations(
        config: EntryConfig,
    ) -> Result<EntryModuleSelectionReport, CoreError> {
        Ok(
            BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations(&config)?
                .module_selection_report(),
        )
    }

    pub fn module_selection_diagnostics_with_first_party_runtime_plugin_registrations(
        config: EntryConfig,
    ) -> Result<String, CoreError> {
        Ok(
            Self::module_selection_report_with_first_party_runtime_plugin_registrations(config)?
                .format_diagnostics(),
        )
    }

    pub fn module_selection_report_with_runtime_plugin_registrations(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<EntryModuleSelectionReport, CoreError> {
        Ok(
            BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(
                &config,
                registrations,
            )?
            .module_selection_report(),
        )
    }

    pub fn module_selection_diagnostics_with_runtime_plugin_registrations(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<String, CoreError> {
        Ok(
            Self::module_selection_report_with_runtime_plugin_registrations(config, registrations)?
                .format_diagnostics(),
        )
    }

    pub fn module_selection_report_with_runtime_plugin_and_feature_registrations(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Result<EntryModuleSelectionReport, CoreError> {
        Ok(
            BuiltinEngineEntry::for_config_with_runtime_plugin_and_feature_registrations(
                &config,
                registrations,
                feature_registrations,
            )?
            .module_selection_report(),
        )
    }

    pub fn module_selection_diagnostics_with_runtime_plugin_and_feature_registrations(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Result<String, CoreError> {
        Ok(
            Self::module_selection_report_with_runtime_plugin_and_feature_registrations(
                config,
                registrations,
                feature_registrations,
            )?
            .format_diagnostics(),
        )
    }

    pub fn bootstrap(config: EntryConfig) -> Result<CoreHandle, CoreError> {
        Ok(Self::bootstrap_with_report(config)?.into_core())
    }

    pub fn bootstrap_with_report(config: EntryConfig) -> Result<EntryRuntimeBootstrap, CoreError> {
        bootstrap_entry_with_report(BuiltinEngineEntry::for_config(&config)?)
    }

    pub fn bootstrap_with_first_party_runtime_plugin_registrations(
        config: EntryConfig,
    ) -> Result<CoreHandle, CoreError> {
        Ok(
            Self::bootstrap_with_first_party_runtime_plugin_registrations_and_report(config)?
                .into_core(),
        )
    }

    pub fn bootstrap_with_first_party_runtime_plugin_registrations_and_report(
        config: EntryConfig,
    ) -> Result<EntryRuntimeBootstrap, CoreError> {
        bootstrap_entry_with_report(
            BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations(&config)?,
        )
    }

    pub fn bootstrap_with_runtime_plugin_registrations(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<CoreHandle, CoreError> {
        Ok(
            Self::bootstrap_with_runtime_plugin_registrations_and_report(config, registrations)?
                .into_core(),
        )
    }

    pub fn bootstrap_with_runtime_plugin_registrations_and_report(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<EntryRuntimeBootstrap, CoreError> {
        bootstrap_entry_with_report(
            BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(
                &config,
                registrations,
            )?,
        )
    }

    pub fn bootstrap_with_runtime_plugin_and_feature_registrations(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Result<CoreHandle, CoreError> {
        Ok(
            Self::bootstrap_with_runtime_plugin_and_feature_registrations_and_report(
                config,
                registrations,
                feature_registrations,
            )?
            .into_core(),
        )
    }

    pub fn bootstrap_with_runtime_plugin_and_feature_registrations_and_report(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Result<EntryRuntimeBootstrap, CoreError> {
        bootstrap_entry_with_report(
            BuiltinEngineEntry::for_config_with_runtime_plugin_and_feature_registrations(
                &config,
                registrations,
                feature_registrations,
            )?,
        )
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
        let entry = BuiltinEngineEntry::for_config_with_runtime_plugin_and_feature_registrations(
            &config,
            native_report.runtime_plugin_registration_reports.clone(),
            native_report
                .runtime_plugin_feature_registration_reports
                .clone(),
        )?;
        let module_selection_report = entry.module_selection_report();
        let core = entry.bootstrap()?;
        Ok(NativePluginRuntimeBootstrap {
            core,
            native_plugin_host,
            module_selection_report,
            diagnostics: native_report.diagnostics,
        })
    }
}

fn bootstrap_entry_with_report(
    entry: BuiltinEngineEntry,
) -> Result<EntryRuntimeBootstrap, CoreError> {
    let module_selection_report = entry.module_selection_report();
    let core = entry.bootstrap()?;
    Ok(EntryRuntimeBootstrap {
        core,
        module_selection_report,
    })
}
