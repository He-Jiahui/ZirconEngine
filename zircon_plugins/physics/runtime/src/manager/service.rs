use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::{
    physics::{
        PhysicsBackendState, PhysicsBackendStatus, PhysicsContactEvent, PhysicsManager,
        PhysicsQueryInterface, PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsSceneStepResult,
        PhysicsSettings, PhysicsSettingsStoreError, PhysicsShapeCastHit, PhysicsShapeCastQuery,
        PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery, PhysicsTriggerEvent,
        PhysicsWorldStepPlan, PhysicsWorldSyncState,
    },
    scene::physics::PhysicsMaterialMetadata,
};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::world::World;

use crate::backend::builtin::{
    compute_contact_events, compute_trigger_events, integrate_builtin_physics_steps,
};
use crate::backend::{physics_backend_status, select_runtime_backend, PhysicsRuntimeBackend};

use super::clock::configured_step_seconds;
use super::command_buffer::apply_commands_to_scene;
use super::poison_recovery::recover_lock;
use super::query;
use super::world_sync::{build_world_sync_state, clear_world_state, sanitize_world_sync_state};
use super::DefaultPhysicsManager;

impl PhysicsManager for DefaultPhysicsManager {
    fn backend_name(&self) -> String {
        self.settings().backend
    }

    fn settings(&self) -> PhysicsSettings {
        recover_lock(&self.settings).clone()
    }

    fn store_settings(&self, settings: PhysicsSettings) -> Result<(), PhysicsSettingsStoreError> {
        DefaultPhysicsManager::store_settings(self, settings)
    }

    fn default_material(&self) -> PhysicsMaterialMetadata {
        self.default_material.clone()
    }

    fn backend_status(&self) -> PhysicsBackendStatus {
        let mut status = physics_backend_status(&self.settings());
        if let Some(error) = recover_lock(&self.last_backend_error).clone() {
            status.active_backend = None;
            status.state = PhysicsBackendState::Unavailable;
            status.detail = Some(error);
        }
        status
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
        match select_runtime_backend(&settings) {
            PhysicsRuntimeBackend::Builtin => self.sync_builtin_world(sync, &settings),
            PhysicsRuntimeBackend::Jolt => {
                #[cfg(feature = "backend-jolt")]
                self.sync_jolt_world(sync, &settings);
            }
            PhysicsRuntimeBackend::Disabled | PhysicsRuntimeBackend::Unavailable => {
                self.clear_world(sync.world);
            }
        }
    }

    fn synchronized_world(&self, world: WorldHandle) -> Option<PhysicsWorldSyncState> {
        recover_lock(&self.synced_worlds).get(&world).cloned()
    }

    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Vec<PhysicsRayCastHit> {
        query::ray_cast(self, query)
    }

    fn shape_overlap(&self, query: &PhysicsShapeOverlapQuery) -> Vec<PhysicsShapeOverlapHit> {
        query::shape_overlap(self, query)
    }

    fn shape_cast(&self, query: &PhysicsShapeCastQuery) -> Vec<PhysicsShapeCastHit> {
        query::shape_cast(self, query)
    }

    fn drain_contacts(&self, world: WorldHandle) -> Vec<PhysicsContactEvent> {
        recover_lock(&self.contacts)
            .remove(&world)
            .unwrap_or_default()
    }

    fn drain_triggers(&self, world: WorldHandle) -> Vec<PhysicsTriggerEvent> {
        recover_lock(&self.triggers)
            .remove(&world)
            .unwrap_or_default()
    }
}

impl DefaultPhysicsManager {
    pub(crate) fn tick_scene_world(
        &self,
        world_handle: WorldHandle,
        world: &mut World,
        delta_seconds: Real,
    ) -> PhysicsSceneStepResult {
        let settings = self.settings();
        let step_plan = fixed_update_step_plan(&settings, delta_seconds);
        match select_runtime_backend(&settings) {
            PhysicsRuntimeBackend::Builtin => {
                if step_plan.steps > 0 {
                    let commands = self.drain_body_commands(world_handle);
                    apply_commands_to_scene(world, &commands, step_plan.step_seconds);
                    integrate_builtin_physics_steps(world, step_plan);
                }
                self.sync_builtin_world(build_world_sync_state(world_handle, world), &settings);
                PhysicsSceneStepResult {
                    step_plan,
                    contacts: self.drain_contacts(world_handle),
                    triggers: self.drain_triggers(world_handle),
                }
            }
            PhysicsRuntimeBackend::Jolt => {
                #[cfg(feature = "backend-jolt")]
                {
                    let commands = if step_plan.steps > 0 {
                        self.drain_body_commands(world_handle)
                    } else {
                        Vec::new()
                    };
                    return self.tick_jolt_scene_world(
                        build_world_sync_state(world_handle, world),
                        &settings,
                        step_plan,
                        commands,
                    );
                }
                #[cfg(not(feature = "backend-jolt"))]
                {
                    self.clear_world(world_handle);
                    PhysicsSceneStepResult {
                        step_plan: PhysicsWorldStepPlan {
                            steps: 0,
                            ..step_plan
                        },
                        contacts: Vec::new(),
                        triggers: Vec::new(),
                    }
                }
            }
            PhysicsRuntimeBackend::Disabled | PhysicsRuntimeBackend::Unavailable => {
                self.clear_world(world_handle);
                PhysicsSceneStepResult {
                    step_plan,
                    contacts: Vec::new(),
                    triggers: Vec::new(),
                }
            }
        }
    }
}

impl PhysicsQueryInterface for DefaultPhysicsManager {
    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Vec<PhysicsRayCastHit> {
        query::ray_cast(self, query)
    }

    fn shape_overlap(&self, query: &PhysicsShapeOverlapQuery) -> Vec<PhysicsShapeOverlapHit> {
        query::shape_overlap(self, query)
    }

    fn shape_cast(&self, query: &PhysicsShapeCastQuery) -> Vec<PhysicsShapeCastHit> {
        query::shape_cast(self, query)
    }
}

fn fixed_update_step_plan(settings: &PhysicsSettings, delta_seconds: Real) -> PhysicsWorldStepPlan {
    let configured_step = configured_step_seconds(settings);
    if !select_runtime_backend(settings).allows_step(settings.simulation_mode)
        || configured_step <= 0.0
    {
        return PhysicsWorldStepPlan {
            steps: 0,
            step_seconds: configured_step,
            remaining_seconds: 0.0,
            interpolation_alpha: 0.0,
        };
    }

    let delta_seconds = if delta_seconds.is_finite() {
        delta_seconds.max(0.0)
    } else {
        0.0
    };
    if delta_seconds <= 0.0 {
        return PhysicsWorldStepPlan {
            steps: 0,
            step_seconds: configured_step,
            remaining_seconds: 0.0,
            interpolation_alpha: 0.0,
        };
    }

    PhysicsWorldStepPlan {
        steps: 1,
        step_seconds: delta_seconds,
        remaining_seconds: 0.0,
        interpolation_alpha: 0.0,
    }
}

impl DefaultPhysicsManager {
    fn sync_builtin_world(&self, sync: PhysicsWorldSyncState, settings: &PhysicsSettings) {
        let sync = sanitize_world_sync_state(sync);
        let contacts = compute_contact_events(&sync, settings);
        let previous_trigger_pairs = recover_lock(&self.trigger_pairs)
            .get(&sync.world)
            .cloned()
            .unwrap_or_default();
        let (trigger_pairs, triggers) =
            compute_trigger_events(&sync, settings, &previous_trigger_pairs);
        recover_lock(&self.synced_worlds).insert(sync.world, sync.clone());
        recover_lock(&self.contacts).insert(sync.world, contacts);
        recover_lock(&self.trigger_pairs).insert(sync.world, trigger_pairs);
        recover_lock(&self.triggers).insert(sync.world, triggers);
        *recover_lock(&self.last_backend_error) = None;
    }

    fn clear_world(&self, world: WorldHandle) {
        self.clear_body_commands(world);
        clear_world_state(
            world,
            &self.synced_worlds,
            &self.contacts,
            &self.trigger_pairs,
            &self.triggers,
        );
        #[cfg(feature = "backend-jolt")]
        self.remove_jolt_world(world);
    }
}
