mod capabilities;
mod crate_name;
mod names;
mod target_modes;

pub(in crate::plugin::runtime_plugin) use capabilities::validate_runtime_plugin_module_capabilities;
pub(in crate::plugin::runtime_plugin) use crate_name::validate_runtime_plugin_module_crate_name;
pub(in crate::plugin::runtime_plugin) use names::validate_runtime_plugin_module_name;
pub(in crate::plugin::runtime_plugin) use target_modes::validate_runtime_plugin_module_target_modes;
