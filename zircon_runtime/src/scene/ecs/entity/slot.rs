use crate::scene::EntityId;

use super::location::EntityLocation;

pub(super) const FIRST_GENERATION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EntitySlot {
    pub(super) generation: u32,
    pub(super) stable_id: Option<EntityId>,
    pub(super) location: Option<EntityLocation>,
}

impl Default for EntitySlot {
    fn default() -> Self {
        Self {
            generation: FIRST_GENERATION,
            stable_id: None,
            location: None,
        }
    }
}

pub(super) fn next_generation(generation: u32) -> Option<u32> {
    generation.checked_add(1)
}
