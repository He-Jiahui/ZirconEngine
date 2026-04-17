use std::sync::Arc;

use zircon_asset::ASSET_MODULE_NAME;
use zircon_core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_manager::RenderingManagerHandle;
use zircon_module::{dependency_on, factory, qualified_name};
use zircon_render_server::RenderServerHandle;

use super::create_render_server::create_render_server;
use super::graphics_core_error::graphics_core_error;
use super::service_names::{GRAPHICS_MODULE_NAME, RENDER_SERVER_NAME};
use super::wgpu_driver::WgpuDriver;
use super::wgpu_rendering_manager::WgpuRenderingManager;

pub fn module_descriptor() -> ModuleDescriptor {
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
        qualified_name(GRAPHICS_MODULE_NAME, ServiceKind::Manager, "RenderServer"),
        StartupMode::Immediate,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        )],
        factory(|core| {
            let render_server = create_render_server(core)
                .map_err(|error| graphics_core_error(RENDER_SERVER_NAME, error))?;
            Ok(Arc::new(RenderServerHandle::new(render_server)) as ServiceObject)
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
