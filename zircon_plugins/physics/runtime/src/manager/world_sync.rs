use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use zircon_runtime::core::framework::physics::{
    PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderShape, PhysicsColliderSyncState,
    PhysicsContactEvent, PhysicsJointSyncState, PhysicsJointType, PhysicsMaterialMetadata,
    PhysicsMaterialSyncState, PhysicsTriggerEvent, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Transform;
use zircon_runtime::scene::components::{ColliderShape, JointKind, RigidBodyType};
use zircon_runtime::scene::world::World;

use crate::trigger::PhysicsTriggerPairMap;

use super::validation::{
    collider_layer_sync_input_is_valid, collider_shape_sync_input_is_valid,
    joint_sync_input_is_finite, material_locator_sync_input_is_valid,
    material_metadata_sync_input_is_finite, physics_body_sync_state_is_valid,
    physics_collider_shape_is_valid, physics_collider_sync_state_is_valid,
    physics_joint_sync_state_is_valid, physics_material_sync_state_is_valid,
    rigid_body_sync_input_is_finite, transform_is_finite,
};

pub fn build_world_sync_state(world_handle: WorldHandle, world: &World) -> PhysicsWorldSyncState {
    let mut sync = PhysicsWorldSyncState {
        world: world_handle,
        ..PhysicsWorldSyncState::default()
    };

    for node in world.node_records() {
        let entity_transform = world.world_transform(node.id).unwrap_or(node.transform);

        if let Some(rigid_body) = node.rigid_body.as_ref() {
            if transform_is_finite(entity_transform) && rigid_body_sync_input_is_finite(rigid_body)
            {
                sync.bodies.push(PhysicsBodySyncState {
                    entity: node.id,
                    body_type: match rigid_body.body_type {
                        RigidBodyType::Static => PhysicsBodyType::Static,
                        RigidBodyType::Dynamic => PhysicsBodyType::Dynamic,
                        RigidBodyType::Kinematic => PhysicsBodyType::Kinematic,
                    },
                    transform: entity_transform,
                    mass: rigid_body.mass,
                    linear_velocity: rigid_body.linear_velocity.to_array(),
                    angular_velocity: rigid_body.angular_velocity.to_array(),
                    linear_damping: rigid_body.linear_damping,
                    angular_damping: rigid_body.angular_damping,
                    gravity_scale: rigid_body.gravity_scale,
                    can_sleep: rigid_body.can_sleep,
                    lock_translation: rigid_body.lock_translation,
                    lock_rotation: rigid_body.lock_rotation,
                });
            }
        }

        if let Some(collider) = node.collider.as_ref() {
            let transform = combine_transforms(entity_transform, collider.local_transform);
            if transform_is_finite(transform)
                && collider_shape_sync_input_is_valid(&collider.shape)
                && collider_layer_sync_input_is_valid(collider.layer)
                && collider
                    .material_override
                    .as_ref()
                    .is_none_or(material_metadata_sync_input_is_finite)
            {
                sync.colliders.push(PhysicsColliderSyncState {
                    entity: node.id,
                    shape: match &collider.shape {
                        ColliderShape::Box { half_extents } => PhysicsColliderShape::Box {
                            half_extents: half_extents.to_array(),
                        },
                        ColliderShape::Sphere { radius } => {
                            PhysicsColliderShape::Sphere { radius: *radius }
                        }
                        ColliderShape::Capsule {
                            radius,
                            half_height,
                        } => PhysicsColliderShape::Capsule {
                            radius: *radius,
                            half_height: *half_height,
                        },
                    },
                    sensor: collider.sensor,
                    layer: collider.layer,
                    collision_group: collider.collision_group,
                    collision_mask: collider.collision_mask,
                    material: collider.material.map(|handle| handle.id().to_string()),
                    material_override: collider.material_override.clone(),
                    transform,
                });

                if collider.material.is_some() || collider.material_override.is_some() {
                    sync.materials.push(PhysicsMaterialSyncState {
                        entity: node.id,
                        locator: collider.material.map(|handle| handle.id().to_string()),
                        material: collider
                            .material_override
                            .clone()
                            .unwrap_or_else(PhysicsMaterialMetadata::default),
                    });
                }
            }
        }

        if let Some(joint) = node.joint.as_ref() {
            if joint_sync_input_is_finite(joint) {
                sync.joints.push(PhysicsJointSyncState {
                    entity: node.id,
                    kind: match joint.joint_type {
                        JointKind::Fixed => PhysicsJointType::Fixed,
                        JointKind::Distance => PhysicsJointType::Distance,
                        JointKind::Hinge => PhysicsJointType::Hinge,
                        JointKind::Slider => PhysicsJointType::Slider,
                        JointKind::ConeTwist => PhysicsJointType::ConeTwist,
                        JointKind::Generic6Dof => PhysicsJointType::Generic6Dof,
                    },
                    connected_entity: joint.connected_entity,
                    anchor: joint.anchor.to_array(),
                    axis: joint.axis.to_array(),
                    limits: joint.limits,
                    collide_connected: joint.collide_connected,
                    constraint: joint.constraint.clone(),
                    skeleton_binding: joint.skeleton_binding.clone(),
                });
            }
        }
    }

    sync
}

pub(super) fn clear_world_state(
    world: WorldHandle,
    synced_worlds: &Mutex<HashMap<WorldHandle, PhysicsWorldSyncState>>,
    contacts: &Mutex<HashMap<WorldHandle, Vec<PhysicsContactEvent>>>,
    trigger_pairs: &Mutex<HashMap<WorldHandle, PhysicsTriggerPairMap>>,
    triggers: &Mutex<HashMap<WorldHandle, Vec<PhysicsTriggerEvent>>>,
) {
    synced_worlds
        .lock()
        .expect("physics sync mutex poisoned")
        .remove(&world);
    contacts
        .lock()
        .expect("physics contact mutex poisoned")
        .remove(&world);
    trigger_pairs
        .lock()
        .expect("physics trigger pair mutex poisoned")
        .remove(&world);
    triggers
        .lock()
        .expect("physics trigger mutex poisoned")
        .remove(&world);
}

pub(super) fn sanitize_world_sync_state(mut sync: PhysicsWorldSyncState) -> PhysicsWorldSyncState {
    let mut synced_body_entities = HashSet::new();
    sync.bodies.retain(|body| {
        physics_body_sync_state_is_valid(body) && synced_body_entities.insert(body.entity)
    });
    let mut synced_collider_entities = HashSet::new();
    sync.colliders.retain(|collider| {
        physics_collider_sync_state_is_valid(collider)
            && synced_collider_entities.insert(collider.entity)
    });
    let mut synced_joint_entities = HashSet::new();
    sync.joints.retain(|joint| {
        physics_joint_sync_state_is_valid(joint) && synced_joint_entities.insert(joint.entity)
    });
    let material_bound_collider_locators = sync
        .colliders
        .iter()
        .filter(|collider| collider.material.is_some() || collider.material_override.is_some())
        .map(|collider| (collider.entity, collider.material.clone()))
        .collect::<HashMap<_, _>>();
    let mut synced_material_entities = HashSet::new();
    sync.materials.retain(|material| {
        physics_material_sync_state_is_valid(material)
            && material_bound_collider_locators
                .get(&material.entity)
                .is_some_and(|locator| locator == &material.locator)
            && synced_material_entities.insert(material.entity)
    });
    sync
}

fn combine_transforms(parent: Transform, local: Transform) -> Transform {
    Transform {
        translation: parent.translation + parent.rotation * (parent.scale * local.translation),
        rotation: parent.rotation * local.rotation,
        scale: parent.scale * local.scale,
    }
}
