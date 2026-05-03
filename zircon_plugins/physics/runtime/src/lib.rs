pub const PLUGIN_ID: &str = "physics";
pub const PHYSICS_SETTINGS_CONFIG_KEY: &str = "physics.settings";

mod backend;
mod manager;
mod module;
mod query_contact;
mod scene_hook;

pub use backend::JOLT_ENABLED;
pub use manager::{
    build_world_sync_state, integrate_builtin_physics_steps, DefaultPhysicsManager, PhysicsTickPlan,
};
pub use module::{
    module_descriptor, PhysicsDriver, PhysicsModule, DEFAULT_PHYSICS_MANAGER_NAME,
    PHYSICS_DRIVER_NAME, PHYSICS_MODULE_NAME,
};
pub use scene_hook::{scene_hook_registration, PhysicsSceneRuntimeHook};
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

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        registry.register_scene_hook(scene_hook_registration())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Physics",
        zircon_runtime::RuntimePluginId::Physics,
        "zircon_plugin_physics_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.physics")
    .with_capability("runtime.capability.physics.raycast")
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
        assert_eq!(
            report.package_manifest.modules[0].target_modes,
            vec![
                zircon_runtime::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::RuntimeTargetMode::ServerRuntime,
                zircon_runtime::RuntimeTargetMode::EditorHost,
            ]
        );
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
