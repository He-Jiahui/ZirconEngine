#[path = "../../../../host/module_registration/graphics_core_error.rs"]
mod graphics_core_error;
#[path = "../../../../host/module_registration/module_descriptor.rs"]
mod module_descriptor;
#[path = "../../module_registration/service_names.rs"]
mod service_names;

pub use module_descriptor::{module_descriptor, module_descriptor_with_render_features};
pub use service_names::{RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME};
