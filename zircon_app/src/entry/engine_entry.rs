use std::fmt;
use std::sync::Arc;

use zircon_runtime::core::framework::render::RENDER_PROFILE_CONFIG_KEY;
use zircon_runtime::core::{CoreError, CoreHandle, CoreRuntime, ModuleDescriptor};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::plugin::RuntimeProfileId;
use zircon_runtime::{
    plugin::RuntimePluginFeatureRegistrationReport, plugin::RuntimePluginRegistrationReport,
};

use crate::plugins::{DefaultPlugins, HeadlessPlugins, PluginGroup};
use crate::plugins::{PluginGroupBuilder, PluginGroupError, ResolvedPluginGroup};

use super::{
    builtin_modules::{
        builtin_modules_for_config, builtin_modules_for_config_with_available_runtime_plugins,
        builtin_modules_for_config_with_runtime_plugin_and_feature_registrations,
        builtin_modules_for_config_with_runtime_plugin_registrations,
    },
    EntryConfig, EntryProfile,
};

use super::first_party_runtime_plugin_registrations_for_config;

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
    plugin_group: ResolvedPluginGroup,
}

impl BuiltinEngineEntry {
    pub fn for_profile(profile: EntryProfile) -> Result<Self, CoreError> {
        Self::for_config(&EntryConfig::new(profile))
    }

    pub fn for_config(config: &EntryConfig) -> Result<Self, CoreError> {
        let modules = builtin_modules_for_config(config)?;
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            plugin_group: plugin_group_for_config(config, modules)?,
        })
    }

    pub fn for_runtime_profile(profile_id: RuntimeProfileId) -> Result<Self, CoreError> {
        Self::for_config_with_first_party_runtime_plugin_registrations(
            &EntryConfig::for_runtime_profile(profile_id),
        )
    }

    pub fn for_config_with_first_party_runtime_plugin_registrations(
        config: &EntryConfig,
    ) -> Result<Self, CoreError> {
        Self::for_config_with_runtime_plugin_registrations(
            config,
            first_party_runtime_plugin_registrations_for_config(config),
        )
    }

    pub fn for_config_with_runtime_plugin_registrations(
        config: &EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<Self, CoreError> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        let modules =
            builtin_modules_for_config_with_runtime_plugin_registrations(config, &registrations)?;
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            plugin_group: plugin_group_for_config(config, modules)?,
        })
    }

    pub fn for_config_with_runtime_plugin_and_feature_registrations(
        config: &EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Result<Self, CoreError> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        let feature_registrations = feature_registrations.into_iter().collect::<Vec<_>>();
        let modules = builtin_modules_for_config_with_runtime_plugin_and_feature_registrations(
            config,
            &registrations,
            &feature_registrations,
        )?;
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            plugin_group: plugin_group_for_config(config, modules)?,
        })
    }

    pub fn for_config_with_available_runtime_plugins(
        config: &EntryConfig,
        available_plugin_ids: impl IntoIterator<Item = String>,
    ) -> Result<Self, CoreError> {
        let available_plugin_ids = available_plugin_ids.into_iter().collect::<Vec<_>>();
        let modules = builtin_modules_for_config_with_available_runtime_plugins(
            config,
            &available_plugin_ids,
        )?;
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            plugin_group: plugin_group_for_config(config, modules)?,
        })
    }

    pub fn plugin_group(&self) -> &ResolvedPluginGroup {
        &self.plugin_group
    }

    fn store_entry_config(&self, runtime: &CoreRuntime) {
        let runtime_handle = runtime.handle();
        runtime_handle
            .store_config(RENDER_PROFILE_CONFIG_KEY, &self.config.render_profile)
            .ok();
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
        self.plugin_group.modules()
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

fn plugin_group_for_config(
    config: &EntryConfig,
    modules: Vec<Arc<dyn EngineModule>>,
) -> Result<ResolvedPluginGroup, CoreError> {
    let mut builder = plugin_group_builder_for_config(config).map_err(plugin_group_core_error)?;
    for module in modules {
        if builder.contains(module.module_name()) {
            builder = builder.set(module).map_err(plugin_group_core_error)?;
        } else {
            builder = builder.add(module).map_err(plugin_group_core_error)?;
        }
    }
    Ok(builder.finish())
}

fn plugin_group_builder_for_config(
    config: &EntryConfig,
) -> Result<PluginGroupBuilder, PluginGroupError> {
    match config.profile {
        EntryProfile::Editor | EntryProfile::Runtime => DefaultPlugins::default().build(),
        EntryProfile::Headless => HeadlessPlugins::default().build(),
    }
}

fn plugin_group_core_error(error: PluginGroupError) -> CoreError {
    CoreError::Initialization("zircon_app plugin group".to_string(), error.to_string())
}
