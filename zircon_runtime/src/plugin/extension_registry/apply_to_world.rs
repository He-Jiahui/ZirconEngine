use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::World;

use super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn apply_component_types_to_world(
        &self,
        world: &mut World,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        for component in self.components() {
            world
                .register_component_type(component.clone())
                .map_err(|error| {
                    if error.contains("already registered") {
                        RuntimeExtensionRegistryError::DuplicateComponentType(
                            component.type_id.clone(),
                        )
                    } else {
                        RuntimeExtensionRegistryError::InvalidComponentType(error)
                    }
                })?;
        }
        Ok(())
    }
}
