#[path = "module_validation/capabilities.rs"]
mod capabilities;
#[path = "module_validation/crate_name.rs"]
mod crate_name;
#[path = "module_validation/names.rs"]
mod names;
#[path = "module_validation/target_modes.rs"]
mod target_modes;

pub(in crate::plugin::runtime_plugin) use capabilities::validate_runtime_plugin_module_capabilities;
pub(in crate::plugin::runtime_plugin) use crate_name::validate_runtime_plugin_module_crate_name;
pub(in crate::plugin::runtime_plugin) use names::validate_runtime_plugin_module_name;
pub(in crate::plugin::runtime_plugin) use target_modes::validate_runtime_plugin_module_target_modes;
