pub mod behavior_tree;
/// Dense, schema-compiled blackboard storage and slot contracts.
pub mod blackboard;
mod capability;
mod manager;
mod module;
pub mod perception;
mod plugin;
mod tick_lod;

pub use capability::{
    AI_BEHAVIOR_TREE_CAPABILITY, AI_BLACKBOARD_CAPABILITY, AI_DECLARATION,
    AI_PERCEPTION_CAPABILITY, AI_RUNTIME_CAPABILITY, NATIVE_PLUGIN_ID,
    NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST,
    PLUGIN_ID, RUNTIME_CAPABILITIES,
};
pub use manager::DefaultAiManager;
pub use module::{
    module_descriptor, module_descriptor_with_manager, AiDriver, AiModule, AI_DRIVER_NAME,
    AI_MANAGER_NAME, AI_MODULE_NAME,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, AiRuntimePlugin,
    AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID, AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA,
    AI_BEHAVIOR_TICK_SYSTEM, AI_DIST_CRATE_NAME, AI_DIST_RUNTIME_ENTRY, AI_EVENT_NAMESPACE,
    AI_PERCEPTION_TICK_SYSTEM, BT_NODE_RESULT_EVENT_ID, BT_NODE_RESULT_PAYLOAD_SCHEMA,
};
pub use tick_lod::AiBehaviorTickLod;

#[cfg(test)]
mod tests;
