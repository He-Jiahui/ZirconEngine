use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::{SceneError, World};
use zircon_runtime_interface::reflect::ReflectError;

use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn apply_component_types_to_world(
        &mut self,
        world: &mut World,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.finalize();
        self.apply_finalized_component_types_to_world(world)
    }

    pub(super) fn apply_finalized_component_types_to_world(
        &self,
        world: &mut World,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        debug_assert!(self.is_finalized());
        for component in self.components() {
            world
                .register_component_type(component.clone())
                .map_err(|error| match error {
                    SceneError::DuplicateComponentType { .. }
                    | SceneError::Reflect(ReflectError::DuplicateTypePath { .. }) => {
                        RuntimeExtensionRegistryError::DuplicateComponentType(
                            component.type_id.clone(),
                        )
                    }
                    error => RuntimeExtensionRegistryError::InvalidComponentType(error.to_string()),
                })?;
        }
        Ok(())
    }
}
