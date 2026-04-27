use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::EntityId;

use super::super::World;
use super::value_conversion::normalized_identifier;

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
        let entries = self.property_entries(entity);
        let target_component = normalized_identifier(property_path.component());
        let target_segments = property_path
            .property_segments()
            .iter()
            .map(|segment| normalized_identifier(segment))
            .collect::<Vec<_>>();

        entries
            .into_iter()
            .find(|entry| {
                normalized_identifier(entry.property_path.component()) == target_component
                    && entry
                        .property_path
                        .property_segments()
                        .iter()
                        .map(|segment| normalized_identifier(segment))
                        .collect::<Vec<_>>()
                        == target_segments
            })
            .map(|entry| entry.value)
            .or_else(|| self.dynamic_component_property(entity, property_path))
            .ok_or_else(|| {
                format!("property `{property_path}` is not available on entity {entity}")
            })
    }
}
