use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;
use crate::core::CoreError;
use crate::scene::World;

use super::{
    PhysicsBackendStatus, PhysicsContactEvent, PhysicsMaterialMetadata, PhysicsRayCastHit,
    PhysicsRayCastQuery, PhysicsSceneStepResult, PhysicsSettings, PhysicsShapeCastHit,
    PhysicsShapeCastQuery, PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery, PhysicsTriggerEvent,
    PhysicsWorldStepPlan, PhysicsWorldSyncState,
};

pub trait PhysicsManager: Send + Sync {
    fn backend_name(&self) -> String;
    fn settings(&self) -> PhysicsSettings;
    fn store_settings(&self, _settings: PhysicsSettings) -> Result<(), CoreError> {
        Err(CoreError::Initialization(
            "PhysicsManager".to_string(),
            "settings are read-only for this backend".to_string(),
        ))
    }
    fn default_material(&self) -> PhysicsMaterialMetadata;
    fn backend_status(&self) -> PhysicsBackendStatus;
    fn plan_world_step(&self, world: WorldHandle, delta_seconds: Real) -> PhysicsWorldStepPlan;
    fn sync_world(&self, sync: PhysicsWorldSyncState);
    fn synchronized_world(&self, world: WorldHandle) -> Option<PhysicsWorldSyncState>;
    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Option<PhysicsRayCastHit>;
    fn shape_overlap(&self, _query: &PhysicsShapeOverlapQuery) -> Vec<PhysicsShapeOverlapHit> {
        Vec::new()
    }
    fn shape_cast(&self, _query: &PhysicsShapeCastQuery) -> Option<PhysicsShapeCastHit> {
        None
    }
    fn drain_contacts(&self, world: WorldHandle) -> Vec<PhysicsContactEvent>;
    fn drain_triggers(&self, _world: WorldHandle) -> Vec<PhysicsTriggerEvent> {
        Vec::new()
    }
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
            triggers: self.drain_triggers(world_handle),
        }
    }
}
