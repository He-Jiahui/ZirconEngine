use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsColliderShape, PhysicsQueryFilter, PhysicsRayCastHit,
        PhysicsRayCastQuery, PhysicsShapeCastHit, PhysicsShapeCastQuery, PhysicsShapeOverlapHit,
        PhysicsShapeOverlapQuery,
    },
    scene::physics::PhysicsMaterialMetadata,
};
use zircon_runtime::core::math::Real;

use super::{
    BodyCommand, BodyDesc, BodyHandle, ConstraintDesc, ConstraintHandle, PhysicsBackendError,
    PhysicsEventBuffer, ShapeHandle,
};

pub trait PhysicsBackend: Send {
    fn name(&self) -> &'static str;
    fn create_shape(
        &mut self,
        shape: &PhysicsColliderShape,
        material: &PhysicsMaterialMetadata,
    ) -> Result<ShapeHandle, PhysicsBackendError>;
    fn create_body(&mut self, desc: &BodyDesc) -> Result<BodyHandle, PhysicsBackendError>;
    fn create_constraint(
        &mut self,
        desc: &ConstraintDesc,
    ) -> Result<ConstraintHandle, PhysicsBackendError>;
    fn destroy_shape(&mut self, shape: ShapeHandle) -> Result<(), PhysicsBackendError>;
    fn destroy_body(&mut self, body: BodyHandle) -> Result<(), PhysicsBackendError>;
    fn destroy_constraint(
        &mut self,
        constraint: ConstraintHandle,
    ) -> Result<(), PhysicsBackendError>;
    fn apply_commands(&mut self, commands: &[BodyCommand]) -> Result<(), PhysicsBackendError>;
    fn step(&mut self, dt: Real) -> Result<(), PhysicsBackendError>;
    fn read_active_states(&mut self, out: &mut Vec<(BodyHandle, PhysicsBodySyncState)>);
    fn ray_cast(
        &self,
        query: &PhysicsRayCastQuery,
        filter: &PhysicsQueryFilter,
        out: &mut Vec<PhysicsRayCastHit>,
    );
    fn shape_cast(
        &self,
        query: &PhysicsShapeCastQuery,
        filter: &PhysicsQueryFilter,
        out: &mut Vec<PhysicsShapeCastHit>,
    );
    fn shape_overlap(
        &self,
        query: &PhysicsShapeOverlapQuery,
        filter: &PhysicsQueryFilter,
        out: &mut Vec<PhysicsShapeOverlapHit>,
    );
    fn drain_events(&mut self, out: &mut PhysicsEventBuffer);
}
