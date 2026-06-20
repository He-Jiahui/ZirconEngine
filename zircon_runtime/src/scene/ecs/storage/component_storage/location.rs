use crate::scene::ecs::{ComponentId, InternalEntity, StorageType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentStorageLocation {
    pub component_id: ComponentId,
    pub storage_type: StorageType,
    pub entity: InternalEntity,
    pub table_row: Option<usize>,
}
