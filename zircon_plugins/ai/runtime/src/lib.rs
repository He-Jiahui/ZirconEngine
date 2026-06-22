pub const PLUGIN_ID: &str = "ai";

mod capability;
mod manager;
mod module;

pub use capability::{
    AI_BEHAVIOR_TREE_CAPABILITY, AI_BLACKBOARD_CAPABILITY, AI_PERCEPTION_CAPABILITY,
    AI_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use manager::DefaultAiManager;
pub use module::{
    module_descriptor, AiDriver, AiModule, AI_DRIVER_NAME, AI_MANAGER_NAME, AI_MODULE_NAME,
};

#[derive(Clone, Debug)]
pub struct AiRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl AiRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for AiRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_runtime::plugin::RuntimePlugin for AiRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "AI",
        zircon_runtime::builtin::RuntimePluginId::Ai,
        "zircon_plugin_ai_runtime",
    )
    .with_category("runtime")
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Experimental)
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(AI_RUNTIME_CAPABILITY)
    .with_capability(AI_BEHAVIOR_TREE_CAPABILITY)
    .with_capability(AI_BLACKBOARD_CAPABILITY)
    .with_capability(AI_PERCEPTION_CAPABILITY)
    .with_capability_status(
        zircon_runtime::plugin::CapabilityStatusManifest::new(
            AI_RUNTIME_CAPABILITY,
            zircon_runtime::plugin::CapabilityStatus::Partial,
        )
        .with_note(
            "Foundational AI runtime package; deterministic selector/sequence/task execution and blackboard decorators are available while advanced node families remain partial.",
        ),
    )
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        AI_BEHAVIOR_TREE_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        AI_BLACKBOARD_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        AI_PERCEPTION_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(AiRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

#[cfg(test)]
mod tests;
