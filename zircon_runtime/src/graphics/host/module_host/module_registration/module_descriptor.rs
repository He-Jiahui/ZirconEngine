use std::sync::Arc;

use zircon_core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_manager::{RenderFrameworkHandle, RenderingManagerHandle};
use zircon_module::{dependency_on, factory, qualified_name};

use crate::asset::ASSET_MODULE_NAME;

use super::super::create::create_render_framework;
use super::super::driver::WgpuDriver;
use super::super::rendering_manager::WgpuRenderingManager;
use super::graphics_core_error::graphics_core_error;
use super::service_names::{GRAPHICS_MODULE_NAME, RENDER_FRAMEWORK_NAME};

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
        qualified_name(
            GRAPHICS_MODULE_NAME,
            ServiceKind::Manager,
            "RenderFramework",
        ),
        StartupMode::Immediate,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        )],
        factory(|core| {
            let render_framework = create_render_framework(core)
                .map_err(|error| graphics_core_error(RENDER_FRAMEWORK_NAME, error))?;
            Ok(Arc::new(RenderFrameworkHandle::new(render_framework)) as ServiceObject)
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
