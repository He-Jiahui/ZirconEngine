use crate::core::framework::scene::physics::PhysicsMaterialMetadata;
use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;

use super::{
    PhysicsBackendStatus, PhysicsContactEvent, PhysicsRayCastHit, PhysicsRayCastQuery,
    PhysicsSettings, PhysicsSettingsStoreError, PhysicsShapeCastHit, PhysicsShapeCastQuery,
    PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery, PhysicsTriggerEvent, PhysicsWorldStepPlan,
    PhysicsWorldSyncState,
};

pub trait PhysicsManager: Send + Sync {
    fn backend_name(&self) -> String;
    fn settings(&self) -> PhysicsSettings;
    fn store_settings(&self, _settings: PhysicsSettings) -> Result<(), PhysicsSettingsStoreError> {
        Err(PhysicsSettingsStoreError::read_only_backend(
            self.backend_name(),
        ))
    }
    fn default_material(&self) -> PhysicsMaterialMetadata;
    fn backend_status(&self) -> PhysicsBackendStatus;
    fn plan_world_step(&self, world: WorldHandle, delta_seconds: Real) -> PhysicsWorldStepPlan;
    fn sync_world(&self, sync: PhysicsWorldSyncState);
    fn synchronized_world(&self, world: WorldHandle) -> Option<PhysicsWorldSyncState>;
    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Vec<PhysicsRayCastHit>;
    fn shape_overlap(&self, _query: &PhysicsShapeOverlapQuery) -> Vec<PhysicsShapeOverlapHit> {
        Vec::new()
    }
    fn shape_cast(&self, _query: &PhysicsShapeCastQuery) -> Vec<PhysicsShapeCastHit> {
        Vec::new()
    }
    fn drain_contacts(&self, world: WorldHandle) -> Vec<PhysicsContactEvent>;
    fn drain_triggers(&self, _world: WorldHandle) -> Vec<PhysicsTriggerEvent> {
        Vec::new()
    }
}
