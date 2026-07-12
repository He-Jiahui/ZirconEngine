use zircon_runtime::core::framework::physics::{PhysicsColliderShape, PhysicsWorldSyncState};
use zircon_runtime::core::math::Transform;
use zircon_runtime::scene::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsOverlayColor {
    Collider,
    Trigger,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsOverlayPrimitive {
    pub entity: EntityId,
    pub shape: PhysicsColliderShape,
    pub transform: Transform,
    pub color: PhysicsOverlayColor,
}

pub fn build_physics_overlay(sync: &PhysicsWorldSyncState) -> Vec<PhysicsOverlayPrimitive> {
    sync.colliders
        .iter()
        .map(|collider| PhysicsOverlayPrimitive {
            entity: collider.entity,
            shape: collider.shape.clone(),
            transform: collider.transform,
            color: if collider.sensor {
                PhysicsOverlayColor::Trigger
            } else {
                PhysicsOverlayColor::Collider
            },
        })
        .collect()
}
