mod config;
mod descriptor;
mod module_type;

pub use config::InputConfig;
pub use descriptor::{module_descriptor, INPUT_DRIVER_NAME, INPUT_MANAGER_NAME, INPUT_MODULE_NAME};
pub use module_type::InputModule;
