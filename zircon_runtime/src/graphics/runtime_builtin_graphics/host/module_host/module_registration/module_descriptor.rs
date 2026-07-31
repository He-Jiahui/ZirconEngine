use std::sync::Arc;

use crate::core::framework::platform::PLATFORM_MODULE_NAME;
use crate::core::framework::render::{
    GRAPHICS_MODULE_NAME, GeometrySourceDescriptor, RenderFramework, RenderingManager,
    ShadingModelDescriptor,
};
use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::manager::RegisteredManagerService;
use crate::core::runtime::ServiceObject;
use crate::core::{
    DriverDescriptor, InitLevel, ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor,
    ServiceKind, StartupMode,
};
use crate::engine_module::{dependency_on, factory, qualified_name};
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};

use super::super::create::create_render_framework_with_render_features;
use super::super::driver::WgpuDriver;
use super::super::rendering_manager::WgpuRenderingManager;
use super::graphics_core_error::graphics_core_error;
use super::service_names::RENDER_FRAMEWORK_NAME;
use crate::asset::ASSET_MODULE_NAME;

pub fn module_descriptor() -> ModuleDescriptor {
    module_descriptor_with_render_features(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

pub fn module_descriptor_with_render_features(
    render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
    plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
    render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
    hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
    solari_runtime_providers: impl IntoIterator<Item = SolariRuntimeProviderRegistration>,
    virtual_geometry_runtime_providers: impl IntoIterator<
        Item = VirtualGeometryRuntimeProviderRegistration,
    >,
) -> ModuleDescriptor {
    let render_features = Arc::new(render_features.into_iter().collect::<Vec<_>>());
    let plugin_geometry_sources = Arc::new(plugin_geometry_sources.into_iter().collect::<Vec<_>>());
    let plugin_shading_models = Arc::new(plugin_shading_models.into_iter().collect::<Vec<_>>());
    let render_pass_executors = Arc::new(render_pass_executors.into_iter().collect::<Vec<_>>());
    let runtime_prepare_collectors =
        Arc::new(runtime_prepare_collectors.into_iter().collect::<Vec<_>>());
    let hybrid_gi_runtime_providers =
        Arc::new(hybrid_gi_runtime_providers.into_iter().collect::<Vec<_>>());
    let solari_runtime_providers =
        Arc::new(solari_runtime_providers.into_iter().collect::<Vec<_>>());
    let virtual_geometry_runtime_providers = Arc::new(
        virtual_geometry_runtime_providers
            .into_iter()
            .collect::<Vec<_>>(),
    );
    ModuleDescriptor::new(
        GRAPHICS_MODULE_NAME,
        "Rendering device abstraction and scene rendering",
    )
    .with_init_level(InitLevel::Scene)
    .with_module_dependency(ModuleDependencySpec::named(PLATFORM_MODULE_NAME))
    .with_module_dependency(ModuleDependencySpec::named(ASSET_MODULE_NAME))
    .with_module_dependency(ModuleDependencySpec::named(SCENE_MODULE_NAME))
    .with_driver(DriverDescriptor::new(
        qualified_name(GRAPHICS_MODULE_NAME, ServiceKind::Driver, "WgpuDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(WgpuDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            GRAPHICS_MODULE_NAME,
            ServiceKind::Manager,
            "RenderFramework",
        ),
        StartupMode::Lazy,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        )],
        factory({
            let render_features = Arc::clone(&render_features);
            let plugin_geometry_sources = Arc::clone(&plugin_geometry_sources);
            let plugin_shading_models = Arc::clone(&plugin_shading_models);
            let render_pass_executors = Arc::clone(&render_pass_executors);
            let runtime_prepare_collectors = Arc::clone(&runtime_prepare_collectors);
            let hybrid_gi_runtime_providers = Arc::clone(&hybrid_gi_runtime_providers);
            let solari_runtime_providers = Arc::clone(&solari_runtime_providers);
            let virtual_geometry_runtime_providers =
                Arc::clone(&virtual_geometry_runtime_providers);
            move |core| {
                let render_framework = create_render_framework_with_render_features(
                    core,
                    render_features.to_vec(),
                    plugin_geometry_sources.to_vec(),
                    plugin_shading_models.to_vec(),
                    render_pass_executors.to_vec(),
                    runtime_prepare_collectors.to_vec(),
                    hybrid_gi_runtime_providers.to_vec(),
                    solari_runtime_providers.to_vec(),
                    virtual_geometry_runtime_providers.to_vec(),
                )
                .map_err(|error| graphics_core_error(RENDER_FRAMEWORK_NAME, error))?;
                Ok(
                    Arc::new(RegisteredManagerService::<dyn RenderFramework>::new(
                        render_framework,
                    )) as ServiceObject,
                )
            }
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            GRAPHICS_MODULE_NAME,
            ServiceKind::Manager,
            "RenderingManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| {
            let manager = Arc::new(WgpuRenderingManager);
            Ok(
                Arc::new(RegisteredManagerService::<dyn RenderingManager>::new(
                    manager,
                )) as ServiceObject,
            )
        }),
    ))
}
