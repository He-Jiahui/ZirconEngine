use crate::scene::EntityId;

use super::id::ArchetypeId;
use super::signature::ArchetypeSignature;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchetypeRecord {
    id: ArchetypeId,
    signature: ArchetypeSignature,
    entities: Vec<EntityId>,
}

impl ArchetypeRecord {
    pub(super) fn new(id: ArchetypeId, signature: ArchetypeSignature) -> Self {
        Self {
            id,
            signature,
            entities: Vec::new(),
        }
    }

    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    pub fn signature(&self) -> &ArchetypeSignature {
        &self.signature
    }

    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub(super) fn push_entity(&mut self, entity: EntityId) -> usize {
        let row = self.entities.len();
        self.entities.push(entity);
        row
    }

    pub(super) fn swap_remove_entity(
        &mut self,
        row: usize,
        entity: EntityId,
    ) -> Option<(EntityId, usize)> {
        let last_row = self.entities.len() - 1;
        let removed = self.entities.swap_remove(row);
        debug_assert_eq!(removed, entity);
        if row != last_row {
            Some((self.entities[row], row))
        } else {
            None
        }
    }
}
