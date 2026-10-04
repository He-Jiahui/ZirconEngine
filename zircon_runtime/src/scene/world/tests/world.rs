use super::{World, WorldPersistentState, WorldPersistentStateError};
use crate::scene::SceneError;

#[test]
fn persistent_state_retains_invalid_entity_allocator_diagnostics() {
    let world = World::empty();
    let mut state: WorldPersistentState =
        serde_json::from_value(serde_json::to_value(world).expect("world serializes"))
            .expect("serialized world decodes as persistent state");
    state.next_id = u64::MAX;

    assert!(matches!(
        World::from_persistent_state(state),
        Err(WorldPersistentStateError::Scene(
            SceneError::EntityIdExhausted { entity: u64::MAX }
        ))
    ));
}
