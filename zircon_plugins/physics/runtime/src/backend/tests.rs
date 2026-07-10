mod builtin_contract;

#[cfg(feature = "backend-jolt")]
mod jolt_contract;

use zircon_runtime::core::framework::physics::{
    PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderShape, PhysicsColliderSyncState,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::{Transform, Vec3};

use super::{BodyDesc, ShapeHandle};

fn body_desc(world: WorldHandle, entity: EntityId, shape: ShapeHandle) -> BodyDesc {
    let transform = Transform {
        translation: Vec3::new(0.0, 2.0, 0.0),
        ..Transform::default()
    };
    BodyDesc::from_sync(
        world,
        shape,
        &PhysicsBodySyncState {
            entity,
            body_type: PhysicsBodyType::Dynamic,
            transform,
            mass: 2.0,
            linear_velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            can_sleep: true,
            lock_translation: [false; 3],
            lock_rotation: [false; 3],
        },
        &PhysicsColliderSyncState {
            entity,
            shape: PhysicsColliderShape::Sphere { radius: 0.5 },
            sensor: false,
            layer: 0,
            collision_group: 7,
            collision_mask: u32::MAX,
            material: None,
            material_override: None,
            transform,
        },
    )
    .expect("matching body and collider sync state")
}
