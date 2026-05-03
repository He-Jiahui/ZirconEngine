use std::sync::Arc;

use crate::core::manager::{RenderFrameworkHandle, RenderingManagerHandle};
use crate::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use crate::engine_module::{dependency_on, factory, qualified_name};
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};

use crate::asset::ASSET_MODULE_NAME;

use super::super::create::create_render_framework_with_render_features;
use super::super::driver::WgpuDriver;
use super::super::rendering_manager::WgpuRenderingManager;
use super::graphics_core_error::graphics_core_error;
use super::service_names::{GRAPHICS_MODULE_NAME, RENDER_FRAMEWORK_NAME};

pub fn module_descriptor() -> ModuleDescriptor {
    module_descriptor_with_render_features(Vec::new(), Vec::new(), Vec::new(), Vec::new())
}

pub fn module_descriptor_with_render_features(
    render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
    virtual_geometry_runtime_providers: impl IntoIterator<
        Item = VirtualGeometryRuntimeProviderRegistration,
    >,
) -> ModuleDescriptor {
    let render_features = Arc::new(render_features.into_iter().collect::<Vec<_>>());
    let render_pass_executors = Arc::new(render_pass_executors.into_iter().collect::<Vec<_>>());
    let hybrid_gi_runtime_providers =
        Arc::new(hybrid_gi_runtime_providers.into_iter().collect::<Vec<_>>());
    let virtual_geometry_runtime_providers = Arc::new(
        virtual_geometry_runtime_providers
            .into_iter()
            .collect::<Vec<_>>(),
    );
    ModuleDescriptor::new(
        GRAPHICS_MODULE_NAME,
        "Rendering device abstraction and scene rendering",
    )
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
            let render_pass_executors = Arc::clone(&render_pass_executors);
            let hybrid_gi_runtime_providers = Arc::clone(&hybrid_gi_runtime_providers);
            let virtual_geometry_runtime_providers =
                Arc::clone(&virtual_geometry_runtime_providers);
            move |core| {
                let render_framework = create_render_framework_with_render_features(
                    core,
                    render_features.to_vec(),
                    render_pass_executors.to_vec(),
                    hybrid_gi_runtime_providers.to_vec(),
                    virtual_geometry_runtime_providers.to_vec(),
                )
                .map_err(|error| graphics_core_error(RENDER_FRAMEWORK_NAME, error))?;
                Ok(Arc::new(RenderFrameworkHandle::new(render_framework)) as ServiceObject)
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
            Ok(Arc::new(RenderingManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}
