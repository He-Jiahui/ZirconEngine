use zircon_module::{EngineModule, ModuleDescriptor};

pub use zircon_texture::{TextureDriver, TextureManager};

pub const TEXTURE_MODULE_NAME: &str = "TextureModule";
pub const TEXTURE_DRIVER_NAME: &str = "TextureModule.Driver.TextureDriver";
pub const TEXTURE_MANAGER_NAME: &str = "TextureModule.Manager.TextureManager";

#[derive(Clone, Debug, Default)]
pub struct TextureConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextureModule;

pub fn module_descriptor() -> ModuleDescriptor {
    super::module_descriptor_with_driver_and_manager::<TextureDriver, TextureManager>(
        TEXTURE_MODULE_NAME,
        "Texture formats, conversion, and upload prep",
        "TextureDriver",
        "TextureManager",
    )
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
