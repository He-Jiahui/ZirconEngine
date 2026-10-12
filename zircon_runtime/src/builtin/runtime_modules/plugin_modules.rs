#[path = "plugin_modules/descriptor_backed.rs"]
mod descriptor_backed;
#[path = "plugin_modules/loader.rs"]
mod loader;

pub(super) use descriptor_backed::descriptor_backed_module;
pub(super) use loader::module_for_plugin;
