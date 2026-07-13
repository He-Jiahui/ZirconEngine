use crate::core::framework::bridge::PluginInterface;

use super::{
    PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsShapeCastHit, PhysicsShapeCastQuery,
    PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery,
};

pub const PHYSICS_QUERY_INTERFACE_ID: &str = "physics.query.v1";

/// Runtime-owned query surface for plugin-to-plugin physics calls.
pub trait PhysicsQueryInterface: Send + Sync {
    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Vec<PhysicsRayCastHit>;
    fn shape_overlap(&self, query: &PhysicsShapeOverlapQuery) -> Vec<PhysicsShapeOverlapHit>;
    fn shape_cast(&self, query: &PhysicsShapeCastQuery) -> Vec<PhysicsShapeCastHit>;
}

impl PluginInterface for dyn PhysicsQueryInterface {
    const INTERFACE_ID: &'static str = PHYSICS_QUERY_INTERFACE_ID;
}
