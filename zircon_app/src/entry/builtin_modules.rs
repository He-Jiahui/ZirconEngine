use std::sync::Arc;

use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::builtin::{
    default_manifest_for_target, manifest_with_mode_baseline, RuntimeModuleCompositionCompiler,
    RuntimeModuleCompositionPlan, RuntimeModuleCompositionRejection,
};
use zircon_runtime::core::framework::project::{
    ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use zircon_runtime::core::framework::render::{RenderProductFeature, RenderProfileBundle};
use zircon_runtime::core::{framework::platform::RuntimeTargetMode, CoreError};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::plugin::{
    CompiledProjectPluginPlan, RuntimePluginBridgeLifecycleState, RuntimePluginCatalog,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

use super::{entry_profile::EntryProfile, ResolvedProductHostConfig};

pub(super) struct BuiltinModuleSelection {
    pub composition: RuntimeModuleCompositionPlan,
    pub plugin_bridge_lifecycle_state: Option<RuntimePluginBridgeLifecycleState>,
    pub compiled_project_plugin_plan: Option<Arc<CompiledProjectPluginPlan>>,
}

pub(super) fn builtin_modules_for_config_with_runtime_plugin_registrations(
    config: &ResolvedProductHostConfig,
    registrations: &[RuntimePluginRegistrationReport],
) -> Result<BuiltinModuleSelection, CoreError> {
    let effective_manifest = effective_project_plugin_manifest(config);
    builtin_modules_for_config_with_effective_manifest_and_runtime_plugin_registrations(
        config,
        &effective_manifest,
        registrations,
    )
}

pub(super) fn builtin_modules_for_config_with_effective_manifest_and_runtime_plugin_registrations(
    config: &ResolvedProductHostConfig,
    effective_manifest: &ProjectPluginManifest,
    registrations: &[RuntimePluginRegistrationReport],
) -> Result<BuiltinModuleSelection, CoreError> {
    builtin_modules_for_config_with_effective_manifest_and_runtime_plugin_and_feature_registrations(
        config,
        effective_manifest,
        registrations,
        std::iter::empty(),
    )
}

pub(super) fn builtin_modules_for_config_with_effective_manifest_and_runtime_plugin_and_feature_registrations(
    config: &ResolvedProductHostConfig,
    effective_manifest: &ProjectPluginManifest,
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
) -> Result<BuiltinModuleSelection, CoreError> {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        registrations.iter().cloned(),
        feature_registrations,
    );
    let plan = compiled_plugin_plan_for_config(config, &catalog, effective_manifest)?;
    builtin_modules_for_config_with_compiled_project_plugin_plan(
        config,
        catalog,
        plan,
        "zircon_app runtime module selection",
    )
}

pub(super) fn builtin_modules_for_config_with_runtime_plugin_and_feature_registrations(
    config: &ResolvedProductHostConfig,
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
) -> Result<BuiltinModuleSelection, CoreError> {
    let effective_manifest = effective_project_plugin_manifest(config);
    let catalog = RuntimePluginCatalog::from_registration_reports(
        registrations.iter().cloned(),
        feature_registrations.iter().cloned(),
    );
    let plan = compiled_plugin_plan_for_config(config, &catalog, &effective_manifest)?;
    builtin_modules_for_config_with_compiled_project_plugin_plan(
        config,
        catalog,
        plan,
        "zircon_app runtime feature selection",
    )
}

fn compiled_plugin_plan_for_config(
    config: &ResolvedProductHostConfig,
    catalog: &RuntimePluginCatalog,
    manifest: &ProjectPluginManifest,
) -> Result<Arc<CompiledProjectPluginPlan>, CoreError> {
    if let Some(id) = config.runtime_profile() {
        let mut profile = zircon_runtime::plugin::RuntimeProfileDescriptor::for_id(id);
        if let Some(minimum) = config.runtime_plugin_minimum_maturity() {
            profile = profile.with_minimum_maturity(minimum);
        }
        Ok(catalog.compiled_project_plan_for_runtime_profile(manifest, &profile))
    } else if config.runtime_plugin_minimum_maturity().is_some() {
        Err(CoreError::Initialization(
            "zircon_app runtime plugin maturity policy".to_owned(),
            "a minimum maturity override requires an explicit Runtime profile".to_owned(),
        ))
    } else {
        Ok(catalog.compiled_project_plan(manifest, config.target_mode()))
    }
}

fn builtin_modules_for_config_with_compiled_project_plugin_plan(
    config: &ResolvedProductHostConfig,
    catalog: RuntimePluginCatalog,
    plan: Arc<CompiledProjectPluginPlan>,
    context: &str,
) -> Result<BuiltinModuleSelection, CoreError> {
    let compiler = RuntimeModuleCompositionCompiler::new(&plan)
        .with_host_modules(host_modules_for_config(config)?);
    let composition = compiler
        .compile()
        .map_err(|rejection| composition_rejection_core_error(context, rejection))?;
    let plugin_bridge_lifecycle_state =
        Some(RuntimePluginBridgeLifecycleState::from_extension_report(
            catalog,
            plan.runtime_extensions_handle(),
        ));

    Ok(BuiltinModuleSelection {
        composition,
        plugin_bridge_lifecycle_state,
        compiled_project_plugin_plan: Some(plan),
    })
}

fn host_modules_for_config(
    config: &ResolvedProductHostConfig,
) -> Result<Vec<Arc<dyn EngineModule>>, CoreError> {
    let mut modules: Vec<Arc<dyn EngineModule>> = Vec::new();
    if matches!(
        config.runtime_profile(),
        Some(zircon_runtime::core::framework::project::RuntimeProfileId::Dev)
    ) {
        modules.push(Arc::new(
            zircon_runtime::core::runtime::modules::LogDiagnosticsModule,
        ));
    }
    #[cfg(feature = "target-editor-host")]
    if matches!(config.profile(), EntryProfile::Editor) {
        modules.push(Arc::new(zircon_editor::EditorModule));
    }
    #[cfg(not(feature = "target-editor-host"))]
    if matches!(config.profile(), EntryProfile::Editor) {
        return Err(CoreError::Initialization(
            "zircon_app runtime host module selection".to_owned(),
            "editor profile requires the target-editor-host feature".to_owned(),
        ));
    }
    Ok(modules)
}

fn composition_rejection_core_error(
    context: &str,
    rejection: RuntimeModuleCompositionRejection,
) -> CoreError {
    CoreError::Initialization(context.to_owned(), rejection.to_string())
}

pub(super) fn effective_project_plugin_manifest(
    config: &ResolvedProductHostConfig,
) -> ProjectPluginManifest {
    effective_project_plugin_manifest_for_profile(
        config.target_mode(),
        config.project_plugin_manifest(),
        config.render_profile(),
        config.runtime_profile(),
    )
}

pub(super) fn effective_project_plugin_manifest_with_render_profile(
    target_mode: RuntimeTargetMode,
    manifest: Option<&ProjectPluginManifest>,
    render_profile: &RenderProfileBundle,
) -> ProjectPluginManifest {
    effective_project_plugin_manifest_for_profile(target_mode, manifest, render_profile, None)
}

fn effective_project_plugin_manifest_for_profile(
    target_mode: RuntimeTargetMode,
    manifest: Option<&ProjectPluginManifest>,
    render_profile: &RenderProfileBundle,
    runtime_profile: Option<RuntimeProfileId>,
) -> ProjectPluginManifest {
    let mut effective_manifest = if runtime_profile == Some(RuntimeProfileId::Minimal) {
        minimal_runtime_plugin_manifest(target_mode, manifest)
    } else {
        manifest_with_mode_baseline(target_mode, manifest)
    };
    let render_profile_overlay =
        render_profile_runtime_plugin_overlay(&effective_manifest, target_mode, render_profile);
    for selection in render_profile_overlay.selections {
        effective_manifest.set_enabled(selection);
    }
    admit_builtin_runtime_ui_selection(&mut effective_manifest);
    effective_manifest
}

fn minimal_runtime_plugin_manifest(
    target_mode: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
) -> ProjectPluginManifest {
    let mut manifest = default_manifest_for_target(target_mode);
    let baseline_runtime_ids = manifest
        .selections
        .iter()
        .filter_map(|selection| RuntimePluginId::parse_key(&selection.id))
        .collect::<Vec<_>>();
    manifest.selections.retain(|selection| {
        !RuntimePluginId::parse_key(&selection.id).is_some_and(|runtime_id| {
            runtime_id == RuntimePluginId::Ui || runtime_id == RuntimePluginId::UiDocumentImporter
        })
    });

    if let Some(manifest_override) = manifest_override {
        for selection in &manifest_override.selections {
            let mut selection = selection.clone();
            if let Some(runtime_id) = RuntimePluginId::parse_key(&selection.id)
                .filter(|runtime_id| baseline_runtime_ids.contains(runtime_id))
            {
                selection.id = runtime_id.key().to_owned();
            }
            manifest.set_enabled(selection);
        }
    }

    manifest
}

fn admit_builtin_runtime_ui_selection(manifest: &mut ProjectPluginManifest) {
    if let Some(selection) = manifest
        .selections
        .iter_mut()
        .find(|selection| selection.id == RuntimePluginId::Ui.key())
    {
        if selection.is_runtime_builtin_domain() {
            // UI is composed as a built-in runtime module, not a first-party plugin provider.
            selection.required = false;
        }
    }
}

pub(super) fn render_profile_runtime_plugin_overlay(
    manifest: &ProjectPluginManifest,
    target_mode: RuntimeTargetMode,
    render_profile: &RenderProfileBundle,
) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: runtime_plugins_for_render_profile(render_profile)
            .filter(|runtime_plugin| {
                !manifest
                    .selections
                    .iter()
                    .filter_map(|selection| RuntimePluginId::parse_key(&selection.id))
                    .any(|selection_id| selection_id.key() == runtime_plugin.key())
            })
            .map(|runtime_plugin| {
                ProjectPluginSelection::runtime_plugin(runtime_plugin, true, false)
                    .with_target_modes([target_mode])
            })
            .collect(),
    }
}

fn runtime_plugins_for_render_profile(
    render_profile: &RenderProfileBundle,
) -> impl Iterator<Item = RuntimePluginId> + '_ {
    [
        (
            RenderProductFeature::VirtualGeometry,
            RuntimePluginId::VirtualGeometry,
        ),
        (
            RenderProductFeature::HybridGlobalIllumination,
            RuntimePluginId::HybridGi,
        ),
        (RenderProductFeature::Solari, RuntimePluginId::Solari),
    ]
    .into_iter()
    .filter_map(|(feature, runtime_plugin)| {
        render_profile
            .has_feature(feature)
            .then_some(runtime_plugin)
    })
}
