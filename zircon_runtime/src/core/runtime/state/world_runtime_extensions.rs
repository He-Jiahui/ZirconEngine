use crate::plugin::{PluginModuleId, RuntimeExtensionRegistry, RuntimeExtensionRegistryError};
use crate::scene::World;

#[cfg(test)]
#[path = "world_runtime_extensions/tests.rs"]
mod tests;

#[derive(Clone, Debug)]
pub(crate) struct WorldRuntimeExtensionSet {
    registry: RuntimeExtensionRegistry,
}

impl Default for WorldRuntimeExtensionSet {
    fn default() -> Self {
        let mut registry = RuntimeExtensionRegistry::default();
        registry.finalize();
        Self { registry }
    }
}

impl WorldRuntimeExtensionSet {
    pub(crate) fn install(
        &mut self,
        extensions: &RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let mut candidate = self.registry.clone();
        merge_world_runtime_extensions(&mut candidate, extensions)?;
        candidate.finalize();
        self.registry = candidate;
        Ok(())
    }

    pub(crate) fn apply_to_world(
        &self,
        world: &mut World,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.registry.apply_finalized_to_world(world)
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
