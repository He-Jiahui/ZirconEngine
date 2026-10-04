//! Graphics module-host registration absorbed into the runtime layer.

mod graphics_module;
mod host;

pub use graphics_module::GraphicsModule;
pub use host::{
    module_descriptor, module_descriptor_with_render_features, RENDERING_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME,
};
