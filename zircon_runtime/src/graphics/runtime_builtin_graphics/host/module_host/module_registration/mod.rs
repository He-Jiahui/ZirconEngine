mod graphics_core_error;
mod module_descriptor;
mod service_names;

pub use module_descriptor::{module_descriptor, module_descriptor_with_render_features};
pub use service_names::{GRAPHICS_MODULE_NAME, RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME};
