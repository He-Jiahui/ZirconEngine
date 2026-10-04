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

pub(super) fn replace_world_snapshots(
    manager: &DefaultAiManager,
    world: WorldHandle,
    snapshots: Vec<AiPerceptionSnapshot>,
) -> Result<(), AiManagerError> {
    // 先验证整批快照，再替换该 world 的旧数据，避免无效输入清空已发布状态。
    for snapshot in &snapshots {
        validate_perception_snapshot(snapshot.agent, snapshot)?;
    }

    let mut state = manager.lock_state();
    state
        .perceptions
        .retain(|(snapshot_world, _), _| *snapshot_world != world);
    for snapshot in snapshots {
        state.perceptions.insert((world, snapshot.agent), snapshot);
    }
    Ok(())
}
