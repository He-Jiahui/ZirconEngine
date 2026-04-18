mod builtin_pbr_wgsl;
mod builtin_reference;
mod builtin_resources;
mod editor_icon_builtin_resources;
mod editor_icon_locators;
mod resource_manager_with_builtins;

pub(in crate::pipeline::manager) use builtin_pbr_wgsl::builtin_pbr_wgsl;
use builtin_reference::builtin_reference;
pub(in crate::pipeline::manager) use builtin_resources::builtin_resources;
use editor_icon_builtin_resources::editor_icon_builtin_resources;
use editor_icon_locators::BUILTIN_EDITOR_ICON_LOCATORS;
pub(in crate::pipeline::manager) use resource_manager_with_builtins::resource_manager_with_builtins;
