use std::sync::Arc;

use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::builtin::{
    manifest_with_mode_baseline, RuntimeModuleCompositionCompiler, RuntimeModuleCompositionPlan,
    RuntimeModuleCompositionRejection,
};
use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};
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
    let catalog = RuntimePluginCatalog::from_registration_reports(
        registrations.iter().cloned(),
        std::iter::empty(),
    );
    let plan = catalog.compiled_project_plan(effective_manifest, config.target_mode());
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
    let plan = catalog.compiled_project_plan(&effective_manifest, config.target_mode());
    builtin_modules_for_config_with_compiled_project_plugin_plan(
        config,
        catalog,
        plan,
        "zircon_app runtime feature selection",
    )
}

fn builtin_modules_for_config_with_compiled_project_plugin_plan(
    config: &ResolvedProductHostConfig,
    catalog: RuntimePluginCatalog,
    plan: Arc<CompiledProjectPluginPlan>,
    context: &str,
) -> Result<BuiltinModuleSelection, CoreError> {
    let mut compiler = RuntimeModuleCompositionCompiler::new(&plan);
    if let Some(runtime_profile) = config.runtime_profile() {
        compiler = compiler.for_runtime_profile(runtime_profile);
    }
    compiler = compiler.with_host_modules(host_modules_for_config(config)?);
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
    effective_project_plugin_manifest_with_render_profile(
        config.target_mode(),
        config.project_plugin_manifest(),
        config.render_profile(),
    )
}

pub(super) fn effective_project_plugin_manifest_with_render_profile(
    target_mode: RuntimeTargetMode,
    manifest: Option<&ProjectPluginManifest>,
    render_profile: &RenderProfileBundle,
) -> ProjectPluginManifest {
    let mut effective_manifest = manifest_with_mode_baseline(target_mode, manifest);
    let render_profile_overlay =
        render_profile_runtime_plugin_overlay(&effective_manifest, target_mode, render_profile);
    for selection in render_profile_overlay.selections {
        effective_manifest.set_enabled(selection);
    }
    effective_manifest
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
