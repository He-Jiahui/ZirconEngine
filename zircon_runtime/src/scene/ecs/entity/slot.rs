use crate::scene::EntityId;

use super::location::EntityLocation;

pub(super) const FIRST_GENERATION: u32 = 1;

/// 内部句柄的槽位状态；释放时递增代数，让旧句柄无法命中新实体。代数耗尽的槽位不再复用。
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
