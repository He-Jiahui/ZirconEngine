use std::collections::{BTreeMap, HashMap};

use crate::core::framework::scene::ComponentTypeDescriptor;

use super::error::{SceneError, SceneResult};

#[cfg(test)]
#[path = "component_type_registry/schema_generation_hash_tests.rs"]
mod schema_generation_hash_tests;

#[derive(Clone, Debug, Default)]
pub struct ComponentTypeRegistry {
    descriptors: BTreeMap<String, ComponentTypeDescriptor>,
    next_schema_generation: u64,
    schema_generations: HashMap<String, u64>,
}

// Schema revisions are runtime cache metadata and do not change persistent
// component-descriptor equality.
impl PartialEq for ComponentTypeRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.descriptors == other.descriptors
    }
}

impl Eq for ComponentTypeRegistry {}

impl ComponentTypeRegistry {
    pub(super) fn validate_new_descriptor(
        &self,
        descriptor: &ComponentTypeDescriptor,
    ) -> SceneResult<()> {
        if !component_type_belongs_to_plugin(&descriptor.type_id, &descriptor.plugin_id) {
            return Err(SceneError::ComponentTypePluginPrefixMismatch {
                type_id: descriptor.type_id.clone(),
                plugin_id: descriptor.plugin_id.clone(),
            });
        }
        if self.descriptors.contains_key(&descriptor.type_id) {
            return Err(SceneError::DuplicateComponentType {
                type_id: descriptor.type_id.clone(),
            });
        }
        Ok(())
    }

    pub fn register(&mut self, descriptor: ComponentTypeDescriptor) -> SceneResult<()> {
        self.validate_new_descriptor(&descriptor)?;
        let type_id = descriptor.type_id.clone();
        self.descriptors.insert(type_id.clone(), descriptor);
        self.advance_schema_generation(&type_id);
        Ok(())
    }

    pub(super) fn publish_prevalidated(&mut self, descriptor: ComponentTypeDescriptor) {
        let type_id = descriptor.type_id.clone();
        debug_assert!(self.validate_new_descriptor(&descriptor).is_ok());
        let previous = self.descriptors.insert(type_id.clone(), descriptor);
        debug_assert!(previous.is_none());
        self.advance_schema_generation(&type_id);
    }

    pub fn descriptor(&self, type_id: &str) -> Option<&ComponentTypeDescriptor> {
        self.descriptors.get(type_id)
    }

    /// Returns the revision for the declared dynamic component schema.
    ///
    /// A compiled dynamic-field writer binds this revision at compile time so a
    /// registration refresh invalidates only writers for that component type.
    pub fn schema_generation(&self, type_id: &str) -> u64 {
        self.schema_generations
            .get(type_id)
            .copied()
            .unwrap_or_default()
    }

    /// Returns the revision for the schema catalog as a whole.
    ///
    /// This is only needed by a compiled field whose type was undeclared at
    /// compile time. Once a catalog appears, that writer must rebind rather
    /// than retain the former permissive empty-registry semantics.
    pub fn schema_catalog_generation(&self) -> u64 {
        self.next_schema_generation
    }

    pub(super) fn upsert_vm_descriptor(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> SceneResult<()> {
        if !component_type_belongs_to_plugin(&descriptor.type_id, &descriptor.plugin_id) {
            return Err(SceneError::ComponentTypePluginPrefixMismatch {
                type_id: descriptor.type_id,
                plugin_id: descriptor.plugin_id,
            });
        }
        if let Some(existing) = self.descriptors.get(&descriptor.type_id) {
            if existing.plugin_id != descriptor.plugin_id {
                return Err(SceneError::DuplicateComponentType {
                    type_id: descriptor.type_id,
                });
            }
        }
        let type_id = descriptor.type_id.clone();
        if self.descriptors.get(&type_id) == Some(&descriptor) {
            return Ok(());
        }
        self.descriptors.insert(type_id.clone(), descriptor);
        self.advance_schema_generation(&type_id);
        Ok(())
    }

    pub(super) fn remove_vm_descriptor(&mut self, type_id: &str) {
        if self.descriptors.remove(type_id).is_some() {
            self.advance_schema_generation(type_id);
        }
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &ComponentTypeDescriptor> {
        self.descriptors.values()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn contains(&self, type_id: &str) -> bool {
        self.descriptors.contains_key(type_id)
    }

    fn advance_schema_generation(&mut self, type_id: &str) {
        self.next_schema_generation = self.next_schema_generation.saturating_add(1);
        self.schema_generations
            .insert(type_id.to_string(), self.next_schema_generation);
    }
}

fn component_type_belongs_to_plugin(type_id: &str, plugin_id: &str) -> bool {
    let Some(suffix) = type_id.strip_prefix(plugin_id) else {
        return false;
    };
    suffix.starts_with('.')
}

#[cfg(test)]
mod tests {
    use super::ComponentTypeRegistry;
    use crate::core::framework::scene::ComponentTypeDescriptor;

    #[test]
    fn schema_generation_changes_only_for_updated_component_type() {
        let mut registry = ComponentTypeRegistry::default();
        let cloud =
            ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
                .with_property("coverage", "Scalar", true);
        registry.upsert_vm_descriptor(cloud.clone()).unwrap();
        let cloud_generation = registry.schema_generation(&cloud.type_id);

        registry.upsert_vm_descriptor(cloud.clone()).unwrap();
        assert_eq!(registry.schema_generation(&cloud.type_id), cloud_generation);

        registry
            .upsert_vm_descriptor(
                ComponentTypeDescriptor::new("weather.Component.Wind", "weather", "Wind")
                    .with_property("speed", "Scalar", true),
            )
            .unwrap();
        assert_eq!(registry.schema_generation(&cloud.type_id), cloud_generation);

        registry
            .upsert_vm_descriptor(cloud.with_property("density", "Scalar", false))
            .unwrap();
        assert!(registry.schema_generation("weather.Component.CloudLayer") > cloud_generation);
        assert!(registry.schema_catalog_generation() > cloud_generation);
    }
}
