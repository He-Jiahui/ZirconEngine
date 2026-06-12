mod component;

use crate::plugin::{RuntimeExtensionRegistry, RuntimeExtensionRegistryError};
use crate::scene::World;

impl RuntimeExtensionRegistry {
    pub fn apply_to_world(
        &mut self,
        world: &mut World,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.apply_component_types_to_world(world)?;
        for (_, resource) in self.plugin_resources() {
            resource.apply(world);
        }
        for (_, event) in self.plugin_events() {
            event.apply(world);
        }
        for (_, system) in self.plugin_systems() {
            let system = system.build(world).map_err(|error| {
                RuntimeExtensionRegistryError::WorldRegistration(error.to_string())
            })?;
            world
                .register_boxed_native_system(system)
                .map_err(|error| {
                    RuntimeExtensionRegistryError::WorldRegistration(error.to_string())
                })?;
        }
        for (_, system) in self.plugin_runtime_systems() {
            let system = system.build();
            world
                .register_boxed_runtime_scene_system(system)
                .map_err(|error| {
                    RuntimeExtensionRegistryError::WorldRegistration(error.to_string())
                })?;
        }
        Ok(())
    }
}
