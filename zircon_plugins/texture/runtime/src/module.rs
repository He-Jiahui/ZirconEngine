use std::sync::Arc;

use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode};
use zircon_runtime::engine_module::{factory, qualified_name};

use crate::capability::TEXTURE_PLUGIN_DECLARATION;
use crate::manager::DefaultTextureManager;

pub const TEXTURE_MODULE_NAME: &str = TEXTURE_PLUGIN_DECLARATION.module_name();
pub const TEXTURE_MANAGER_NAME: &str = "texture.runtime.Manager.TextureManager";

pub fn module_descriptor() -> ModuleDescriptor {
    TEXTURE_PLUGIN_DECLARATION
        .module_descriptor()
        .with_manager(ManagerDescriptor::new(
            qualified_name(TEXTURE_MODULE_NAME, ServiceKind::Manager, "TextureManager"),
            StartupMode::Lazy,
            Vec::new(),
            factory(|_| Ok(Arc::new(DefaultTextureManager) as ServiceObject)),
        ))
}
