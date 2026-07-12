use std::sync::Arc;

use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::core::framework::bridge::PluginInterface;
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, ExportPackagingStrategy,
    PluginDistributionManifest, PluginMaturity, PluginModuleManifest, PluginPackageManifest,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};

use crate::behavior_tree::BehaviorNodeRegistry;
use crate::{
    module_descriptor_with_manager, DefaultAiManager, AI_BEHAVIOR_TREE_CAPABILITY,
    AI_BLACKBOARD_CAPABILITY, AI_PERCEPTION_CAPABILITY, AI_RUNTIME_CAPABILITY, PLUGIN_ID,
    RUNTIME_CAPABILITIES,
};

mod registration;

use registration::{ai_event_catalog, register_runtime_extensions};
pub use registration::{AI_BEHAVIOR_TICK_SYSTEM, AI_EVENT_NAMESPACE};

pub const AI_DIST_CRATE_NAME: &str = "zircon_plugin_ai_dist";
pub const AI_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_ai_runtime_entry_v3";
const AI_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct AiRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
    manager: Arc<DefaultAiManager>,
}

impl AiRuntimePlugin {
    pub fn new() -> Self {
        Self::with_manager(Arc::new(DefaultAiManager::default()))
    }

    pub fn with_behavior_node_catalog(catalog: crate::behavior_tree::BehaviorNodeCatalog) -> Self {
        Self::with_manager(Arc::new(DefaultAiManager::with_behavior_node_catalog(
            catalog,
        )))
    }

    fn with_manager(manager: Arc<DefaultAiManager>) -> Self {
        Self {
            descriptor: runtime_plugin_descriptor_with_manager(manager.clone()),
            manager,
        }
    }

    pub fn manager(&self) -> Arc<DefaultAiManager> {
        self.manager.clone()
    }
}

impl Default for AiRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for AiRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        manifest = manifest.with_event_catalog(ai_event_catalog());
        manifest
            .default_packaging
            .push(ExportPackagingStrategy::NativeDynamic);
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("ai.dist", AI_DIST_CRATE_NAME)
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
        );
        manifest.with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: AI_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: AI_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: AI_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        register_runtime_extensions(registry, self.manager.clone())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    runtime_plugin_descriptor_with_manager(Arc::new(DefaultAiManager::default()))
}

fn runtime_plugin_descriptor_with_manager(
    manager: Arc<DefaultAiManager>,
) -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "AI",
        RuntimePluginId::Ai,
        "zircon_plugin_ai_runtime",
    )
    .with_module_descriptor(module_descriptor_with_manager(Some(manager)))
    .with_system_anchors([AI_BEHAVIOR_TICK_SYSTEM])
    .with_category("runtime")
    .with_maturity(PluginMaturity::Experimental)
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability(AI_RUNTIME_CAPABILITY)
    .with_capability(AI_BEHAVIOR_TREE_CAPABILITY)
    .with_capability(AI_BLACKBOARD_CAPABILITY)
    .with_capability(AI_PERCEPTION_CAPABILITY)
    .with_capability_status(
        CapabilityStatusManifest::new(AI_RUNTIME_CAPABILITY, CapabilityStatus::Partial).with_note(
            "Foundational AI runtime package; deterministic selector/sequence/task execution and blackboard decorators are available while advanced node families remain partial.",
        ),
    )
    .with_capability_status(CapabilityStatusManifest::new(
        AI_BEHAVIOR_TREE_CAPABILITY,
        CapabilityStatus::Partial,
    ))
    .with_capability_status(CapabilityStatusManifest::new(
        AI_BLACKBOARD_CAPABILITY,
        CapabilityStatus::Partial,
    ))
    .with_capability_status(CapabilityStatusManifest::new(
        AI_PERCEPTION_CAPABILITY,
        CapabilityStatus::Partial,
    ))
    .with_provided_interface_id(<dyn BehaviorNodeRegistry as PluginInterface>::INTERFACE_ID)
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(AiRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
