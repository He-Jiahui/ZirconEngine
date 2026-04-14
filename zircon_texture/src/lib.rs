//! Texture module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, ModuleDescriptor};

pub const TEXTURE_MODULE_NAME: &str = "TextureModule";

#[derive(Clone, Debug, Default)]
pub struct TextureConfig {
    pub enabled: bool,
}

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        TEXTURE_MODULE_NAME,
        "Texture formats, conversion, and upload prep",
        "TextureDriver",
        "TextureManager",
    )
}
