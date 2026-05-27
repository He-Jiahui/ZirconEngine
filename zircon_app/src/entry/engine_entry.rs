use std::fmt;
use std::sync::Arc;

use zircon_runtime::core::framework::render::RENDER_PROFILE_CONFIG_KEY;
use zircon_runtime::core::framework::window::{
    WindowDescriptor, PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,
};
use zircon_runtime::core::{CoreError, CoreHandle, CoreRuntime, ModuleDescriptor};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::platform::{
    PlatformConfig, PlatformFeatureSelection, PlatformTarget, PLATFORM_CONFIG_KEY,
};
use zircon_runtime::plugin::{RuntimePluginAvailabilityReport, RuntimeProfileId};
use zircon_runtime::RuntimeTargetMode;
use zircon_runtime::{
    plugin::RuntimePluginFeatureRegistrationReport, plugin::RuntimePluginRegistrationReport,
};

use crate::plugins::{DefaultPlugins, DevPlugins, HeadlessPlugins, MinimalPlugins, PluginGroup};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryModuleSelection {
    pub name: String,
    pub description: String,
    pub driver_count: usize,
    pub manager_count: usize,
    pub plugin_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntryModuleSelectionReport {
    pub profile: EntryProfile,
    pub run_mode: EntryRunMode,
    pub runtime_profile: Option<RuntimeProfileId>,
    pub target_mode: RuntimeTargetMode,
    pub platform_config: PlatformConfig,
    pub window_descriptor: WindowDescriptor,
    pub plugin_group: String,
    pub runtime_plugin_availability: RuntimePluginAvailabilityReport,
    pub modules: Vec<EntryModuleSelection>,
}

impl EntryModuleSelectionReport {
    pub fn module_keys(&self) -> Vec<&str> {
        self.modules
            .iter()
            .map(|module| module.name.as_str())
            .collect()
    }

    pub fn diagnostic_lines(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(self.modules.len() + 6);
        lines.push(format!("entry.profile={:?}", self.profile));
        lines.push(format!("entry.run_mode={:?}", self.run_mode));
        lines.push(format!(
            "entry.runtime_profile={}",
            self.runtime_profile
                .map(|profile| format!("{profile:?}"))
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!("entry.target_mode={:?}", self.target_mode));
        lines.extend(self.platform_config.diagnostic_lines());
        lines.extend(self.window_descriptor.diagnostic_lines());
        lines.push(format!("entry.plugin_group={}", self.plugin_group));
        self.runtime_plugin_availability
            .push_diagnostic_lines(&mut lines);
        lines.push(format!("entry.modules={}", self.modules.len()));
        lines.extend(
            self.modules
                .iter()
                .map(EntryModuleSelection::diagnostic_line),
        );
        lines
    }

    pub fn format_diagnostics(&self) -> String {
        self.diagnostic_lines().join("\n")
    }
}

impl EntryModuleSelection {
    fn diagnostic_line(&self) -> String {
        format!(
            "module={} drivers={} managers={} plugins={} description={}",
            self.name, self.driver_count, self.manager_count, self.plugin_count, self.description
        )
    }
}

impl From<ModuleDescriptor> for EntryModuleSelection {
    fn from(descriptor: ModuleDescriptor) -> Self {
        Self {
            name: descriptor.name,
            description: descriptor.description,
            driver_count: descriptor.drivers.len(),
            manager_count: descriptor.managers.len(),
            plugin_count: descriptor.plugins.len(),
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
    runtime_plugin_availability: RuntimePluginAvailabilityReport,
}

impl BuiltinEngineEntry {
    pub fn for_profile(profile: EntryProfile) -> Result<Self, CoreError> {
        Self::for_config(&EntryConfig::new(profile))
    }

    pub fn for_config(config: &EntryConfig) -> Result<Self, CoreError> {
        let selection = builtin_modules_for_config(config)?;
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            plugin_group: plugin_group_for_config(config, selection.modules)?,
            runtime_plugin_availability: selection.runtime_plugin_availability,
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
        let selection =
            builtin_modules_for_config_with_runtime_plugin_registrations(config, &registrations)?;
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            plugin_group: plugin_group_for_config(config, selection.modules)?,
            runtime_plugin_availability: selection.runtime_plugin_availability,
        })
    }

    pub fn for_config_with_runtime_plugin_and_feature_registrations(
        config: &EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Result<Self, CoreError> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        let feature_registrations = feature_registrations.into_iter().collect::<Vec<_>>();
        let selection = builtin_modules_for_config_with_runtime_plugin_and_feature_registrations(
            config,
            &registrations,
            &feature_registrations,
        )?;
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            plugin_group: plugin_group_for_config(config, selection.modules)?,
            runtime_plugin_availability: selection.runtime_plugin_availability,
        })
    }

    pub fn for_config_with_available_runtime_plugins(
        config: &EntryConfig,
        available_plugin_ids: impl IntoIterator<Item = String>,
    ) -> Result<Self, CoreError> {
        let available_plugin_ids = available_plugin_ids.into_iter().collect::<Vec<_>>();
        let selection = builtin_modules_for_config_with_available_runtime_plugins(
            config,
            &available_plugin_ids,
        )?;
        Ok(Self {
            config: config.clone(),
            profile: config.profile,
            plugin_group: plugin_group_for_config(config, selection.modules)?,
            runtime_plugin_availability: selection.runtime_plugin_availability,
        })
    }

    pub fn plugin_group(&self) -> &ResolvedPluginGroup {
        &self.plugin_group
    }

    pub fn runtime_plugin_availability(&self) -> &RuntimePluginAvailabilityReport {
        &self.runtime_plugin_availability
    }

    pub fn module_selection_report(&self) -> EntryModuleSelectionReport {
        EntryModuleSelectionReport {
            profile: self.profile,
            run_mode: self.run_mode(),
            runtime_profile: self.config.runtime_profile(),
            target_mode: self.config.target_mode,
            platform_config: platform_config_for_entry_config(&self.config),
            window_descriptor: self.config.window_descriptor.clone(),
            plugin_group: self.plugin_group.name().to_string(),
            runtime_plugin_availability: self.runtime_plugin_availability.clone(),
            modules: self
                .module_descriptors()
                .into_iter()
                .map(EntryModuleSelection::from)
                .collect(),
        }
    }

    fn store_entry_config(&self, runtime: &CoreRuntime) {
        let runtime_handle = runtime.handle();
        runtime_handle
            .store_config(
                PLATFORM_CONFIG_KEY,
                &platform_config_for_entry_config(&self.config),
            )
            .ok();
        runtime_handle
            .store_config(RENDER_PROFILE_CONFIG_KEY, &self.config.render_profile)
            .ok();
        runtime_handle
            .store_config(
                PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,
                &self.config.window_descriptor,
            )
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
        self.store_entry_config(&runtime);

        Ok(runtime.handle())
    }
}

fn plugin_group_for_config(
    config: &EntryConfig,
    modules: Vec<Arc<dyn EngineModule>>,
) -> Result<ResolvedPluginGroup, CoreError> {
    let mut builder = plugin_group_builder_for_config(config).map_err(plugin_group_core_error)?;
    let append_unmatched_modules =
        !matches!(config.runtime_profile(), Some(RuntimeProfileId::Minimal));
    for module in modules {
        if builder.contains(module.module_name()) {
            builder = builder.set(module).map_err(plugin_group_core_error)?;
        } else if append_unmatched_modules {
            builder = builder.add(module).map_err(plugin_group_core_error)?;
        }
    }
    Ok(builder.finish())
}

fn plugin_group_builder_for_config(
    config: &EntryConfig,
) -> Result<PluginGroupBuilder, PluginGroupError> {
    match config.runtime_profile() {
        Some(RuntimeProfileId::Minimal) => return MinimalPlugins.build(),
        Some(RuntimeProfileId::Dev) => return DevPlugins::default().build(),
        _ => {}
    }
    match config.profile {
        EntryProfile::Editor | EntryProfile::Runtime => DefaultPlugins::default().build(),
        EntryProfile::Headless => HeadlessPlugins::default().build(),
    }
}

fn platform_config_for_entry_config(config: &EntryConfig) -> PlatformConfig {
    let headless = matches!(config.target_mode, RuntimeTargetMode::ServerRuntime);
    PlatformConfig {
        enabled: !matches!(config.runtime_profile(), Some(RuntimeProfileId::Minimal)),
        target: if headless {
            PlatformTarget::Headless
        } else {
            PlatformTarget::current()
        },
        target_mode: config.target_mode,
        features: if headless {
            PlatformFeatureSelection::headless()
        } else {
            PlatformFeatureSelection::from_compiled_features()
        },
    }
}

fn plugin_group_core_error(error: PluginGroupError) -> CoreError {
    CoreError::Initialization("zircon_app plugin group".to_string(), error.to_string())
}
