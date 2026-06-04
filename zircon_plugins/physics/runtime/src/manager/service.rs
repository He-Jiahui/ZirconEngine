use zircon_runtime::core::framework::physics::{
    PhysicsBackendStatus, PhysicsContactEvent, PhysicsManager, PhysicsMaterialMetadata,
    PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsSceneStepResult, PhysicsSettings,
    PhysicsShapeCastHit, PhysicsShapeCastQuery, PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery,
    PhysicsTriggerEvent, PhysicsWorldStepPlan, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::world::World;

use crate::backend::{physics_backend_status, select_runtime_backend};
use crate::query_contact::compute_contact_events;
use crate::trigger::compute_trigger_events;

use super::builtin_step::integrate_builtin_physics_steps;
use super::clock::configured_step_seconds;
use super::query;
use super::world_sync::{build_world_sync_state, clear_world_state, sanitize_world_sync_state};
use super::DefaultPhysicsManager;

impl PhysicsManager for DefaultPhysicsManager {
    fn backend_name(&self) -> String {
        self.settings().backend
    }

    fn settings(&self) -> PhysicsSettings {
        self.settings
            .lock()
            .expect("physics settings mutex poisoned")
            .clone()
    }

    fn default_material(&self) -> PhysicsMaterialMetadata {
        self.default_material.clone()
    }

    fn backend_status(&self) -> PhysicsBackendStatus {
        physics_backend_status(&self.settings())
    }

    fn plan_world_step(&self, world: WorldHandle, delta_seconds: Real) -> PhysicsWorldStepPlan {
        let settings = self.settings();
        if !select_runtime_backend(&settings).allows_step(settings.simulation_mode) {
            return PhysicsWorldStepPlan {
                steps: 0,
                step_seconds: configured_step_seconds(&settings),
                remaining_seconds: 0.0,
                interpolation_alpha: 0.0,
            };
        }

        self.advance_clock(world, delta_seconds)
    }

    fn sync_world(&self, sync: PhysicsWorldSyncState) {
        let settings = self.settings();
        if !select_runtime_backend(&settings).allows_sync() {
            clear_world_state(
                sync.world,
                &self.synced_worlds,
                &self.contacts,
                &self.trigger_pairs,
                &self.triggers,
            );
            return;
        }

        let sync = sanitize_world_sync_state(sync);
        let contacts = compute_contact_events(&sync, &settings);
        let previous_trigger_pairs = self
            .trigger_pairs
            .lock()
            .expect("physics trigger pair mutex poisoned")
            .get(&sync.world)
            .cloned()
            .unwrap_or_default();
        let (trigger_pairs, triggers) =
            compute_trigger_events(&sync, &settings, &previous_trigger_pairs);
        self.synced_worlds
            .lock()
            .expect("physics sync mutex poisoned")
            .insert(sync.world, sync.clone());
        self.contacts
            .lock()
            .expect("physics contact mutex poisoned")
            .insert(sync.world, contacts);
        self.trigger_pairs
            .lock()
            .expect("physics trigger pair mutex poisoned")
            .insert(sync.world, trigger_pairs);
        self.triggers
            .lock()
            .expect("physics trigger mutex poisoned")
            .insert(sync.world, triggers);
    }

    fn synchronized_world(&self, world: WorldHandle) -> Option<PhysicsWorldSyncState> {
        self.synced_worlds
            .lock()
            .expect("physics sync mutex poisoned")
            .get(&world)
            .cloned()
    }

    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Option<PhysicsRayCastHit> {
        query::ray_cast(self, query)
    }

    fn shape_overlap(&self, query: &PhysicsShapeOverlapQuery) -> Vec<PhysicsShapeOverlapHit> {
        query::shape_overlap(self, query)
    }

    fn shape_cast(&self, query: &PhysicsShapeCastQuery) -> Option<PhysicsShapeCastHit> {
        query::shape_cast(self, query)
    }

    fn drain_contacts(&self, world: WorldHandle) -> Vec<PhysicsContactEvent> {
        self.contacts
            .lock()
            .expect("physics contact mutex poisoned")
            .remove(&world)
            .unwrap_or_default()
    }

    fn drain_triggers(&self, world: WorldHandle) -> Vec<PhysicsTriggerEvent> {
        self.triggers
            .lock()
            .expect("physics trigger mutex poisoned")
            .remove(&world)
            .unwrap_or_default()
    }

    fn tick_scene_world(
        &self,
        world_handle: WorldHandle,
        world: &mut World,
        delta_seconds: Real,
    ) -> PhysicsSceneStepResult {
        let step_plan = self.plan_world_step(world_handle, delta_seconds);
        let settings = self.settings();
        if select_runtime_backend(&settings).allows_step(settings.simulation_mode) {
            integrate_builtin_physics_steps(world, step_plan);
        }
        self.sync_world(build_world_sync_state(world_handle, world));
        PhysicsSceneStepResult {
            step_plan,
            contacts: self.drain_contacts(world_handle),
            triggers: self.drain_triggers(world_handle),
        }
    }
}
