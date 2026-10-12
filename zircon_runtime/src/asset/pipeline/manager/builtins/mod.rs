#[path = "builtin_pbr_wgsl.rs"]
mod builtin_pbr_wgsl;
#[path = "builtin_reference.rs"]
mod builtin_reference;
#[path = "builtin_resources.rs"]
mod builtin_resources;
#[path = "../../../manager/builtins/resource_manager_with_builtins.rs"]
mod resource_manager_with_builtins;

pub(in crate::asset::pipeline::manager) use builtin_pbr_wgsl::builtin_pbr_wgsl;
use builtin_reference::builtin_reference;
pub(in crate::asset::pipeline::manager) use builtin_resources::builtin_resource;
pub(in crate::asset::pipeline::manager) use builtin_resources::builtin_resources;
pub(in crate::asset::pipeline::manager) use resource_manager_with_builtins::resource_manager_with_builtins;
