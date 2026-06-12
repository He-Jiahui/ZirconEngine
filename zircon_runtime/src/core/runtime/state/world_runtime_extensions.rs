use crate::plugin::{PluginModuleId, RuntimeExtensionRegistry, RuntimeExtensionRegistryError};
use crate::scene::World;

#[derive(Clone, Debug, Default)]
pub(crate) struct WorldRuntimeExtensionSet {
    registry: RuntimeExtensionRegistry,
}

impl WorldRuntimeExtensionSet {
    pub(crate) fn install(
        &mut self,
        extensions: &RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        merge_world_runtime_extensions(&mut self.registry, extensions)
    }

    pub(crate) fn apply_to_world(
        &self,
        world: &mut World,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let mut registry = self.registry.clone();
        registry.apply_to_world(world)
    }
}

fn merge_world_runtime_extensions(
    target: &mut RuntimeExtensionRegistry,
    source: &RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    for (owner, system) in source.plugin_runtime_systems() {
        let target_owner = intern_target_owner(target, source, owner)?;
        target.register_runtime_scene_system_registration(target_owner, system.clone())?;
    }
    Ok(())
}

fn intern_target_owner(
    target: &mut RuntimeExtensionRegistry,
    source: &RuntimeExtensionRegistry,
    owner: PluginModuleId,
) -> Result<PluginModuleId, RuntimeExtensionRegistryError> {
    let Some(module_name) = source.plugin_module_name(owner) else {
        return Err(RuntimeExtensionRegistryError::InvalidPluginModule(format!(
            "unknown plugin module owner {}",
            owner.raw()
        )));
    };
    target.intern_plugin_module(module_name.to_string())
}
