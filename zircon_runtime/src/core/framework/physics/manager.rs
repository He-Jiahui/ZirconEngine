use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;
use crate::scene::World;

use super::{
    PhysicsBackendStatus, PhysicsContactEvent, PhysicsMaterialMetadata, PhysicsRayCastHit,
    PhysicsRayCastQuery, PhysicsSceneStepResult, PhysicsSettings, PhysicsWorldStepPlan,
    PhysicsWorldSyncState,
};

pub trait PhysicsManager: Send + Sync {
    fn backend_name(&self) -> String;
    fn settings(&self) -> PhysicsSettings;
    fn default_material(&self) -> PhysicsMaterialMetadata;
    fn backend_status(&self) -> PhysicsBackendStatus;
    fn plan_world_step(&self, world: WorldHandle, delta_seconds: Real) -> PhysicsWorldStepPlan;
    fn sync_world(&self, sync: PhysicsWorldSyncState);
    fn synchronized_world(&self, world: WorldHandle) -> Option<PhysicsWorldSyncState>;
    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Option<PhysicsRayCastHit>;
    fn drain_contacts(&self, world: WorldHandle) -> Vec<PhysicsContactEvent>;
    fn tick_scene_world(
        &self,
        world_handle: WorldHandle,
        _world: &mut World,
        delta_seconds: Real,
    ) -> PhysicsSceneStepResult {
        let step_plan = self.plan_world_step(world_handle, delta_seconds);
        PhysicsSceneStepResult {
            step_plan,
            contacts: self.drain_contacts(world_handle),
        }
    }
}
