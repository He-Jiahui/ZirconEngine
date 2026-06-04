pub const PLUGIN_ID: &str = "ai";
pub const AI_RUNTIME_CAPABILITY: &str = "runtime.plugin.ai";
pub const AI_BEHAVIOR_TREE_CAPABILITY: &str = "runtime.feature.ai.behavior_tree";
pub const AI_BLACKBOARD_CAPABILITY: &str = "runtime.feature.ai.blackboard";
pub const AI_PERCEPTION_CAPABILITY: &str = "runtime.feature.ai.perception";

mod manager;
mod module;

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

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "AI",
        zircon_runtime::RuntimePluginId::Ai,
        "zircon_plugin_ai_runtime",
    )
    .with_category("runtime")
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Experimental)
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
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
            "Foundational AI runtime package; behavior-tree execution is intentionally staged behind manager contracts.",
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
}

pub fn runtime_plugin() -> AiRuntimePlugin {
    AiRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        AI_RUNTIME_CAPABILITY,
        AI_BEHAVIOR_TREE_CAPABILITY,
        AI_BLACKBOARD_CAPABILITY,
        AI_PERCEPTION_CAPABILITY,
    ]
}

#[cfg(test)]
mod tests;
