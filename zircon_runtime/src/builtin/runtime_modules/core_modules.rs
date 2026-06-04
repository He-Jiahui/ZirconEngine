use std::sync::Arc;

use crate::asset::AssetImporterRegistry;
use crate::engine_module::EngineModule;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::{asset, foundation, graphics, input, platform, scene, script};

use super::RuntimeTargetMode;

pub fn runtime_core_modules() -> Vec<Arc<dyn EngineModule>> {
    runtime_core_modules_for_target(RuntimeTargetMode::ClientRuntime)
}

pub(super) fn runtime_core_modules_for_target(
    target: RuntimeTargetMode,
) -> Vec<Arc<dyn EngineModule>> {
    runtime_core_modules_for_target_with_render_features(
        target,
        &AssetImporterRegistry::default(),
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
    )
}

pub(super) fn minimal_profile_runtime_modules() -> Vec<Arc<dyn EngineModule>> {
    vec![
        Arc::new(foundation::FoundationModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::modules::TasksModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::modules::TimeModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::modules::FrameCountModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::modules::DiagnosticsCoreModule) as Arc<dyn EngineModule>,
    ]
}

pub(super) fn runtime_core_modules_for_target_with_render_features(
    target: RuntimeTargetMode,
    asset_importers: &AssetImporterRegistry,
    render_features: &[RenderFeatureDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    runtime_prepare_collectors: &[RuntimePrepareCollectorRegistration],
    hybrid_gi_runtime_providers: &[HybridGiRuntimeProviderRegistration],
    solari_runtime_providers: &[SolariRuntimeProviderRegistration],
    virtual_geometry_runtime_providers: &[VirtualGeometryRuntimeProviderRegistration],
) -> Vec<Arc<dyn EngineModule>> {
    let mut modules: Vec<Arc<dyn EngineModule>> = vec![
        Arc::new(foundation::FoundationModule),
        Arc::new(platform::PlatformModule),
        Arc::new(input::InputModule),
        Arc::new(asset::AssetModule::with_asset_importers(
            asset_importers.clone(),
        )),
        Arc::new(scene::SceneModule),
    ];
    if target != RuntimeTargetMode::ServerRuntime {
        modules.push(Arc::new(
            graphics::GraphicsModule::with_render_extensions_and_runtime_providers(
                render_features.iter().cloned(),
                render_pass_executors.iter().cloned(),
                runtime_prepare_collectors.iter().cloned(),
                hybrid_gi_runtime_providers.iter().cloned(),
                solari_runtime_providers.iter().cloned(),
                virtual_geometry_runtime_providers.iter().cloned(),
            ),
        ));
    }
    modules.push(Arc::new(script::ScriptModule));
    modules
}
