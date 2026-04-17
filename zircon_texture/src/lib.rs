//! Texture module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, EngineModule, ModuleDescriptor};

pub const TEXTURE_MODULE_NAME: &str = "TextureModule";

#[derive(Clone, Debug, Default)]
pub struct TextureConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextureModule;

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
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
