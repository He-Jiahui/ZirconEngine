use crate::core::framework::physics::{
    PhysicsContactEvent, PhysicsTriggerEvent, PhysicsWorldStepPlan,
};

use super::LevelSystem;

#[derive(Clone, Debug, Default)]
pub(super) struct PhysicsRuntimeState {
    step_plan: Option<PhysicsWorldStepPlan>,
    contacts: Vec<PhysicsContactEvent>,
    triggers: Vec<PhysicsTriggerEvent>,
}

impl LevelSystem {
    pub fn last_physics_step_plan(&self) -> Option<PhysicsWorldStepPlan> {
        self.lock_runtime_state().physics.step_plan
    }

    pub fn physics_contacts(&self) -> Vec<PhysicsContactEvent> {
        self.lock_runtime_state().physics.contacts.clone()
    }

    pub fn physics_triggers(&self) -> Vec<PhysicsTriggerEvent> {
        self.lock_runtime_state().physics.triggers.clone()
    }

    pub fn record_physics_step(
        &self,
        step_plan: PhysicsWorldStepPlan,
        contacts: Vec<PhysicsContactEvent>,
        triggers: Vec<PhysicsTriggerEvent>,
    ) {
        let mut runtime_state = self.lock_runtime_state();
        runtime_state.physics.step_plan = Some(step_plan);
        runtime_state.physics.contacts = contacts;
        runtime_state.physics.triggers = triggers;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::core::framework::physics::{
        PhysicsContactEvent, PhysicsTriggerEvent, PhysicsWorldStepPlan,
    };
    use crate::core::framework::scene::WorldHandle;
    use crate::scene::world::World;

    use super::LevelSystem;
    use crate::scene::level_system::LevelMetadata;

    #[test]
    fn physics_runtime_state_records_and_resets_with_the_level() {
        let level = LevelSystem::new(
            WorldHandle::new(42),
            Arc::new(Mutex::new(World::empty())),
            LevelMetadata::default(),
        );
        let plan = PhysicsWorldStepPlan {
            steps: 1,
            step_seconds: 1.0 / 60.0,
            remaining_seconds: 0.0,
            interpolation_alpha: 0.0,
        };

        level.record_physics_step(
            plan,
            Vec::<PhysicsContactEvent>::new(),
            Vec::<PhysicsTriggerEvent>::new(),
        );

        assert_eq!(level.last_physics_step_plan(), Some(plan));
        assert!(level.physics_contacts().is_empty());
        assert!(level.physics_triggers().is_empty());

        level.replace_world_and_reset_runtime_state(World::empty());
        assert_eq!(level.last_physics_step_plan(), None);
    }
}
