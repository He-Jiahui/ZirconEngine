use crate::core::framework::physics::{
    PhysicsContactEvent, PhysicsTriggerEvent, PhysicsWorldStepPlan, SimulatedPoseFeed,
    SkeletalPoseTargets,
};
use crate::scene::world::World;
use std::sync::Arc;

use super::LevelSystem;

pub(super) fn clear_retained_pose_resources(world: &mut World) {
    if let Some(targets) = world.get_resource_mut::<SkeletalPoseTargets>() {
        targets.clear();
    }
    if let Some(feed) = world.get_resource_mut::<SimulatedPoseFeed>() {
        feed.clear();
    }
}

/// Immutable physics payload published by one LevelSystem physics revision.
///
/// Readers clone the outer `Arc` or one event-slice handle after a short lane lock; they never
/// clone the contact/trigger payload while holding the physics writer lane.
#[derive(Clone, Debug)]
pub(crate) struct PhysicsFrameStateSnapshot {
    generation: u64,
    step_plan: Option<PhysicsWorldStepPlan>,
    contacts: Arc<[PhysicsContactEvent]>,
    triggers: Arc<[PhysicsTriggerEvent]>,
}

impl Default for PhysicsFrameStateSnapshot {
    fn default() -> Self {
        Self {
            generation: 0,
            step_plan: None,
            contacts: Arc::from([]),
            triggers: Arc::from([]),
        }
    }
}

impl PhysicsFrameStateSnapshot {
    fn cleared(&self) -> Self {
        Self {
            generation: self.generation.saturating_add(1),
            step_plan: None,
            contacts: Arc::from([]),
            triggers: Arc::from([]),
        }
    }

    fn with_values(
        &self,
        step_plan: PhysicsWorldStepPlan,
        contacts: Vec<PhysicsContactEvent>,
        triggers: Vec<PhysicsTriggerEvent>,
    ) -> Self {
        Self {
            generation: self.generation.saturating_add(1),
            step_plan: Some(step_plan),
            contacts: contacts.into(),
            triggers: triggers.into(),
        }
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn step_plan(&self) -> Option<PhysicsWorldStepPlan> {
        self.step_plan
    }

    pub(crate) fn contacts(&self) -> &Arc<[PhysicsContactEvent]> {
        &self.contacts
    }

    pub(crate) fn triggers(&self) -> &Arc<[PhysicsTriggerEvent]> {
        &self.triggers
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct PhysicsRuntimeState {
    snapshot: Arc<PhysicsFrameStateSnapshot>,
}

impl PhysicsRuntimeState {
    pub(super) fn reset_after_world_replacement(&mut self) {
        self.snapshot = Arc::new(self.snapshot.cleared());
    }
}

impl LevelSystem {
    pub fn last_physics_step_plan(&self) -> Option<PhysicsWorldStepPlan> {
        self.physics_frame_snapshot().step_plan()
    }

    pub(crate) fn physics_frame_snapshot(&self) -> Arc<PhysicsFrameStateSnapshot> {
        Arc::clone(&self.lock_physics_state().snapshot)
    }

    pub fn physics_contacts(&self) -> Arc<[PhysicsContactEvent]> {
        Arc::clone(self.physics_frame_snapshot().contacts())
    }

    pub fn physics_triggers(&self) -> Arc<[PhysicsTriggerEvent]> {
        Arc::clone(self.physics_frame_snapshot().triggers())
    }

    /// Publishes physics output only while the producer still targets the installed World.
    ///
    /// The World lane is acquired before the physics publication lane, matching replacement's
    /// lock order. This makes a replacement between simulation and publication retire the
    /// result instead of exposing it through the next World's frame state.
    pub fn record_physics_step_if_replacement_epoch(
        &self,
        replacement_epoch: u64,
        step_plan: PhysicsWorldStepPlan,
        contacts: Vec<PhysicsContactEvent>,
        triggers: Vec<PhysicsTriggerEvent>,
    ) -> bool {
        let _world = self.lock_world();
        if self
            .world_replacement_epoch
            .load(std::sync::atomic::Ordering::Acquire)
            != replacement_epoch
        {
            return false;
        }
        let mut state = self.lock_physics_state();
        let published = &state.snapshot;
        if published.step_plan() == Some(step_plan)
            && published.contacts().as_ref() == contacts.as_slice()
            && published.triggers().as_ref() == triggers.as_slice()
        {
            return true;
        }

        state.snapshot = Arc::new(published.with_values(step_plan, contacts, triggers));
        true
    }
}

#[cfg(test)]
#[path = "tests/physics_runtime_enabled.rs"]
mod tests;
