use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::core::CoreError;
use crate::scene::{world::DetachPreparationOwner, SceneError, SceneResult, World};
use zircon_runtime_interface::world_sync::LifecycleTargetChange;

use super::SceneScheduleStagePlan;

/// This guard releases the original Schedule even if its World is replaced or
/// the Schedule is retained elsewhere. Drop never locks or mutates a new World.
pub(crate) struct ScheduleExecution {
    plan: Arc<SceneScheduleStagePlan>,
    active_execution_count: Arc<AtomicUsize>,
    world_owner: DetachPreparationOwner,
}

impl ScheduleExecution {
    pub(super) fn new(
        plan: Arc<SceneScheduleStagePlan>,
        active_execution_count: Arc<AtomicUsize>,
        world_owner: DetachPreparationOwner,
    ) -> Self {
        Self {
            plan,
            active_execution_count,
            world_owner,
        }
    }

    pub(crate) fn target_change(&self, world: &World) -> Option<LifecycleTargetChange> {
        if !self.world_owner.is_same_world(world.owner_identity()) {
            Some(LifecycleTargetChange::World)
        } else if !world.schedule().owns_stage_plan(&self.plan) {
            Some(LifecycleTargetChange::Schedule)
        } else {
            None
        }
    }

    pub(crate) fn ensure_bound_to(
        &self,
        world: &World,
        operation: &'static str,
    ) -> SceneResult<()> {
        match self.target_change(world) {
            Some(target) => Err(SceneError::LifecycleTargetReplaced { operation, target }),
            None => Ok(()),
        }
    }

    pub(crate) fn is_bound_to(&self, world: &World) -> bool {
        self.target_change(world).is_none()
    }

    pub(crate) fn plan(&self) -> &Arc<SceneScheduleStagePlan> {
        &self.plan
    }
}

impl Drop for ScheduleExecution {
    fn drop(&mut self) {
        self.active_execution_count.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(crate) fn schedule_owner_changed_error() -> CoreError {
    CoreError::Initialization(
        "SceneScheduleRunner schedule owner".to_string(),
        "World schedule was replaced during the active tick".to_string(),
    )
}
