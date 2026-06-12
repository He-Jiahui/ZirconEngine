use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::EntityId;

use super::super::World;

impl World {
    pub fn property(
        &self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
    ) -> Result<ScenePropertyValue, String> {
        self.property_impl(entity, property_path)
    }

    fn property_impl(
        &self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
    ) -> Result<ScenePropertyValue, String> {
        let target_component = property_path.component();
        let target_segments = property_path.property_segments();

        if let Some(value) = self.property_entry_value(entity, target_component, target_segments) {
            return Ok(value);
        }

        if let Some(value) = self.dynamic_component_property(entity, property_path) {
            return Ok(value);
        }

        Err(format!(
            "property `{property_path}` is not available on entity {entity}"
        ))
    }
}
