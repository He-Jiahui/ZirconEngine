use std::any::{TypeId, type_name};
use std::collections::HashMap;
use std::fmt;

use crate::scene::ecs::StorageType;

use super::id::ComponentId;
use super::marker::Component;
use super::table_column::TableColumnLayout;

mod transferred;

pub(crate) use transferred::{
    PreflightedTransferredDescriptorImports, TransferredComponentDescriptor,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentDescriptor {
    pub id: ComponentId,
    pub type_name: String,
    pub storage_type: StorageType,
    pub source: ComponentDescriptorSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentDescriptorSource {
    RustType { type_id: TypeId },
    DynamicPlugin { component_type_id: String },
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct ComponentRegistry {
    descriptors: Vec<ComponentDescriptor>,
    rust_ids_by_type_id: HashMap<TypeId, ComponentId>,
    dynamic_ids_by_type_id: HashMap<String, ComponentId>,
    table_column_layouts: HashMap<ComponentId, TableColumnLayout>,
}

impl ComponentRegistry {
    pub(crate) fn generation(&self) -> u64 {
        self.descriptors.len() as u64
    }

    pub fn component_id<T>(&mut self) -> ComponentId
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        if let Some(id) = self.rust_ids_by_type_id.get(&type_id).copied() {
            return id;
        }
        let storage_type = T::STORAGE_TYPE;
        let id = self.insert_descriptor(
            type_name::<T>().to_string(),
            storage_type,
            ComponentDescriptorSource::RustType { type_id },
        );
        self.rust_ids_by_type_id.insert(type_id, id);
        if storage_type == StorageType::Table {
            self.table_column_layouts
                .insert(id, TableColumnLayout::of::<T>());
        }
        id
    }

    pub fn dynamic_component_id(&mut self, component_type_id: &str) -> ComponentId {
        if let Some(id) = self.dynamic_ids_by_type_id.get(component_type_id).copied() {
            return id;
        }
        let id = self.insert_descriptor(
            component_type_id.to_string(),
            StorageType::SparseSet,
            ComponentDescriptorSource::DynamicPlugin {
                component_type_id: component_type_id.to_string(),
            },
        );
        self.dynamic_ids_by_type_id
            .insert(component_type_id.to_string(), id);
        id
    }

    pub fn registered_component_id<T>(&self) -> Option<ComponentId>
    where
        T: Component,
    {
        self.rust_ids_by_type_id.get(&TypeId::of::<T>()).copied()
    }

    pub fn registered_dynamic_component_id(&self, component_type_id: &str) -> Option<ComponentId> {
        self.dynamic_ids_by_type_id.get(component_type_id).copied()
    }

    pub fn descriptor(&self, id: ComponentId) -> Option<&ComponentDescriptor> {
        self.descriptors.get(id.index())
    }

    pub(crate) fn rust_type_for_id(&self, id: ComponentId) -> Option<(TypeId, &str)> {
        let descriptor = self.descriptor(id)?;
        match &descriptor.source {
            ComponentDescriptorSource::RustType { type_id } => {
                Some((*type_id, descriptor.type_name.as_str()))
            }
            ComponentDescriptorSource::DynamicPlugin { .. } => None,
        }
    }

    pub fn descriptors(&self) -> &[ComponentDescriptor] {
        &self.descriptors
    }

    pub(crate) fn table_column_layout(&self, id: ComponentId) -> Option<&TableColumnLayout> {
        self.table_column_layouts.get(&id)
    }

    pub(crate) fn table_column_layouts_for_ids(
        &self,
        component_ids: &[ComponentId],
    ) -> Option<Vec<(ComponentId, TableColumnLayout)>> {
        let mut layouts = Vec::with_capacity(component_ids.len());
        for component_id in component_ids {
            let layout = self.table_column_layout(*component_id)?.clone();
            layouts.push((*component_id, layout));
        }
        Some(layouts)
    }

    fn insert_descriptor(
        &mut self,
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

#[cfg(test)]
#[path = "registry/tests.rs"]
mod tests;
