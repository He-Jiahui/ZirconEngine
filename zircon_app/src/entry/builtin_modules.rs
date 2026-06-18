use std::sync::Arc;

use zircon_runtime::builtin::{
    runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports,
    runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports,
    runtime_modules_for_target, runtime_modules_for_target_with_linked_plugins,
    runtime_modules_for_target_with_plugin_and_feature_registration_reports,
    runtime_modules_for_target_with_plugin_registration_reports, RuntimeTargetMode,
};
use zircon_runtime::core::{CoreError, ModuleDescriptor};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::plugin::{
    ProjectPluginManifest, RuntimePluginAvailabilityReport, RuntimePluginBridgeLifecycleState,
    RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

use super::{entry_profile::EntryProfile, EntryConfig};

pub(super) struct BuiltinModuleSelection {
    pub modules: Vec<Arc<dyn EngineModule>>,
    pub runtime_plugin_availability: RuntimePluginAvailabilityReport,
    pub plugin_bridge_lifecycle_state: Option<RuntimePluginBridgeLifecycleState>,
}

pub(super) fn builtin_modules_for_config(
    config: &EntryConfig,
) -> Result<BuiltinModuleSelection, CoreError> {
    let manifest = config.project_plugin_manifest();
    let report = runtime_modules_for_target(config.target_mode, manifest.as_ref());
    for warning in &report.warnings {
        eprintln!("[zircon_app] runtime plugin warning: {warning}");
    }
    let required_missing = report.effective_required_missing();
    if !required_missing.is_empty() {
        return Err(CoreError::Initialization(
            "zircon_app runtime module selection".to_string(),
            report.required_missing_summary(),
        ));
    }

    let runtime_plugin_availability = report.runtime_plugin_availability;
    let modules = report.modules;
    #[cfg(feature = "target-editor-host")]
    let mut modules = modules;
    if matches!(config.profile, EntryProfile::Editor) {
        #[cfg(feature = "target-editor-host")]
        {
            modules.push(Arc::new(zircon_editor::EditorModule));
        }
        #[cfg(not(feature = "target-editor-host"))]
        {
            eprintln!(
                "[zircon_app] editor profile requested but target-editor-host feature is disabled"
            );
        }
    }

    Ok(BuiltinModuleSelection {
        modules,
        runtime_plugin_availability,
        plugin_bridge_lifecycle_state: None,
    })
}

pub(super) fn builtin_modules_for_config_with_runtime_plugin_registrations(
    config: &EntryConfig,
    registrations: &[RuntimePluginRegistrationReport],
) -> Result<BuiltinModuleSelection, CoreError> {
    let manifest = config.project_plugin_manifest();
    let effective_manifest = project_manifest_for_plugin_selection(config, &manifest);
    let report = if let Some(runtime_profile) = config.runtime_profile() {
        runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports(
            runtime_profile,
            &effective_manifest,
            registrations,
        )
    } else {
        runtime_modules_for_target_with_plugin_registration_reports(
            config.target_mode,
            manifest.as_ref(),
            registrations,
        )
    };
    for warning in &report.warnings {
        eprintln!("[zircon_app] runtime plugin warning: {warning}");
    }
    let required_missing = report.effective_required_missing();
    if !required_missing.is_empty() {
        return Err(CoreError::Initialization(
            "zircon_app runtime module selection".to_string(),
            report.required_missing_summary(),
        ));
    }

    let runtime_plugin_availability = report.runtime_plugin_availability;
    let mut modules = report.modules;
    #[cfg(feature = "target-editor-host")]
    if matches!(config.profile, EntryProfile::Editor) {
        modules.push(Arc::new(zircon_editor::EditorModule));
    }
    #[cfg(not(feature = "target-editor-host"))]
    if matches!(config.profile, EntryProfile::Editor) {
        eprintln!(
            "[zircon_app] editor profile requested but target-editor-host feature is disabled"
        );
    }
    let active_registrations = registrations
        .iter()
        .filter(|registration| {
            registration.project_selection.enabled
                && registration
                    .project_selection
                    .supports_target(config.target_mode)
        })
        .collect::<Vec<_>>();
    for registration in &active_registrations {
        for diagnostic in &registration.diagnostics {
            eprintln!(
                "[zircon_app] linked runtime plugin {} diagnostic: {diagnostic}",
                registration.package_manifest.id
            );
        }
    }
    for registration in active_registrations {
        for descriptor in registration.extensions.modules() {
            modules.push(Arc::new(DescriptorBackedEngineModule::new(
                descriptor.clone(),
            )));
        }
    }
    let plugin_bridge_lifecycle_state = Some(plugin_bridge_lifecycle_state_for_selection(
        registrations,
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
        &effective_manifest,
        config.target_mode,
    ));

    Ok(BuiltinModuleSelection {
        modules,
        runtime_plugin_availability,
        plugin_bridge_lifecycle_state,
    })
}

pub(super) fn builtin_modules_for_config_with_runtime_plugin_and_feature_registrations(
    config: &EntryConfig,
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
) -> Result<BuiltinModuleSelection, CoreError> {
    let manifest = config.project_plugin_manifest();
    let effective_manifest = project_manifest_for_plugin_selection(config, &manifest);
    let report = if let Some(runtime_profile) = config.runtime_profile() {
        runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports(
            runtime_profile,
            &effective_manifest,
            registrations,
            feature_registrations,
        )
    } else {
        runtime_modules_for_target_with_plugin_and_feature_registration_reports(
            config.target_mode,
            manifest.as_ref(),
            registrations,
            feature_registrations,
        )
    };
    for warning in &report.warnings {
        eprintln!("[zircon_app] runtime plugin warning: {warning}");
    }
    let required_missing = report.effective_required_missing();
    if !required_missing.is_empty() {
        return Err(CoreError::Initialization(
            "zircon_app runtime module selection".to_string(),
            report.required_missing_summary(),
        ));
    }
    let errors = report.effective_errors();
    if !errors.is_empty() {
        return Err(CoreError::Initialization(
            "zircon_app runtime feature selection".to_string(),
            errors.join("; "),
        ));
    }

    let runtime_plugin_availability = report.runtime_plugin_availability;
    let mut modules = report.modules;
    #[cfg(feature = "target-editor-host")]
    if matches!(config.profile, EntryProfile::Editor) {
        modules.push(Arc::new(zircon_editor::EditorModule));
    }
    #[cfg(not(feature = "target-editor-host"))]
    if matches!(config.profile, EntryProfile::Editor) {
        eprintln!(
            "[zircon_app] editor profile requested but target-editor-host feature is disabled"
        );
    }
    let active_registrations = registrations
        .iter()
        .filter(|registration| {
            registration.project_selection.enabled
                && registration
                    .project_selection
                    .supports_target(config.target_mode)
        })
        .collect::<Vec<_>>();
    for registration in &active_registrations {
        for diagnostic in &registration.diagnostics {
            eprintln!(
                "[zircon_app] linked runtime plugin {} diagnostic: {diagnostic}",
                registration.package_manifest.id
            );
        }
    }
    for registration in active_registrations {
        for descriptor in registration.extensions.modules() {
            modules.push(Arc::new(DescriptorBackedEngineModule::new(
                descriptor.clone(),
            )));
        }
    }

    let catalog = RuntimePluginCatalog::from_registration_reports(
        registrations.iter().cloned(),
        feature_registrations.iter().cloned(),
    );
    let feature_report = catalog.feature_dependency_report(&effective_manifest, config.target_mode);
    for registration in feature_registrations.iter().filter(|registration| {
        feature_report
            .available_features
            .iter()
            .any(|id| id == &registration.manifest.id)
    }) {
        for diagnostic in &registration.diagnostics {
            eprintln!(
                "[zircon_app] linked runtime plugin feature {} diagnostic: {diagnostic}",
                registration.manifest.id
            );
        }
        for descriptor in registration.extensions.modules() {
            modules.push(Arc::new(DescriptorBackedEngineModule::new(
                descriptor.clone(),
            )));
        }
    }
    let extension_report =
        catalog.runtime_extensions_for_project(&effective_manifest, config.target_mode);
    let plugin_bridge_lifecycle_state = Some(
        RuntimePluginBridgeLifecycleState::from_extension_report(catalog, extension_report),
    );

    Ok(BuiltinModuleSelection {
        modules,
        runtime_plugin_availability,
        plugin_bridge_lifecycle_state,
    })
}

pub(super) fn builtin_modules_for_config_with_available_runtime_plugins(
    config: &EntryConfig,
    available_plugin_ids: &[String],
) -> Result<BuiltinModuleSelection, CoreError> {
    let manifest = config.project_plugin_manifest();
    let report = runtime_modules_for_target_with_linked_plugins(
        config.target_mode,
        manifest.as_ref(),
        available_plugin_ids.iter().map(String::as_str),
    );
    for warning in &report.warnings {
        eprintln!("[zircon_app] runtime plugin warning: {warning}");
    }
    let required_missing = report.effective_required_missing();
    if !required_missing.is_empty() {
        return Err(CoreError::Initialization(
            "zircon_app runtime module selection".to_string(),
            report.required_missing_summary(),
        ));
    }

    let runtime_plugin_availability = report.runtime_plugin_availability;
    let modules = report.modules;
    #[cfg(feature = "target-editor-host")]
    let mut modules = modules;
    if matches!(config.profile, EntryProfile::Editor) {
        #[cfg(feature = "target-editor-host")]
        {
            modules.push(Arc::new(zircon_editor::EditorModule));
        }
        #[cfg(not(feature = "target-editor-host"))]
        {
            eprintln!(
                "[zircon_app] editor profile requested but target-editor-host feature is disabled"
            );
        }
    }

    Ok(BuiltinModuleSelection {
        modules,
        runtime_plugin_availability,
        plugin_bridge_lifecycle_state: None,
    })
}

fn plugin_bridge_lifecycle_state_for_selection(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    manifest: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> RuntimePluginBridgeLifecycleState {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        registrations.iter().cloned(),
        feature_registrations,
    );
    let extension_report = catalog.runtime_extensions_for_project(manifest, target);
    RuntimePluginBridgeLifecycleState::from_extension_report(catalog, extension_report)
}

fn project_manifest_for_plugin_selection(
    config: &EntryConfig,
    manifest: &Option<ProjectPluginManifest>,
) -> ProjectPluginManifest {
    manifest.clone().unwrap_or_else(|| {
        config
            .runtime_profile()
            .map(|runtime_profile| {
                zircon_runtime::plugin::RuntimeProfileDescriptor::for_id(runtime_profile)
                    .project_manifest()
            })
            .unwrap_or_default()
    })
}

#[derive(Debug)]
struct DescriptorBackedEngineModule {
    name: &'static str,
    description: &'static str,
    descriptor: ModuleDescriptor,
}

impl DescriptorBackedEngineModule {
    fn new(descriptor: ModuleDescriptor) -> Self {
        let name = Box::leak(descriptor.name.clone().into_boxed_str());
        let description = Box::leak(descriptor.description.clone().into_boxed_str());
        Self {
            name,
            description,
            descriptor,
        }
    }
}

impl EngineModule for DescriptorBackedEngineModule {
    fn module_name(&self) -> &'static str {
        self.name
    }

    fn module_description(&self) -> &'static str {
        self.description
    }

    fn descriptor(&self) -> ModuleDescriptor {
        self.descriptor.clone()
    }
}
