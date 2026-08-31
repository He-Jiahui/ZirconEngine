#[cfg(test)]
mod tests {
    use crate::scene::{SceneError, SceneResult};

    use super::{EntityIdAllocator, FIRST_ENTITY_ID, TERMINAL_ENTITY_ID};

    #[test]
    fn allocator_reserves_only_valid_ids_and_rejects_terminal_state_without_mutation() {
        let mut allocator = EntityIdAllocator::default();

        assert_eq!(allocator.reserve_next(), Ok(FIRST_ENTITY_ID));
        assert_eq!(allocator.next_id(), FIRST_ENTITY_ID + 1);
        assert_eq!(
            EntityIdAllocator::from_persisted_next(0),
            Err(SceneError::EntityIdExhausted { entity: 0 })
        );
        assert_eq!(
            EntityIdAllocator::from_persisted_next(TERMINAL_ENTITY_ID),
            Err(SceneError::EntityIdExhausted {
                entity: TERMINAL_ENTITY_ID,
            })
        );

        let mut exhausted = EntityIdAllocator {
            next_id: TERMINAL_ENTITY_ID,
        };
        let result: SceneResult<_> = exhausted.reserve_next();

        assert_eq!(
            result,
            Err(SceneError::EntityIdExhausted {
                entity: TERMINAL_ENTITY_ID,
            })
        );
        assert_eq!(exhausted.next_id(), TERMINAL_ENTITY_ID);
    }
}
use crate::scene::{EntityId, SceneError, SceneResult};

pub(super) const FIRST_ENTITY_ID: EntityId = 1;
pub(super) const TERMINAL_ENTITY_ID: EntityId = u64::MAX;

/// Owns allocation of persistent scene entity IDs for one World instance.
///
/// `0` remains the absent-entity sentinel and `u64::MAX` is terminal, so a
/// successful allocation always has a representable, non-sentinel successor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EntityIdAllocator {
    next_id: EntityId,
}

impl EntityIdAllocator {
    pub(super) fn from_persisted_next(next_id: EntityId) -> SceneResult<Self> {
        Self::validate_next_id(next_id)?;
        Ok(Self { next_id })
    }

    pub(super) const fn next_id(self) -> EntityId {
        self.next_id
    }

    pub(super) fn reserve_next(&mut self) -> SceneResult<EntityId> {
        let entity = self.next_available()?;
        self.next_id = Self::successor(entity)?;
        Ok(entity)
    }

    pub(super) fn next_available(self) -> SceneResult<EntityId> {
        Self::validate_next_id(self.next_id)?;
        Ok(self.next_id)
    }

    pub(super) fn next_after(self, entity: EntityId) -> SceneResult<EntityId> {
        Ok(self.next_id.max(Self::successor(entity)?))
    }

    pub(super) fn advance_past(&mut self, entity: EntityId) -> SceneResult<()> {
        self.next_id = self.next_after(entity)?;
        Ok(())
    }

    pub(super) fn replace_next(&mut self, next_id: EntityId) -> SceneResult<()> {
        Self::validate_next_id(next_id)?;
        self.next_id = next_id;
        Ok(())
    }

    fn successor(entity: EntityId) -> SceneResult<EntityId> {
        if entity == 0 || entity >= TERMINAL_ENTITY_ID {
            return Err(SceneError::EntityIdExhausted { entity });
        }
        entity
            .checked_add(1)
            .ok_or(SceneError::EntityIdExhausted { entity })
    }

    fn validate_next_id(next_id: EntityId) -> SceneResult<()> {
        if next_id == 0 || next_id >= TERMINAL_ENTITY_ID {
            return Err(SceneError::EntityIdExhausted { entity: next_id });
        }
        Ok(())
    }
}

impl Default for EntityIdAllocator {
    fn default() -> Self {
        Self {
            next_id: FIRST_ENTITY_ID,
        }
    }
}
