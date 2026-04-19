//! Graphics module-host registration absorbed into the runtime layer.

mod host;

use crate::engine_module::{EngineModule, ModuleDescriptor};

pub use host::{
    module_descriptor, GRAPHICS_MODULE_NAME, RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct GraphicsModule;

impl EngineModule for GraphicsModule {
    fn module_name(&self) -> &'static str {
        GRAPHICS_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Rendering device abstraction and scene rendering"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
