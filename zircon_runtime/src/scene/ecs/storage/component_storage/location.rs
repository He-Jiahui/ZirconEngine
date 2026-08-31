use std::any::TypeId;
use std::fmt;

use crate::scene::ecs::{ArchetypeId, ComponentId, InternalEntity, StorageType};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ComponentStorageLocation {
    pub(crate) component_id: ComponentId,
    pub(crate) storage_type: StorageType,
    pub(crate) entity: InternalEntity,
    pub(crate) table_row: Option<usize>,
    pub(crate) table_archetype: Option<ArchetypeId>,
    pub(crate) table_column_slot: Option<usize>,
    /// Set by a compiled query binding so typed fetches never re-probe the registry.
    pub(crate) rust_type_id: Option<TypeId>,
}

impl fmt::Debug for ComponentStorageLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ComponentStorageLocation")
            .field("component_id", &self.component_id)
            .field("storage_type", &self.storage_type)
            .field("table_row", &self.table_row)
            .field("table_archetype", &self.table_archetype)
            .field("table_column_slot", &self.table_column_slot)
            .finish()
    }
}

impl ComponentStorageLocation {
    pub(crate) const fn table(
        component_id: ComponentId,
        entity: InternalEntity,
        archetype: ArchetypeId,
        row: usize,
        column_slot: usize,
    ) -> Self {
        Self {
            component_id,
            storage_type: StorageType::Table,
            entity,
            table_row: Some(row),
            table_archetype: Some(archetype),
            table_column_slot: Some(column_slot),
            rust_type_id: None,
        }
    }

    pub(crate) const fn sparse(component_id: ComponentId, entity: InternalEntity) -> Self {
        Self {
            component_id,
            storage_type: StorageType::SparseSet,
            entity,
            table_row: None,
            table_archetype: None,
            table_column_slot: None,
            rust_type_id: None,
        }
    }

    pub(crate) const fn with_rust_type_id(mut self, rust_type_id: TypeId) -> Self {
        self.rust_type_id = Some(rust_type_id);
        self
    }
}
