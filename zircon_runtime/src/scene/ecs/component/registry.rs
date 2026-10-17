use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use zircon_runtime_interface::retained_memory::{
    RetainedBoundBuilderV1, RetainedOwnerV1, RetentionContractError, RETAINED_PROVIDER_VERSION_V1,
};

use crate::scene::ecs::StorageType;

use super::id::ComponentId;
use super::marker::Component;
use super::table_column::TableColumnLayout;

#[path = "registry/transferred.rs"]
mod transferred;

pub(crate) use transferred::{
    PreflightedTransferredDescriptorImports, TransferredComponentDescriptor,
};

/// The concrete provider survives Any erasure and descriptor transfer.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ComponentRetentionProviderV1 {
    type_id: TypeId,
    version: u32,
    measure: fn(&dyn Any, &mut RetainedBoundBuilderV1) -> Result<(), RetentionContractError>,
    measure_world: fn(
        &crate::scene::World,
        crate::scene::EntityId,
        ComponentId,
        &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError>,
}
impl PartialEq for ComponentRetentionProviderV1 {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id && self.version == other.version
    }
}
impl Eq for ComponentRetentionProviderV1 {}
impl ComponentRetentionProviderV1 {
    fn of<T: Component>() -> Self {
        fn measure<T: Component>(
            value: &dyn Any,
            meter: &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError> {
            let value = value
                .downcast_ref::<T>()
                .ok_or(RetentionContractError::Provider {
                    reason: "component retention type mismatch",
                })?;
            meter.owned(value)
        }
        fn measure_world<T: Component>(
            world: &crate::scene::World,
            entity: crate::scene::EntityId,
            _component_id: ComponentId,
            m: &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError> {
            m.owned(
                world
                    .get::<T>(entity)
                    .ok_or(RetentionContractError::Stale)?,
            )
        }
        Self {
            type_id: TypeId::of::<T>(),
            version: RETAINED_PROVIDER_VERSION_V1,
            measure: measure::<T>,
            measure_world: measure_world::<T>,
        }
    }
    pub(crate) fn measure_world(
        &self,
        world: &crate::scene::World,
        entity: crate::scene::EntityId,
        component_id: ComponentId,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        (self.measure_world)(world, entity, component_id, m)
    }
    pub(crate) fn measurement_v1(
        &self,
    ) -> fn(&dyn Any, &mut RetainedBoundBuilderV1) -> Result<(), RetentionContractError> {
        self.measure
    }
    pub(crate) fn measure(
        &self,
        value: &dyn Any,
        meter: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        if self.version != RETAINED_PROVIDER_VERSION_V1 || value.type_id() != self.type_id {
            return Err(RetentionContractError::Provider {
                reason: "component retention provider identity changed",
            });
        }
        (self.measure)(value, meter)
    }
}

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
/// 为当前 World 分配组件 ID；Rust 类型和动态插件类型分别索引，只有静态表组件登记连续列布局。
pub struct ComponentRegistry {
    descriptors: Vec<ComponentDescriptor>,
    rust_ids_by_type_id: HashMap<TypeId, ComponentId>,
    dynamic_ids_by_type_id: HashMap<String, ComponentId>,
    table_column_layouts: HashMap<ComponentId, TableColumnLayout>,
    retention_providers: HashMap<ComponentId, ComponentRetentionProviderV1>,
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
        self.retention_providers
            .insert(id, ComponentRetentionProviderV1::of::<T>());
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

    /// The World owns dynamic-marker construction and supplies an id-aware
    /// borrow. Erased detached rows still use the same concrete T provider.
    pub(in crate::scene) fn bind_dynamic_retention_provider_v1<T: Component>(
        &mut self, id: ComponentId,
        measure_world: fn(
            &crate::scene::World, crate::scene::EntityId, ComponentId,
            &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError>,
    ) -> Result<(), RetentionContractError> {
        let descriptor = self.descriptor(id).ok_or(RetentionContractError::Stale)?;
        if !matches!(&descriptor.source, ComponentDescriptorSource::DynamicPlugin { .. })
            || descriptor.storage_type != T::STORAGE_TYPE
        {
            return Err(RetentionContractError::Provider { reason: "dynamic retained owner descriptor mismatch" });
        }
        let mut provider = ComponentRetentionProviderV1::of::<T>();
        provider.measure_world = measure_world;
        if self.retention_providers.get(&id).is_some_and(|existing| existing != &provider) {
            return Err(RetentionContractError::Provider { reason: "dynamic retained owner type changed" });
        }
        self.retention_providers.insert(id, provider);
        Ok(())
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

    /// Overlapping IDs must name the same type and storage. A staging snapshot
    /// can extend the original prefix or omit unused trailing registrations.
    pub(in crate::scene) fn preserves_ids_of(&self, original: &Self) -> bool {
        self.descriptors
            .iter()
            .zip(&original.descriptors)
            .all(|(staged, live)| {
                staged == live
                    && self.retention_providers.get(&staged.id)
                        == original.retention_providers.get(&live.id)
                    && self.table_column_layouts.get(&staged.id)
                        == original.table_column_layouts.get(&live.id)
            })
    }

    pub(in crate::scene) fn retain_missing_ids_from(&mut self, original: &Self) {
        for descriptor in original.descriptors.iter().skip(self.descriptors.len()) {
            self.descriptors.push(descriptor.clone());
        }
        for (type_id, id) in &original.rust_ids_by_type_id {
            self.rust_ids_by_type_id.insert(*type_id, *id);
        }
        for (type_id, id) in &original.dynamic_ids_by_type_id {
            self.dynamic_ids_by_type_id.insert(type_id.clone(), *id);
        }
        for (id, provider) in &original.retention_providers {
            self.retention_providers.insert(*id, *provider);
        }
        for (id, layout) in &original.table_column_layouts {
            self.table_column_layouts.insert(*id, layout.clone());
        }
    }

    pub(crate) fn retention_provider_v1(
        &self,
        id: ComponentId,
    ) -> Option<ComponentRetentionProviderV1> {
        self.retention_providers.get(&id).copied()
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
#[path = "registry/tests/cases.rs"]
mod tests;
