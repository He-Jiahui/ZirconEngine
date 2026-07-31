//! Graphics module registration and manager services.

mod module_host;

pub use module_host::{
    RENDER_FRAMEWORK_NAME, RENDERING_MANAGER_NAME, module_descriptor,
    module_descriptor_with_render_features,
};
