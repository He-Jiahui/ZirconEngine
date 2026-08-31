use std::fmt;
use std::sync::Arc;

use zircon_runtime::builtin::{RuntimeModuleCompositionIdentity, RuntimeModuleCompositionPlan};
use zircon_runtime::core::diagnostics::RuntimeDevtoolsPluginCatalogEntry;
use zircon_runtime::core::framework::platform::{
    PreferenceStorageBackendKind, RuntimeTargetMode, PLATFORM_MODULE_NAME,
};
use zircon_runtime::core::framework::project::RuntimeProfileId;
use zircon_runtime::core::framework::render::RENDER_PROFILE_CONFIG_KEY;
use zircon_runtime::core::framework::window::{
    WindowDescriptor, PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,
};
use zircon_runtime::core::{CoreError, CoreHandle, CoreRuntime, ModuleDescriptor};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::platform::{
    PlatformConfig, PlatformDriver, PlatformFeatureSelection, PreferenceStorageBackend,
    PLATFORM_CONFIG_KEY, PLATFORM_DRIVER_NAME,
};
use zircon_runtime::plugin::{
    CompiledProjectPluginPlan, RuntimePluginAvailabilityReport, RuntimePluginBridgeLifecycleState,
    RuntimePluginDescriptor,
};
use zircon_runtime::{
    engine_module::factory, plugin::RuntimePluginFeatureRegistrationReport,
    plugin::RuntimePluginRegistrationReport,
};

use crate::plugins::{PluginGroupError, ResolvedPluginGroup};

use super::{
    builtin_modules::{
        builtin_modules_for_config_with_effective_manifest_and_runtime_plugin_registrations,
        builtin_modules_for_config_with_runtime_plugin_and_feature_registrations,
        builtin_modules_for_config_with_runtime_plugin_registrations,
        effective_project_plugin_manifest,
    },
    EntryConfig, EntryProfile, ProductHostConfigError, ProductHostConfigProvenance,
    ProductRoleRequest, ResolvedProductHostConfig,
};

use super::first_party_runtime_plugin_registrations_for_manifest;
use super::platform_preferences::{
    planned_preference_storage_backend, preference_storage_backend_for_bootstrap,
    HostPreferenceStorageBackend,
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
    pub product_role: ProductRoleRequest,
    pub product_config_provenance: ProductHostConfigProvenance,
    pub profile: EntryProfile,
    pub run_mode: EntryRunMode,
    pub runtime_profile: Option<RuntimeProfileId>,
    pub target_mode: RuntimeTargetMode,
    pub platform_config: PlatformConfig,
    pub preference_storage_backend: PreferenceStorageBackendKind,
    pub window_descriptor: WindowDescriptor,
    pub plugin_group: String,
    pub runtime_plugin_availability: RuntimePluginAvailabilityReport,
    pub runtime_module_composition_identity: RuntimeModuleCompositionIdentity,
    pub runtime_module_warnings: Vec<String>,
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
        let mut lines = Vec::with_capacity(self.modules.len() + 18);
        lines.push(format!("entry.product_role={:?}", self.product_role));
        lines.extend(self.product_role.descriptor().diagnostic_lines());
        lines.push(format!(
            "entry.config_source.profile={:?}",
            self.product_config_provenance.profile()
        ));
        lines.push(format!(
            "entry.config_source.runtime_profile={:?}",
            self.product_config_provenance.runtime_profile()
        ));
        lines.push(format!(
            "entry.config_source.target_mode={:?}",
            self.product_config_provenance.target_mode()
        ));
        lines.push(format!(
            "entry.config_source.platform_target={:?}",
            self.product_config_provenance.platform_target()
        ));
        lines.push(format!(
            "entry.config_source.project_plugins={:?}",
            self.product_config_provenance.project_plugins()
        ));
        lines.push(format!(
            "entry.config_source.export_profile={:?}",
            self.product_config_provenance.export_profile()
        ));
        lines.push(format!(
            "entry.config_source.render_profile={:?}",
            self.product_config_provenance.render_profile()
        ));
        lines.push(format!(
            "entry.config_source.window_descriptor={:?}",
            self.product_config_provenance.window_descriptor()
        ));
        lines.push(format!(
            "entry.config_source.editor_enabled_subsystems={:?}",
            self.product_config_provenance.editor_enabled_subsystems()
        ));
        lines.push(format!(
            "entry.config_source.editor_runtime_sandbox={:?}",
            self.product_config_provenance.editor_runtime_sandbox()
        ));
        lines.push(format!("entry.profile={:?}", self.profile));
        lines.push(format!("entry.run_mode={:?}", self.run_mode));
        lines.push(format!(
            "entry.runtime_profile={}",
            self.runtime_profile
                .map(|profile| format!("{profile:?}"))
                .unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!("entry.target_mode={:?}", self.target_mode));
        lines.extend(
            self.platform_config
                .diagnostic_lines_with_preference_storage_backend(self.preference_storage_backend),
        );
        lines.extend(self.window_descriptor.diagnostic_lines());
        lines.push(format!("entry.plugin_group={}", self.plugin_group));
        self.runtime_plugin_availability
            .push_diagnostic_lines(&mut lines);
        lines.push(format!(
            "entry.runtime_module_composition.hash={}",
            self.runtime_module_composition_identity
                .composition_hash_hex()
        ));
        lines.push(format!(
            "entry.runtime_module_composition.catalog_generation={}",
            self.runtime_module_composition_identity
                .catalog_generation()
                .map(|generation| generation.to_string())
                .unwrap_or_else(|| "legacy".to_owned())
        ));
        lines.push(format!(
            "entry.runtime_module_warnings={}",
            self.runtime_module_warnings.len()
        ));
        lines.extend(
            self.runtime_module_warnings
                .iter()
                .map(|warning| format!("entry.runtime_module_warning={warning}")),
        );
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

pub(crate) trait EngineEntry: Send + Sync + fmt::Debug {
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
        runtime.activate_registered_modules()?;

        Ok(runtime.handle())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BuiltinEngineEntry {
    config: ResolvedProductHostConfig,
    plugin_group: ResolvedPluginGroup,
    runtime_module_composition: RuntimeModuleCompositionPlan,
    plugin_bridge_lifecycle_state: Option<RuntimePluginBridgeLifecycleState>,
    compiled_project_plugin_plan: Option<Arc<CompiledProjectPluginPlan>>,
    preference_storage_backend: Option<HostPreferenceStorageBackend>,
}

impl BuiltinEngineEntry {
    pub fn for_profile(profile: EntryProfile) -> Result<Self, CoreError> {
        Self::for_config(&EntryConfig::new(profile))
    }

    pub fn for_config(config: &EntryConfig) -> Result<Self, CoreError> {
        Self::for_config_with_first_party_runtime_plugin_registrations(config)
    }

    pub fn for_runtime_profile(profile_id: RuntimeProfileId) -> Result<Self, CoreError> {
        Self::for_config(&EntryConfig::for_runtime_profile(profile_id))
    }

    pub(crate) fn for_config_with_first_party_runtime_plugin_registrations(
        config: &EntryConfig,
    ) -> Result<Self, CoreError> {
        let config = resolve_product_host_config(config)?;
        Self::for_resolved_config_with_first_party_runtime_plugin_registrations(&config)
    }

    pub(super) fn for_resolved_config_with_first_party_runtime_plugin_registrations(
        config: &ResolvedProductHostConfig,
    ) -> Result<Self, CoreError> {
        let effective_manifest = effective_project_plugin_manifest(config);
        let registrations = first_party_runtime_plugin_registrations_for_manifest(
            config.target_mode(),
            &effective_manifest,
        );
        let selection =
            builtin_modules_for_config_with_effective_manifest_and_runtime_plugin_registrations(
                config,
                &effective_manifest,
                &registrations,
            )?;
        Ok(Self {
            config: config.clone(),
            plugin_group: plugin_group_for_config(config, &selection.composition)?,
            runtime_module_composition: selection.composition,
            plugin_bridge_lifecycle_state: selection.plugin_bridge_lifecycle_state,
            compiled_project_plugin_plan: selection.compiled_project_plugin_plan,
            preference_storage_backend: None,
        })
    }

    pub(crate) fn for_config_with_runtime_plugin_registrations(
        config: &EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<Self, CoreError> {
        let config = resolve_product_host_config(config)?;
        Self::for_resolved_config_with_runtime_plugin_registrations(&config, registrations)
    }

    pub(super) fn for_resolved_config_with_runtime_plugin_registrations(
        config: &ResolvedProductHostConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<Self, CoreError> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        let selection =
            builtin_modules_for_config_with_runtime_plugin_registrations(config, &registrations)?;
        Ok(Self {
            config: config.clone(),
            plugin_group: plugin_group_for_config(config, &selection.composition)?,
            runtime_module_composition: selection.composition,
            plugin_bridge_lifecycle_state: selection.plugin_bridge_lifecycle_state,
            compiled_project_plugin_plan: selection.compiled_project_plugin_plan,
            preference_storage_backend: None,
        })
    }

    pub(crate) fn for_config_with_runtime_plugin_and_feature_registrations(
        config: &EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Result<Self, CoreError> {
        let config = resolve_product_host_config(config)?;
        Self::for_resolved_config_with_runtime_plugin_and_feature_registrations(
            &config,
            registrations,
            feature_registrations,
        )
    }

    pub(super) fn for_resolved_config_with_runtime_plugin_and_feature_registrations(
        config: &ResolvedProductHostConfig,
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
            plugin_group: plugin_group_for_config(config, &selection.composition)?,
            runtime_module_composition: selection.composition,
            plugin_bridge_lifecycle_state: selection.plugin_bridge_lifecycle_state,
            compiled_project_plugin_plan: selection.compiled_project_plugin_plan,
            preference_storage_backend: None,
        })
    }

    pub fn with_preference_storage_backend(
        mut self,
        backend: Arc<dyn PreferenceStorageBackend>,
    ) -> Self {
        self.preference_storage_backend = Some(HostPreferenceStorageBackend::new(backend));
        self
    }

    pub fn plugin_group(&self) -> &ResolvedPluginGroup {
        &self.plugin_group
    }

    pub fn runtime_plugin_availability(&self) -> &RuntimePluginAvailabilityReport {
        self.runtime_module_composition
            .runtime_plugin_availability()
    }

    pub fn runtime_module_composition_identity(&self) -> &RuntimeModuleCompositionIdentity {
        self.runtime_module_composition.identity()
    }

    pub fn runtime_plugin_bridge_lifecycle_state(
        &self,
    ) -> Option<&RuntimePluginBridgeLifecycleState> {
        self.plugin_bridge_lifecycle_state.as_ref()
    }

    pub fn compiled_project_plugin_plan(&self) -> Option<Arc<CompiledProjectPluginPlan>> {
        self.compiled_project_plugin_plan.as_ref().map(Arc::clone)
    }

    pub fn module_selection_report(&self) -> EntryModuleSelectionReport {
        let platform_config = platform_config_for_entry_config(&self.config);
        EntryModuleSelectionReport {
            product_role: self.config.role(),
            product_config_provenance: self.config.provenance().clone(),
            profile: self.config.profile(),
            run_mode: self.run_mode(),
            runtime_profile: self.config.runtime_profile(),
            target_mode: self.config.target_mode(),
            preference_storage_backend: planned_preference_storage_backend(
                &platform_config,
                self.preference_storage_backend.as_ref(),
            ),
            platform_config,
            window_descriptor: self.config.window_descriptor().clone(),
            plugin_group: self.plugin_group.name().to_string(),
            runtime_plugin_availability: self
                .runtime_module_composition
                .runtime_plugin_availability()
                .clone(),
            runtime_module_composition_identity: self.runtime_module_composition.identity().clone(),
            runtime_module_warnings: self.runtime_module_composition.warning_messages(),
            modules: self
                .runtime_module_composition
                .module_descriptors()
                .iter()
                .cloned()
                .map(EntryModuleSelection::from)
                .collect(),
        }
    }

    fn store_entry_config(&self, runtime: &CoreRuntime) -> Result<(), CoreError> {
        let runtime_handle = runtime.handle();
        runtime_handle.store_config(
            PLATFORM_CONFIG_KEY,
            &platform_config_for_entry_config(&self.config),
        )?;
        runtime_handle.store_config(RENDER_PROFILE_CONFIG_KEY, self.config.render_profile())?;
        runtime_handle.store_config(
            PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,
            self.config.window_descriptor(),
        )?;
        #[cfg(not(feature = "target-editor-host"))]
        let _ = runtime;
        #[cfg(feature = "target-editor-host")]
        if matches!(self.config.profile(), EntryProfile::Editor) {
            if let Some(subsystems) = self.config.editor_enabled_subsystems() {
                runtime.store_config_value(
                    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
                    serde_json::json!(subsystems),
                );
            }
            runtime.store_config_value(
                zircon_editor::EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY,
                serde_json::json!(self.config.editor_runtime_sandbox_enabled()),
            );
        }
        Ok(())
    }
}

impl EngineEntry for BuiltinEngineEntry {
    fn profile(&self) -> EntryProfile {
        self.config.profile()
    }

    fn run_mode(&self) -> EntryRunMode {
        self.config.profile().into()
    }

    fn modules(&self) -> &[Arc<dyn EngineModule>] {
        self.runtime_module_composition.modules()
    }

    fn module_descriptors(&self) -> Vec<ModuleDescriptor> {
        self.runtime_module_composition
            .module_descriptors()
            .to_vec()
    }

    fn bootstrap(&self) -> Result<CoreHandle, CoreError> {
        let runtime = CoreRuntime::new();
        let descriptors = self.runtime_module_composition.module_descriptors();
        let platform_config = platform_config_for_entry_config(&self.config);

        self.store_entry_config(&runtime)?;
        runtime.replace_devtools_plugin_catalog_entries(builtin_plugin_catalog_entries());
        if let Some(state) = self.plugin_bridge_lifecycle_state.clone() {
            runtime.install_runtime_module_lifecycle_observer(Arc::new(state));
        }
        let preference_storage_backend = preference_storage_backend_for_bootstrap(
            &platform_config,
            self.preference_storage_backend.as_ref(),
        );
        for descriptor in descriptors {
            runtime.register_module(descriptor_with_preference_storage_backend(
                descriptor.clone(),
                preference_storage_backend.as_ref(),
            )?)?;
        }
        runtime.activate_registered_modules()?;
        self.store_entry_config(&runtime)?;

        Ok(runtime.handle())
    }
}

fn descriptor_with_preference_storage_backend(
    mut descriptor: ModuleDescriptor,
    backend: Option<&Arc<dyn PreferenceStorageBackend>>,
) -> Result<ModuleDescriptor, CoreError> {
    if descriptor.name != PLATFORM_MODULE_NAME {
        return Ok(descriptor);
    }
    let Some(backend) = backend else {
        return Ok(descriptor);
    };
    let driver = descriptor
        .drivers
        .iter_mut()
        .find(|driver| driver.name.as_str() == PLATFORM_DRIVER_NAME)
        .ok_or_else(|| {
            CoreError::Initialization(
                "platform preference storage".to_owned(),
                "platform descriptor does not own its canonical driver".to_owned(),
            )
        })?;
    let backend = Arc::clone(backend);
    driver.factory = factory(move |core| {
        let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
        let driver = PlatformDriver::with_preference_storage_backend(
            core.task_graph().worker_pool().clone(),
            Arc::clone(&backend),
        )
        .map_err(|error| {
            CoreError::Initialization("platform preference storage".to_owned(), error.to_string())
        })?;
        Ok(Arc::new(driver) as _)
    });
    Ok(descriptor)
}

fn builtin_plugin_catalog_entries() -> Vec<RuntimeDevtoolsPluginCatalogEntry> {
    RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .map(|descriptor| RuntimeDevtoolsPluginCatalogEntry {
            package_id: descriptor.package_id().to_string(),
            display_name: descriptor.display_name().to_string(),
            crate_name: descriptor.crate_name().to_string(),
            capabilities: descriptor.capabilities().to_vec(),
            target_modes: descriptor
                .target_modes()
                .iter()
                .map(|mode| format!("{:?}", mode))
                .collect(),
        })
        .collect()
}

fn plugin_group_for_config(
    config: &ResolvedProductHostConfig,
    composition: &RuntimeModuleCompositionPlan,
) -> Result<ResolvedPluginGroup, CoreError> {
    ResolvedPluginGroup::from_compiled_module_graph(
        plugin_group_name_for_config(config),
        composition.modules().to_vec(),
        composition.module_descriptors().to_vec(),
    )
    .map_err(plugin_group_core_error)
}

fn plugin_group_name_for_config(config: &ResolvedProductHostConfig) -> &'static str {
    match config.runtime_profile() {
        Some(RuntimeProfileId::Minimal) => return "MinimalPlugins",
        Some(RuntimeProfileId::Dev) => return "DevPlugins",
        _ => {}
    }
    match config.profile() {
        EntryProfile::Editor | EntryProfile::Runtime => "DefaultPlugins",
        EntryProfile::Headless => "HeadlessPlugins",
    }
}

fn platform_config_for_entry_config(config: &ResolvedProductHostConfig) -> PlatformConfig {
    let headless = matches!(config.target_mode(), RuntimeTargetMode::ServerRuntime);
    PlatformConfig {
        enabled: !matches!(config.runtime_profile(), Some(RuntimeProfileId::Minimal)),
        target: config.platform_target(),
        target_mode: config.target_mode(),
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

pub(super) fn resolve_product_host_config(
    config: &EntryConfig,
) -> Result<ResolvedProductHostConfig, CoreError> {
    config.resolve().map_err(product_host_config_core_error)
}

pub(super) fn product_host_config_core_error(error: ProductHostConfigError) -> CoreError {
    CoreError::Initialization(
        "zircon_app product host config".to_owned(),
        error.to_string(),
    )
}
