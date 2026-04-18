//! Texture module scaffold with explicit core service descriptors.

use std::sync::Arc;

use zircon_module::{
    dependency_on, factory, qualified_name, DriverDescriptor, EngineModule, ManagerDescriptor,
    ModuleDescriptor, ServiceKind, StartupMode,
};

pub const TEXTURE_MODULE_NAME: &str = "TextureModule";
pub const TEXTURE_DRIVER_NAME: &str = "TextureModule.Driver.TextureDriver";
pub const TEXTURE_MANAGER_NAME: &str = "TextureModule.Manager.TextureManager";

#[derive(Clone, Debug, Default)]
pub struct TextureConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextureModule;

#[derive(Clone, Debug, Default)]
pub struct TextureDriver;

#[derive(Clone, Debug, Default)]
pub struct TextureManager;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        TEXTURE_MODULE_NAME,
        "Texture formats, conversion, and upload prep",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(TEXTURE_MODULE_NAME, ServiceKind::Driver, "TextureDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(TextureDriver::default()) as _)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(TEXTURE_MODULE_NAME, ServiceKind::Manager, "TextureManager"),
        StartupMode::Lazy,
        vec![dependency_on(
            TEXTURE_MODULE_NAME,
            ServiceKind::Driver,
            "TextureDriver",
        )],
        factory(|_| Ok(Arc::new(TextureManager::default()) as _)),
    ))
}

impl EngineModule for TextureModule {
    fn module_name(&self) -> &'static str {
        TEXTURE_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Texture formats, conversion, and upload prep"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
