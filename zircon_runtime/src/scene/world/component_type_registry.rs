use std::collections::BTreeMap;

use crate::plugin::ComponentTypeDescriptor;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComponentTypeRegistry {
    descriptors: BTreeMap<String, ComponentTypeDescriptor>,
}

impl ComponentTypeRegistry {
    pub fn register(&mut self, descriptor: ComponentTypeDescriptor) -> Result<(), String> {
        if !component_type_belongs_to_plugin(&descriptor.type_id, &descriptor.plugin_id) {
            return Err(format!(
                "component type {} must be prefixed by plugin id {}",
                descriptor.type_id, descriptor.plugin_id
            ));
        }
        if self.descriptors.contains_key(&descriptor.type_id) {
            return Err(format!(
                "component type {} already registered",
                descriptor.type_id
            ));
        }
        self.descriptors
            .insert(descriptor.type_id.clone(), descriptor);
        Ok(())
    }

    pub fn descriptor(&self, type_id: &str) -> Option<&ComponentTypeDescriptor> {
        self.descriptors.get(type_id)
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
}

fn component_type_belongs_to_plugin(type_id: &str, plugin_id: &str) -> bool {
    let Some(suffix) = type_id.strip_prefix(plugin_id) else {
        return false;
    };
    suffix.starts_with('.')
}
