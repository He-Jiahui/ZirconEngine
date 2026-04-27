//! Graphics module registration and manager services.

mod module_host;

pub use module_host::{
    module_descriptor, module_descriptor_with_render_features, GRAPHICS_MODULE_NAME,
    RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME,
};
