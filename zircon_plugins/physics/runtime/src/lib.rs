pub const PLUGIN_ID: &str = "physics";

mod config;
mod module;
mod physics_interface;
mod service_types;

pub use config::PhysicsConfig;
pub use module::{
    module_descriptor, PhysicsModule, PHYSICS_DRIVER_NAME, PHYSICS_MANAGER_NAME,
    PHYSICS_MODULE_NAME, PHYSICS_SETTINGS_CONFIG_KEY,
};
pub use physics_interface::PhysicsInterface;
pub use service_types::{
    build_world_sync_state, integrate_builtin_physics_steps, DefaultPhysicsManager, PhysicsDriver,
    PhysicsTickPlan, JOLT_ENABLED,
};

#[cfg(test)]
pub(crate) use module::DEFAULT_PHYSICS_MANAGER_NAME;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct PhysicsRuntimePlugin {
    descriptor: zircon_runtime::RuntimePluginDescriptor,
}

impl PhysicsRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::RuntimePlugin for PhysicsRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::RuntimePluginDescriptor {
    zircon_runtime::RuntimePluginDescriptor::new(
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
}

pub fn runtime_plugin() -> PhysicsRuntimePlugin {
    PhysicsRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::PluginPackageManifest {
    zircon_runtime::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::ProjectPluginSelection {
    zircon_runtime::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::RuntimePluginRegistrationReport {
    zircon_runtime::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &["runtime.plugin.physics"]
}
