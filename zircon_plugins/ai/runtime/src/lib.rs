pub const PLUGIN_ID: &str = "ai";

mod capability;
mod manager;
mod module;
mod plugin;

pub use capability::{
    AI_BEHAVIOR_TREE_CAPABILITY, AI_BLACKBOARD_CAPABILITY, AI_PERCEPTION_CAPABILITY,
    AI_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use manager::DefaultAiManager;
pub use module::{
    module_descriptor, AiDriver, AiModule, AI_DRIVER_NAME, AI_MANAGER_NAME, AI_MODULE_NAME,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, AiRuntimePlugin, AI_DIST_CRATE_NAME,
    AI_DIST_RUNTIME_ENTRY,
};

#[cfg(test)]
mod tests;
