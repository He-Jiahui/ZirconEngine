use std::sync::Arc;

use crate::capability::{
    PHYSICS_CONSTRAINTS_CAPABILITY, PHYSICS_DECLARATION, PHYSICS_OVERLAP_CAPABILITY,
    PHYSICS_RAYCAST_CAPABILITY, PHYSICS_RUNTIME_CAPABILITY, PHYSICS_SHAPE_CAST_CAPABILITY,
    PHYSICS_SKELETAL_JOINTS_CAPABILITY, PHYSICS_TRIGGER_EVENTS_CAPABILITY, RUNTIME_CAPABILITIES,
    RUNTIME_CRATE_NAME,
};
use crate::manager::DefaultPhysicsManager;
use crate::module::module_descriptor_with_manager;
use crate::runtime_system::{
    PHYSICS_STEP_SYSTEM, PHYSICS_SYNC_TO_SCENE_SYSTEM, PHYSICS_SYSTEM_SET, register_runtime_systems,
};
use zircon_runtime::core::framework::physics::{PHYSICS_QUERY_INTERFACE_ID, PhysicsQueryInterface};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, PluginDistributionManifest, PluginModuleManifest,
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};

pub const PLUGIN_RUNTIME_MODULE_NAME: &str = "physics.runtime";
pub const PHYSICS_DIST_CRATE_NAME: &str = "zircon_plugin_physics_dist";
pub const PHYSICS_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_physics_runtime_entry_v3";
const PHYSICS_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct PhysicsRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
    manager: Arc<DefaultPhysicsManager>,
}

impl PhysicsRuntimePlugin {
    pub fn new() -> Self {
        let manager = Arc::new(DefaultPhysicsManager::new(None));
        Self {
            descriptor: runtime_plugin_descriptor_with_manager(manager.clone()),
            manager,
        }
    }
}

impl Default for PhysicsRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for PhysicsRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("physics.dist", PHYSICS_DIST_CRATE_NAME)
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
            engine_compat: PHYSICS_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: PHYSICS_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: PHYSICS_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
            .module(PLUGIN_RUNTIME_MODULE_NAME)?;
        let manager: Arc<dyn PhysicsQueryInterface> = self.manager.clone();
        module.export_interface::<dyn PhysicsQueryInterface>(manager)?;
        register_runtime_systems(&mut module)
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    runtime_plugin_descriptor_with_manager(Arc::new(DefaultPhysicsManager::new(None)))
}

fn runtime_plugin_descriptor_with_manager(
    manager: Arc<DefaultPhysicsManager>,
) -> RuntimePluginDescriptor {
    PHYSICS_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor_with_manager(Some(manager)))
        .with_capability_status(CapabilityStatusManifest::new(
            PHYSICS_RUNTIME_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .with_capability_status(CapabilityStatusManifest::new(
            PHYSICS_RAYCAST_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .with_capability_status(CapabilityStatusManifest::new(
            PHYSICS_OVERLAP_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .with_capability_status(CapabilityStatusManifest::new(
            PHYSICS_SHAPE_CAST_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .with_capability_status(CapabilityStatusManifest::new(
            PHYSICS_TRIGGER_EVENTS_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .with_capability_status(CapabilityStatusManifest::new(
            PHYSICS_CONSTRAINTS_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .with_capability_status(CapabilityStatusManifest::new(
            PHYSICS_SKELETAL_JOINTS_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .with_provided_interface_id(PHYSICS_QUERY_INTERFACE_ID)
        .with_system_sets([PHYSICS_SYSTEM_SET])
        .with_system_anchors([PHYSICS_STEP_SYSTEM, PHYSICS_SYNC_TO_SCENE_SYSTEM])
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(PhysicsRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
