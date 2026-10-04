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
