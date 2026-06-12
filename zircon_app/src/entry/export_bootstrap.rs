use std::path::{Path, PathBuf};

use zircon_runtime::core::{CoreError, CoreHandle};
use zircon_runtime::{
    plugin::ExportProfile, plugin::ProjectPluginManifest,
    plugin::RuntimePluginFeatureRegistrationReport, plugin::RuntimePluginRegistrationReport,
    RuntimeTargetMode,
};

use super::{
    EntryConfig, EntryProfile, EntryRunner, EntryRuntimeBootstrap, NativePluginRuntimeBootstrap,
};

#[derive(Clone, Debug)]
pub struct ExportRuntimeBootstrapConfig {
    pub entry_profile: EntryProfile,
    pub target_mode: RuntimeTargetMode,
    pub project_plugins: ProjectPluginManifest,
    pub export_profile: ExportProfile,
    pub runtime_plugin_registrations: Vec<RuntimePluginRegistrationReport>,
    pub runtime_plugin_feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
}

#[derive(Clone, Copy, Debug)]
pub struct ExportRuntimePluginRegistrationProvider {
    register: fn() -> RuntimePluginRegistrationReport,
}

impl ExportRuntimePluginRegistrationProvider {
    pub const fn new(register: fn() -> RuntimePluginRegistrationReport) -> Self {
        Self { register }
    }

    fn into_report(self) -> RuntimePluginRegistrationReport {
        (self.register)()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExportRuntimePluginFeatureRegistrationProvider {
    register: fn() -> RuntimePluginFeatureRegistrationReport,
    provider_package_id: Option<&'static str>,
}

impl ExportRuntimePluginFeatureRegistrationProvider {
    pub const fn new(register: fn() -> RuntimePluginFeatureRegistrationReport) -> Self {
        Self {
            register,
            provider_package_id: None,
        }
    }

    pub const fn with_provider_package_id(mut self, provider_package_id: &'static str) -> Self {
        self.provider_package_id = Some(provider_package_id);
        self
    }

    fn into_report(self) -> RuntimePluginFeatureRegistrationReport {
        let report = (self.register)();
        match self.provider_package_id {
            Some(provider_package_id) => report.with_provider_package_id(provider_package_id),
            None => report,
        }
    }
}

impl ExportRuntimeBootstrapConfig {
    pub fn new(
        entry_profile: EntryProfile,
        target_mode: RuntimeTargetMode,
        project_plugins: ProjectPluginManifest,
        export_profile: ExportProfile,
    ) -> Self {
        Self {
            entry_profile,
            target_mode,
            project_plugins,
            export_profile,
            runtime_plugin_registrations: Vec::new(),
            runtime_plugin_feature_registrations: Vec::new(),
        }
    }

    pub fn with_runtime_plugin_registrations(
        mut self,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Self {
        self.runtime_plugin_registrations.extend(registrations);
        self
    }

    pub fn with_runtime_plugin_registration_providers(
        mut self,
        providers: impl IntoIterator<Item = ExportRuntimePluginRegistrationProvider>,
    ) -> Self {
        self.runtime_plugin_registrations.extend(
            providers
                .into_iter()
                .map(ExportRuntimePluginRegistrationProvider::into_report),
        );
        self
    }

    pub fn with_runtime_plugin_feature_registrations(
        mut self,
        registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Self {
        self.runtime_plugin_feature_registrations
            .extend(registrations);
        self
    }

    pub fn with_runtime_plugin_feature_registration_providers(
        mut self,
        providers: impl IntoIterator<Item = ExportRuntimePluginFeatureRegistrationProvider>,
    ) -> Self {
        self.runtime_plugin_feature_registrations.extend(
            providers
                .into_iter()
                .map(ExportRuntimePluginFeatureRegistrationProvider::into_report),
        );
        self
    }

    pub fn entry_config(&self) -> EntryConfig {
        EntryConfig::new(self.entry_profile)
            .with_target_mode(self.target_mode)
            .with_project_plugins(self.project_plugins.clone())
            .with_export_profile(self.export_profile.clone())
    }

    fn into_parts(
        self,
    ) -> (
        EntryConfig,
        Vec<RuntimePluginRegistrationReport>,
        Vec<RuntimePluginFeatureRegistrationReport>,
    ) {
        (
            EntryConfig::new(self.entry_profile)
                .with_target_mode(self.target_mode)
                .with_project_plugins(self.project_plugins)
                .with_export_profile(self.export_profile),
            self.runtime_plugin_registrations,
            self.runtime_plugin_feature_registrations,
        )
    }
}

pub fn bootstrap_export_runtime(
    config: ExportRuntimeBootstrapConfig,
) -> Result<CoreHandle, CoreError> {
    Ok(bootstrap_export_runtime_with_report(config)?.into_core())
}

pub fn bootstrap_export_runtime_with_report(
    config: ExportRuntimeBootstrapConfig,
) -> Result<EntryRuntimeBootstrap, CoreError> {
    let (entry_config, registrations, feature_registrations) = config.into_parts();
    EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations_and_report(
        entry_config,
        registrations,
        feature_registrations,
    )
}

pub fn bootstrap_export_runtime_with_native_plugins_from_export_root(
    config: ExportRuntimeBootstrapConfig,
    export_root: impl AsRef<Path>,
) -> Result<NativePluginRuntimeBootstrap, CoreError> {
    let (entry_config, registrations, feature_registrations) = config.into_parts();
    EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations_and_native_plugins_from_export_root(
        entry_config,
        registrations,
        feature_registrations,
        export_root,
    )
}

pub fn discover_export_root() -> std::io::Result<PathBuf> {
    let current_exe = std::env::current_exe()?;
    let current_dir = std::env::current_dir()?;
    let mut candidates = Vec::new();
    if let Some(parent) = current_exe.parent() {
        candidates.extend(parent.ancestors().map(PathBuf::from));
    }
    candidates.extend(current_dir.ancestors().map(PathBuf::from));
    for root in candidates {
        if root.join("plugins/native_plugins.toml").exists() {
            return Ok(root);
        }
    }
    Ok(current_dir)
}
