use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::scene::ecs::{Component, ComponentId, StorageType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentDescriptor {
    pub id: ComponentId,
    pub type_name: String,
    pub storage_type: StorageType,
    pub source: ComponentDescriptorSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentDescriptorSource {
    RustType,
    DynamicPlugin { component_type_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComponentKey {
    Rust(TypeId),
    Dynamic(String),
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct ComponentRegistry {
    descriptors: Vec<ComponentDescriptor>,
    ids_by_key: HashMap<ComponentKey, ComponentId>,
}

impl ComponentRegistry {
    pub fn component_id<T>(&mut self) -> ComponentId
    where
        T: Component,
    {
        let key = ComponentKey::Rust(TypeId::of::<T>());
        if let Some(id) = self.ids_by_key.get(&key).copied() {
            return id;
        }
        self.insert_descriptor(
            key,
            type_name::<T>().to_string(),
            T::STORAGE_TYPE,
            ComponentDescriptorSource::RustType,
        )
    }

    pub fn dynamic_component_id(&mut self, component_type_id: &str) -> ComponentId {
        let key = ComponentKey::Dynamic(component_type_id.to_string());
        if let Some(id) = self.ids_by_key.get(&key).copied() {
            return id;
        }
        self.insert_descriptor(
            key,
            component_type_id.to_string(),
            StorageType::SparseSet,
            ComponentDescriptorSource::DynamicPlugin {
                component_type_id: component_type_id.to_string(),
            },
        )
    }

    pub fn registered_component_id<T>(&self) -> Option<ComponentId>
    where
        T: Component,
    {
        self.ids_by_key
            .get(&ComponentKey::Rust(TypeId::of::<T>()))
            .copied()
    }

    pub fn registered_dynamic_component_id(&self, component_type_id: &str) -> Option<ComponentId> {
        self.ids_by_key
            .get(&ComponentKey::Dynamic(component_type_id.to_string()))
            .copied()
    }

    pub fn descriptor(&self, id: ComponentId) -> Option<&ComponentDescriptor> {
        self.descriptors.get(id.index())
    }

    pub fn descriptors(&self) -> &[ComponentDescriptor] {
        &self.descriptors
    }

    fn insert_descriptor(
        &mut self,
        key: ComponentKey,
        type_name: String,
        storage_type: StorageType,
        source: ComponentDescriptorSource,
    ) -> ComponentId {
        let id = ComponentId::new(self.descriptors.len());
        self.descriptors.push(ComponentDescriptor {
            id,
            type_name,
            storage_type,
            source,
        });
        self.ids_by_key.insert(key, id);
        id
    }
}

impl fmt::Debug for ComponentRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentRegistry")
            .field("descriptors", &self.descriptors)
            .finish()
    }
}
