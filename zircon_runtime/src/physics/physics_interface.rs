use crate::core::framework::physics::{
    PhysicsContactEvent, PhysicsManager, PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsSettings,
    PhysicsSimulationMode, PhysicsWorldSyncState,
};
use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;
use crate::core::CoreError;
use crate::scene::world::World;

use super::PhysicsTickPlan;

pub trait PhysicsInterface: PhysicsManager {
    fn store_settings(&self, settings: PhysicsSettings) -> Result<(), CoreError>;

    fn advance_clock(&self, world: WorldHandle, delta_seconds: Real) -> PhysicsTickPlan;

    fn sync_scene_world(&self, world_handle: WorldHandle, world: &World) {
        PhysicsManager::sync_world(self, super::build_world_sync_state(world_handle, world));
    }

    fn sync_world(&self, sync: PhysicsWorldSyncState) {
        PhysicsManager::sync_world(self, sync);
    }

    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Option<PhysicsRayCastHit> {
        PhysicsManager::ray_cast(self, query)
    }

    fn drain_contacts(&self, world: WorldHandle) -> Vec<PhysicsContactEvent> {
        PhysicsManager::drain_contacts(self, world)
    }

    fn is_enabled(&self) -> bool {
        self.settings().simulation_mode != PhysicsSimulationMode::Disabled
    }
}
