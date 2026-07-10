use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::AssetImporterRegistry;
use crate::core::framework::error::{CoreError, CoreResult};
#[cfg(feature = "graphics")]
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
use crate::core::runtime::modules::{
    DiagnosticsCoreModule, FrameCountModule, LogModule, TasksModule, TimeModule,
};
use crate::core::sort_module_activation_order;
use crate::engine_module::EngineModule;
#[cfg(feature = "graphics")]
use crate::graphics;
#[cfg(feature = "graphics")]
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
#[cfg(feature = "script")]
use crate::script;
use crate::{asset, foundation, input, platform, scene};

use super::ids::RuntimeTargetMode;

pub fn runtime_core_modules() -> Vec<Arc<dyn EngineModule>> {
    runtime_core_modules_for_target(RuntimeTargetMode::ClientRuntime)
        .expect("built-in runtime core module descriptors must sort")
}

pub(super) fn runtime_core_modules_for_target(
    target: RuntimeTargetMode,
) -> CoreResult<Vec<Arc<dyn EngineModule>>> {
    #[cfg(feature = "graphics")]
    {
        runtime_core_modules_for_target_with_render_features(
            target,
            &AssetImporterRegistry::default(),
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
        )
    }
    #[cfg(not(feature = "graphics"))]
    {
        runtime_core_modules_for_target_with_render_features(
            target,
            &AssetImporterRegistry::default(),
        )
    }
}

pub(super) fn minimal_profile_runtime_modules() -> CoreResult<Vec<Arc<dyn EngineModule>>> {
    sort_runtime_modules_by_descriptor_order(vec![
        Arc::new(foundation::FoundationModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::runtime::modules::TasksModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::runtime::modules::TimeModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::runtime::modules::FrameCountModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::runtime::modules::DiagnosticsCoreModule) as Arc<dyn EngineModule>,
    ])
}

#[cfg(feature = "graphics")]
pub(super) fn runtime_core_modules_for_target_with_render_features(
    target: RuntimeTargetMode,
    asset_importers: &AssetImporterRegistry,
    render_features: &[RenderFeatureDescriptor],
    geometry_sources: &[GeometrySourceDescriptor],
    shading_models: &[ShadingModelDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    runtime_prepare_collectors: &[RuntimePrepareCollectorRegistration],
    hybrid_gi_runtime_providers: &[HybridGiRuntimeProviderRegistration],
    solari_runtime_providers: &[SolariRuntimeProviderRegistration],
    virtual_geometry_runtime_providers: &[VirtualGeometryRuntimeProviderRegistration],
) -> CoreResult<Vec<Arc<dyn EngineModule>>> {
    let mut modules: Vec<Arc<dyn EngineModule>> = vec![
        Arc::new(foundation::FoundationModule),
        Arc::new(LogModule),
        Arc::new(TasksModule),
        Arc::new(TimeModule),
        Arc::new(FrameCountModule),
        Arc::new(DiagnosticsCoreModule),
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
                geometry_sources.iter().cloned(),
                shading_models.iter().cloned(),
                render_pass_executors.iter().cloned(),
                runtime_prepare_collectors.iter().cloned(),
                hybrid_gi_runtime_providers.iter().cloned(),
                solari_runtime_providers.iter().cloned(),
                virtual_geometry_runtime_providers.iter().cloned(),
            ),
        ));
    }
    #[cfg(feature = "script")]
    if target != RuntimeTargetMode::ServerRuntime {
        modules.push(Arc::new(script::ScriptModule));
    }
    sort_runtime_modules_by_descriptor_order(modules)
}

#[cfg(not(feature = "graphics"))]
pub(super) fn runtime_core_modules_for_target_with_render_features(
    target: RuntimeTargetMode,
    asset_importers: &AssetImporterRegistry,
) -> CoreResult<Vec<Arc<dyn EngineModule>>> {
    let mut modules: Vec<Arc<dyn EngineModule>> = vec![
        Arc::new(foundation::FoundationModule),
        Arc::new(LogModule),
        Arc::new(TasksModule),
        Arc::new(TimeModule),
        Arc::new(FrameCountModule),
        Arc::new(DiagnosticsCoreModule),
        Arc::new(platform::PlatformModule),
        Arc::new(input::InputModule),
        Arc::new(asset::AssetModule::with_asset_importers(
            asset_importers.clone(),
        )),
        Arc::new(scene::SceneModule),
    ];
    #[cfg(feature = "script")]
    if target != RuntimeTargetMode::ServerRuntime {
        modules.push(Arc::new(script::ScriptModule));
    }
    sort_runtime_modules_by_descriptor_order(modules)
}

pub(super) fn sort_runtime_modules_by_descriptor_order(
    modules: Vec<Arc<dyn EngineModule>>,
) -> CoreResult<Vec<Arc<dyn EngineModule>>> {
    let descriptors = modules
        .iter()
        .map(|module| module.descriptor())
        .collect::<Vec<_>>();
    let order = sort_module_activation_order(&descriptors)?;
    let mut modules_by_name = modules
        .into_iter()
        .map(|module| (module.module_name().to_owned(), module))
        .collect::<HashMap<_, _>>();
    order
        .into_iter()
        .map(|module_name| {
            modules_by_name
                .remove(&module_name)
                .ok_or(CoreError::MissingModule(module_name))
        })
        .collect()
}
