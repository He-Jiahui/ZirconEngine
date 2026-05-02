use std::fmt;
use std::sync::Arc;

use zircon_runtime::core::{CoreError, CoreHandle, CoreRuntime, ModuleDescriptor};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::{
    plugin::RuntimePluginFeatureRegistrationReport, plugin::RuntimePluginRegistrationReport,
};

use super::{
    builtin_modules::{
        builtin_modules_for_config, builtin_modules_for_config_with_available_runtime_plugins,
        builtin_modules_for_config_with_runtime_plugin_and_feature_registrations,
        builtin_modules_for_config_with_runtime_plugin_registrations,
    },
    EntryConfig, EntryProfile,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryRunMode {
    Editor,
    Runtime,
    Headless,
}

impl From<EntryProfile> for EntryRunMode {
    fn from(value: EntryProfile) -> Self {
        match value {
            EntryProfile::Editor => Self::Editor,
            EntryProfile::Runtime => Self::Runtime,
            EntryProfile::Headless => Self::Headless,
        }
    }
}

pub trait EngineEntry: Send + Sync + fmt::Debug {
    fn profile(&self) -> EntryProfile;
    fn run_mode(&self) -> EntryRunMode;
    fn modules(&self) -> &[Arc<dyn EngineModule>];

    fn module_descriptors(&self) -> Vec<ModuleDescriptor> {
        self.modules()
            .iter()
            .map(|module| module.descriptor())
            .collect()
    }

    fn bootstrap(&self) -> Result<CoreHandle, CoreError> {
        let runtime = CoreRuntime::new();
        let descriptors = self.module_descriptors();

        for descriptor in &descriptors {
            runtime.register_module(descriptor.clone())?;
        }
        for descriptor in &descriptors {
            runtime.activate_module(&descriptor.name)?;
        }

        Ok(runtime.handle())
    }
}

#[derive(Clone, Debug)]
pub struct BuiltinEngineEntry {
    config: EntryConfig,
    profile: EntryProfile,
    modules: Vec<Arc<dyn EngineModule>>,
}

impl BuiltinEngineEntry {
    pub fn for_profile(profile: EntryProfile) -> Result<Self, CoreError> {
        Self::for_config(&EntryConfig::new(profile))
    }

    pub fn for_config(config: &EntryConfig) -> Result<Self, CoreError> {
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            modules: builtin_modules_for_config(config)?,
        })
    }

    pub fn for_config_with_runtime_plugin_registrations(
        config: &EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<Self, CoreError> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            modules: builtin_modules_for_config_with_runtime_plugin_registrations(
                config,
                &registrations,
            )?,
        })
    }

    pub fn for_config_with_runtime_plugin_and_feature_registrations(
        config: &EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Result<Self, CoreError> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        let feature_registrations = feature_registrations.into_iter().collect::<Vec<_>>();
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            modules: builtin_modules_for_config_with_runtime_plugin_and_feature_registrations(
                config,
                &registrations,
                &feature_registrations,
            )?,
        })
    }

    pub fn for_config_with_available_runtime_plugins(
        config: &EntryConfig,
        available_plugin_ids: impl IntoIterator<Item = String>,
    ) -> Result<Self, CoreError> {
        let available_plugin_ids = available_plugin_ids.into_iter().collect::<Vec<_>>();
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            modules: builtin_modules_for_config_with_available_runtime_plugins(
                config,
                &available_plugin_ids,
            )?,
        })
    }

    fn store_entry_config(&self, runtime: &CoreRuntime) {
        let _ = &self.config;
        #[cfg(not(feature = "target-editor-host"))]
        let _ = runtime;
        #[cfg(feature = "target-editor-host")]
        if matches!(self.config.profile, EntryProfile::Editor) {
            if let Some(subsystems) = &self.config.editor_enabled_subsystems {
                runtime.store_config_value(
                    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
                    serde_json::json!(subsystems),
                );
            }
            runtime.store_config_value(
                zircon_editor::EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY,
                serde_json::json!(self.config.editor_runtime_sandbox_enabled),
            );
        }
    }
}

impl EngineEntry for BuiltinEngineEntry {
    fn profile(&self) -> EntryProfile {
        self.profile
    }

    fn run_mode(&self) -> EntryRunMode {
        self.profile.into()
    }

    fn modules(&self) -> &[Arc<dyn EngineModule>] {
        &self.modules
    }

    fn bootstrap(&self) -> Result<CoreHandle, CoreError> {
        let runtime = CoreRuntime::new();
        let descriptors = self.module_descriptors();

        self.store_entry_config(&runtime);
        for descriptor in &descriptors {
            runtime.register_module(descriptor.clone())?;
        }
        for descriptor in &descriptors {
            runtime.activate_module(&descriptor.name)?;
        }

        Ok(runtime.handle())
    }
}
