use std::any::TypeId;

use crate::scene::ecs::{ArchetypeId, ComponentId, InternalEntity, StorageType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentStorageLocation {
    pub component_id: ComponentId,
    pub storage_type: StorageType,
    pub entity: InternalEntity,
    pub table_row: Option<usize>,
    pub table_archetype: Option<ArchetypeId>,
    pub table_column_slot: Option<usize>,
    /// Set by a compiled query binding so typed fetches never re-probe the registry.
    pub rust_type_id: Option<TypeId>,
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
