//! Graphics module registration and manager services.

#[path = "module_host/mod.rs"]
mod module_host;

pub use module_host::{
    module_descriptor, module_descriptor_with_render_features, RENDERING_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME,
};
