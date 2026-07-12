use zircon_runtime::core::framework::ai::{AiManagerError, AiPerceptionSnapshot};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

use super::validation::validate_perception_snapshot;
use super::DefaultAiManager;

pub(super) fn set_snapshot(
    manager: &DefaultAiManager,
    world: WorldHandle,
    entity: EntityId,
    snapshot: AiPerceptionSnapshot,
) -> Result<(), AiManagerError> {
    validate_perception_snapshot(entity, &snapshot)?;

    manager
        .lock_state()
        .perceptions
        .insert((world, entity), snapshot);
    Ok(())
}

pub(super) fn snapshot(
    manager: &DefaultAiManager,
    world: WorldHandle,
    entity: EntityId,
) -> Option<AiPerceptionSnapshot> {
    manager
        .lock_state()
        .perceptions
        .get(&(world, entity))
        .cloned()
}
