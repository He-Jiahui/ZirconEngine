pub const PLUGIN_ID: &str = "physics";
pub const PLUGIN_RUNTIME_MODULE_NAME: &str = "physics.runtime";
pub const PHYSICS_SETTINGS_CONFIG_KEY: &str = "physics.settings";

mod backend;
mod manager;
mod module;
mod query_contact;
mod runtime_system;
mod trigger;

pub use backend::JOLT_ENABLED;
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
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Physics",
        zircon_runtime::RuntimePluginId::Physics,
        "zircon_plugin_physics_runtime",
    )
    .with_category("runtime")
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Experimental)
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.physics")
    .with_capability("runtime.capability.physics.raycast")
    .with_capability("runtime.capability.physics.overlap")
    .with_capability("runtime.capability.physics.shape_cast")
    .with_capability("runtime.capability.physics.trigger_events")
    .with_capability("runtime.capability.physics.constraints")
    .with_capability("runtime.capability.physics.skeletal_joints")
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        "runtime.plugin.physics",
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        "runtime.capability.physics.raycast",
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        "runtime.capability.physics.overlap",
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        "runtime.capability.physics.shape_cast",
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        "runtime.capability.physics.trigger_events",
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        "runtime.capability.physics.constraints",
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        "runtime.capability.physics.skeletal_joints",
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_system_sets([PHYSICS_SYSTEM_SET])
    .with_system_anchors([PHYSICS_STEP_SYSTEM])
}

pub fn runtime_plugin() -> PhysicsRuntimePlugin {
    PhysicsRuntimePlugin::new()
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
        "runtime.plugin.physics",
        "runtime.capability.physics.raycast",
        "runtime.capability.physics.overlap",
        "runtime.capability.physics.shape_cast",
        "runtime.capability.physics.trigger_events",
        "runtime.capability.physics.constraints",
        "runtime.capability.physics.skeletal_joints",
    ]
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
                zircon_runtime::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::RuntimeTargetMode::ServerRuntime,
                zircon_runtime::RuntimeTargetMode::EditorHost,
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
