mod builtin_pbr_wgsl;
mod builtin_reference;
mod builtin_resources;
mod resource_manager_with_builtins;

pub(in crate::asset::pipeline::manager) use builtin_pbr_wgsl::builtin_pbr_wgsl;
use builtin_reference::builtin_reference;
pub(in crate::asset::pipeline::manager) use builtin_resources::builtin_resources;
pub(in crate::asset::pipeline::manager) use resource_manager_with_builtins::resource_manager_with_builtins;
