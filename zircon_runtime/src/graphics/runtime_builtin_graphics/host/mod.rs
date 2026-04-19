//! Graphics module registration and manager services.

mod module_host;

pub use module_host::{
    module_descriptor, GRAPHICS_MODULE_NAME, RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME,
};
