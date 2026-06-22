pub const PLUGIN_ID: &str = "physics";
pub const PLUGIN_RUNTIME_MODULE_NAME: &str = "physics.runtime";
pub const PHYSICS_SETTINGS_CONFIG_KEY: &str = "physics.settings";

mod backend;
mod capability;
mod manager;
mod module;
mod query_contact;
mod runtime_system;
mod trigger;

pub use backend::JOLT_ENABLED;
pub use capability::{
    PHYSICS_CONSTRAINTS_CAPABILITY, PHYSICS_OVERLAP_CAPABILITY, PHYSICS_RAYCAST_CAPABILITY,
    PHYSICS_RUNTIME_CAPABILITY, PHYSICS_SHAPE_CAST_CAPABILITY, PHYSICS_SKELETAL_JOINTS_CAPABILITY,
    PHYSICS_TRIGGER_EVENTS_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use manager::{
    build_world_sync_state, integrate_builtin_physics_steps, DefaultPhysicsManager, PhysicsTickPlan,
};
pub use module::{
    module_descriptor, PhysicsDriver, PhysicsModule, DEFAULT_PHYSICS_MANAGER_NAME,
    PHYSICS_DRIVER_NAME, PHYSICS_MODULE_NAME,
};
pub use runtime_system::{
    register_runtime_system, PhysicsRuntimeSystem, PHYSICS_STEP_SYSTEM, PHYSICS_SYSTEM_SET,
};
pub use zircon_runtime::core::manager::PHYSICS_MANAGER_NAME;

#[derive(Clone, Debug)]
pub struct PhysicsRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl PhysicsRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for PhysicsRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        let owner = registry.intern_plugin_module(PLUGIN_RUNTIME_MODULE_NAME)?;
        registry.register_module(module_descriptor())?;
        register_runtime_system(registry, owner)
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Physics",
        zircon_runtime::builtin::RuntimePluginId::Physics,
        "zircon_plugin_physics_runtime",
    )
    .with_category("runtime")
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Experimental)
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(PHYSICS_RUNTIME_CAPABILITY)
    .with_capability(PHYSICS_RAYCAST_CAPABILITY)
    .with_capability(PHYSICS_OVERLAP_CAPABILITY)
    .with_capability(PHYSICS_SHAPE_CAST_CAPABILITY)
    .with_capability(PHYSICS_TRIGGER_EVENTS_CAPABILITY)
    .with_capability(PHYSICS_CONSTRAINTS_CAPABILITY)
    .with_capability(PHYSICS_SKELETAL_JOINTS_CAPABILITY)
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        PHYSICS_RUNTIME_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        PHYSICS_RAYCAST_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        PHYSICS_OVERLAP_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        PHYSICS_SHAPE_CAST_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        PHYSICS_TRIGGER_EVENTS_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        PHYSICS_CONSTRAINTS_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        PHYSICS_SKELETAL_JOINTS_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_system_sets([PHYSICS_SYSTEM_SET])
    .with_system_anchors([PHYSICS_STEP_SYSTEM])
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(PhysicsRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::CoreRuntime;

    use super::*;

    #[test]
    fn physics_registration_contributes_runtime_module() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == PHYSICS_MODULE_NAME));
        assert!(report
            .extensions
            .plugin_runtime_systems()
            .any(|(owner, system)| {
                report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
                    && system.id == PHYSICS_STEP_SYSTEM
                    && system.stage == zircon_runtime::scene::SystemStage::FixedUpdate
            }));
        assert_eq!(
            report.package_manifest.modules[0].system_sets,
            vec![PHYSICS_SYSTEM_SET.to_string()]
        );
        assert_eq!(
            report.package_manifest.modules[0].system_anchors,
            vec![PHYSICS_STEP_SYSTEM.to_string()]
        );
        assert_eq!(
            report.package_manifest.modules[0].target_modes,
            vec![
                zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
                zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
            ]
        );
        assert_eq!(report.package_manifest.category, "runtime");
        assert_eq!(
            report.package_manifest.maturity,
            zircon_runtime::plugin::PluginMaturity::Experimental
        );
        for capability in [
            "runtime.plugin.physics",
            "runtime.capability.physics.raycast",
            "runtime.capability.physics.overlap",
            "runtime.capability.physics.shape_cast",
            "runtime.capability.physics.trigger_events",
            "runtime.capability.physics.constraints",
            "runtime.capability.physics.skeletal_joints",
        ] {
            assert!(report
                .package_manifest
                .capabilities
                .contains(&capability.to_string()));
            assert!(report
                .package_manifest
                .capability_statuses
                .iter()
                .any(|status| {
                    status.capability == capability
                        && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
                }));
        }
    }

    #[test]
    fn physics_module_resolves_manager() {
        let runtime = CoreRuntime::new();
        runtime.register_module(module_descriptor()).unwrap();
        runtime.activate_module(PHYSICS_MODULE_NAME).unwrap();

        runtime
            .handle()
            .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
            .unwrap();
    }
}
