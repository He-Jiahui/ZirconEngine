use crate::scene::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchetypeMove {
    pub entity_row: usize,
    pub swapped_entity: Option<(EntityId, usize)>,
}
