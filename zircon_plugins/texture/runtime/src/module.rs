use std::sync::Arc;

use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode};
use zircon_runtime::engine_module::{factory, qualified_name};

use crate::manager::DefaultTextureManager;

pub const TEXTURE_MODULE_NAME: &str = "texture.runtime";
pub const TEXTURE_MANAGER_NAME: &str = "texture.runtime.Manager.TextureManager";

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        TEXTURE_MODULE_NAME,
        "Texture import and runtime metadata plugin",
    )
    .with_manager(ManagerDescriptor::new(
        qualified_name(TEXTURE_MODULE_NAME, ServiceKind::Manager, "TextureManager"),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_| Ok(Arc::new(DefaultTextureManager) as ServiceObject)),
    ))
}
