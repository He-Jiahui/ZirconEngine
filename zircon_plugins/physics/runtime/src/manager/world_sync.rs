use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::physics::{
    PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderShape, PhysicsColliderSyncState,
    PhysicsContactEvent, PhysicsJointSyncState, PhysicsJointType, PhysicsMaterialSyncState,
    PhysicsTriggerEvent, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::{
    is_finite_mat4, transform_to_mat4, try_affine_inverse, Mat4, NumericPolicy, Transform,
    ValidatedTransform, Vec3,
};
use zircon_runtime::scene::components::{ColliderShape, JointKind, RigidBodyType};
use zircon_runtime::scene::world::World;

use super::poison_recovery::recover_lock;
use super::validation::{
    collider_layer_sync_input_is_valid, collider_shape_sync_input_is_valid,
    joint_sync_input_is_finite, material_metadata_sync_input_is_finite,
    physics_body_sync_state_is_valid, physics_collider_sync_state_is_valid,
    physics_joint_sync_state_is_valid, physics_material_sync_state_is_valid,
    rigid_body_sync_input_is_finite, transform_is_finite,
};
use crate::backend::builtin::PhysicsTriggerPairMap;

pub fn build_world_sync_state(world_handle: WorldHandle, world: &World) -> PhysicsWorldSyncState {
    let mut entities = world.stable_entity_ids().collect::<Vec<_>>();
    entities.sort_unstable();
    let (body_capacity, collider_capacity, joint_capacity, material_capacity) =
        world_sync_projection_capacities(world, &entities);
    let mut sync = PhysicsWorldSyncState {
        world: world_handle,
        bodies: Vec::with_capacity(body_capacity),
        colliders: Vec::with_capacity(collider_capacity),
        joints: Vec::with_capacity(joint_capacity),
        materials: Vec::with_capacity(material_capacity),
    };

    for entity in entities {
        let entity_transform = world
            .world_transform(entity)
            .or_else(|| world.local_transform(entity))
            .unwrap_or_default();

        if let Some(rigid_body) = world.rigid_body(entity) {
            if transform_is_finite(entity_transform) && rigid_body_sync_input_is_finite(&rigid_body)
            {
                sync.bodies.push(PhysicsBodySyncState {
                    entity,
                    body_type: match rigid_body.body_type {
                        RigidBodyType::Static => PhysicsBodyType::Static,
                        RigidBodyType::Dynamic => PhysicsBodyType::Dynamic,
                        RigidBodyType::Kinematic => PhysicsBodyType::Kinematic,
                    },
                    transform: entity_transform,
                    mass: rigid_body.mass,
                    mass_properties: rigid_body.mass_properties,
                    linear_velocity: rigid_body.linear_velocity.to_array(),
                    angular_velocity: rigid_body.angular_velocity.to_array(),
                    linear_damping: rigid_body.linear_damping,
                    angular_damping: rigid_body.angular_damping,
                    gravity_scale: rigid_body.gravity_scale,
                    ccd_mode: rigid_body.ccd_mode,
                    sleep_policy: rigid_body.sleep_policy,
                    lock_translation: rigid_body.lock_translation,
                    lock_rotation: rigid_body.lock_rotation,
                });
            }
        }

        if let Some(collider) = world.collider(entity) {
            let transform = combine_transforms(entity_transform, collider.local_transform);
            if transform_is_finite(transform)
                && collider_shape_sync_input_is_valid(&collider.shape)
                && collider_layer_sync_input_is_valid(collider.layer)
                && collider
                    .material_override
                    .as_ref()
                    .is_none_or(material_metadata_sync_input_is_finite)
            {
                let material_locator = collider.material.map(|handle| handle.id().to_string());
                let material_override = collider.material_override.clone();
                let material = if material_locator.is_some() || material_override.is_some() {
                    Some(PhysicsMaterialSyncState {
                        entity,
                        locator: material_locator.clone(),
                        material: material_override.clone().unwrap_or_default(),
                    })
                } else {
                    None
                };
                sync.colliders.push(PhysicsColliderSyncState {
                    entity,
                    shape: collider_shape_to_physics(&collider.shape),
                    sensor: collider.sensor,
                    layer: collider.layer,
                    collision_group: collider.collision_group,
                    collision_mask: collider.collision_mask,
                    material: material_locator,
                    material_override,
                    transform,
                });
                sync.materials.extend(material);
            }
        }

        if let Some(joint) = world.joint(entity) {
            if joint_sync_input_is_finite(&joint) {
                sync.joints.push(PhysicsJointSyncState {
                    entity,
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

fn world_sync_projection_capacities(
    world: &World,
    entities: &[u64],
) -> (usize, usize, usize, usize) {
    entities.iter().fold(
        (0, 0, 0, 0),
        |(bodies, colliders, joints, materials), &entity| {
            let collider = world.collider(entity);
            (
                bodies + usize::from(world.rigid_body(entity).is_some()),
                colliders + usize::from(collider.is_some()),
                joints + usize::from(world.joint(entity).is_some()),
                materials
                    + usize::from(
                        collider
                            .is_some_and(|c| c.material.is_some() || c.material_override.is_some()),
                    ),
            )
        },
    )
}

pub(super) fn collider_shape_to_physics(shape: &ColliderShape) -> PhysicsColliderShape {
    match shape {
        ColliderShape::Box { half_extents } => PhysicsColliderShape::Box {
            half_extents: half_extents.to_array(),
        },
        ColliderShape::Sphere { radius } => PhysicsColliderShape::Sphere { radius: *radius },
        ColliderShape::Capsule {
            radius,
            half_height,
        } => PhysicsColliderShape::Capsule {
            radius: *radius,
            half_height: *half_height,
        },
        ColliderShape::Cylinder {
            radius,
            half_height,
        } => PhysicsColliderShape::Cylinder {
            radius: *radius,
            half_height: *half_height,
        },
        ColliderShape::ConvexHull { points } => PhysicsColliderShape::ConvexHull {
            points: points.iter().map(|point| point.to_array()).collect(),
        },
        ColliderShape::TriangleMesh { mesh } => {
            PhysicsColliderShape::TriangleMesh { mesh: mesh.clone() }
        }
        ColliderShape::HeightField {
            resolution,
            heights,
        } => PhysicsColliderShape::HeightField {
            resolution: *resolution,
            heights: heights.clone(),
        },
        ColliderShape::Compound { children } => PhysicsColliderShape::Compound {
            children: children
                .iter()
                .map(|(transform, child)| (*transform, Box::new(collider_shape_to_physics(child))))
                .collect(),
        },
    }
}

fn collider_shape_into_physics(shape: ColliderShape) -> PhysicsColliderShape {
    match shape {
        ColliderShape::Box { half_extents } => PhysicsColliderShape::Box {
            half_extents: half_extents.to_array(),
        },
        ColliderShape::Sphere { radius } => PhysicsColliderShape::Sphere { radius },
        ColliderShape::Capsule {
            radius,
            half_height,
        } => PhysicsColliderShape::Capsule {
            radius,
            half_height,
        },
        ColliderShape::Cylinder {
            radius,
            half_height,
        } => PhysicsColliderShape::Cylinder {
            radius,
            half_height,
        },
        ColliderShape::ConvexHull { points } => PhysicsColliderShape::ConvexHull {
            points: points.into_iter().map(|point| point.to_array()).collect(),
        },
        ColliderShape::TriangleMesh { mesh } => PhysicsColliderShape::TriangleMesh { mesh },
        ColliderShape::HeightField {
            resolution,
            heights,
        } => PhysicsColliderShape::HeightField {
            resolution,
            heights,
        },
        ColliderShape::Compound { children } => PhysicsColliderShape::Compound {
            children: children
                .into_iter()
                .map(|(transform, child)| {
                    (transform, Box::new(collider_shape_into_physics(*child)))
                })
                .collect(),
        },
    }
}

pub(crate) fn apply_synchronized_bodies_to_scene(scene: &mut World, sync: &PhysicsWorldSyncState) {
    let mut attached = Vec::new();
    let entity_count = scene.entity_count();
    for body in &sync.bodies {
        if let Some(parent) = scene.parent_of(body.entity) {
            let mut depth = 1;
            let mut ancestor = parent;
            while depth < entity_count {
                let Some(next) = scene.parent_of(ancestor) else {
                    break;
                };
                ancestor = next;
                depth += 1;
            }
            attached.push((depth, parent, body));
        } else {
            apply_synchronized_body(scene, body, None);
        }
    }
    // A stable entity ID is not a hierarchy order. Read each child's actual
    // parent matrix after accepted ancestor poses, including dirty chains.
    attached.sort_unstable_by_key(|(depth, _, body)| (*depth, body.entity));
    for (_, parent, body) in attached {
        apply_synchronized_body(scene, body, Some(parent));
    }
}

fn apply_synchronized_body(
    scene: &mut World,
    body: &PhysicsBodySyncState,
    parent: Option<EntityId>,
) {
    if !physics_body_sync_state_is_valid(body) {
        return;
    }
    let Some(mut rigid_body) = scene.rigid_body(body.entity).cloned() else {
        return;
    };
    rigid_body.body_type = match body.body_type {
        PhysicsBodyType::Static => RigidBodyType::Static,
        PhysicsBodyType::Dynamic => RigidBodyType::Dynamic,
        PhysicsBodyType::Kinematic => RigidBodyType::Kinematic,
    };
    rigid_body.mass = body.mass;
    rigid_body.mass_properties = body.mass_properties;
    rigid_body.linear_velocity = Vec3::from_array(body.linear_velocity);
    rigid_body.angular_velocity = Vec3::from_array(body.angular_velocity);
    rigid_body.linear_damping = body.linear_damping;
    rigid_body.angular_damping = body.angular_damping;
    rigid_body.gravity_scale = body.gravity_scale;
    rigid_body.ccd_mode = body.ccd_mode;
    rigid_body.sleep_policy = body.sleep_policy;
    rigid_body.lock_translation = body.lock_translation;
    rigid_body.lock_rotation = body.lock_rotation;

    let local_pose = match parent {
        None => Some(body.transform),
        // The outgoing solver projection is TRS, not a copy of the affine
        // scene matrix. Do not write that lossy projection back as new geometry.
        Some(_) if scene.world_transform(body.entity) == Some(body.transform) => None,
        Some(parent) => parented_body_local_pose(scene, parent, body.transform),
    };
    if let Some(local_pose) = local_pose {
        let _ = scene.update_transform(body.entity, local_pose);
    }
    // Refusing an unrepresentable pose must not discard valid body properties.
    let _ = scene.set_rigid_body(body.entity, Some(rigid_body));
}

fn parented_body_local_pose(
    scene: &World,
    parent: EntityId,
    world_pose: Transform,
) -> Option<Transform> {
    let parent_world = scene.world_matrix(parent)?;
    let parent_inverse = try_affine_inverse(parent_world, NumericPolicy::STRICT).ok()?;
    let requested_world = transform_to_mat4(
        ValidatedTransform::try_new(world_pose, NumericPolicy::STRICT)
            .ok()?
            .into_transform(),
    );
    let requested_local = parent_inverse * requested_world;
    let determinant = requested_local.determinant();
    if !is_finite_mat4(requested_local)
        || !determinant.is_finite()
        || determinant == 0.0
        || [
            requested_local.x_axis,
            requested_local.y_axis,
            requested_local.z_axis,
        ]
        .into_iter()
        .any(|axis| {
            let length_squared = axis.truncate().length_squared();
            !length_squared.is_finite() || length_squared == 0.0
        })
    {
        return None;
    }
    let (scale, rotation, translation) = requested_local.to_scale_rotation_translation();
    let local = ValidatedTransform::try_new(
        Transform {
            translation,
            rotation,
            scale,
        },
        NumericPolicy::STRICT,
    )
    .ok()?
    .into_transform();
    let recomposed_local = transform_to_mat4(local);
    (pose_matrix_matches(requested_local, recomposed_local)
        && pose_matrix_matches(requested_world, parent_world * recomposed_local))
    .then_some(local)
}

fn pose_matrix_matches(expected: Mat4, actual: Mat4) -> bool {
    const RELATIVE_TOLERANCE: f32 = 1.0e-5;
    if !is_finite_mat4(expected) || !is_finite_mat4(actual) {
        return false;
    }
    // Measure each affine column separately: a large translation must not
    // make lost shear (or another change to the basis) pass the check.
    expected
        .to_cols_array_2d()
        .into_iter()
        .zip(actual.to_cols_array_2d())
        .all(|(expected, actual)| {
            let magnitude = expected.iter().map(|value| value.abs()).fold(0.0, f32::max);
            expected
                .into_iter()
                .zip(actual)
                .all(|(value, actual)| (value - actual).abs() <= magnitude * RELATIVE_TOLERANCE)
        })
}

pub(super) fn clear_world_state(
    world: WorldHandle,
    synced_worlds: &Mutex<HashMap<WorldHandle, Arc<PhysicsWorldSyncState>>>,
    contacts: &Mutex<HashMap<WorldHandle, Vec<PhysicsContactEvent>>>,
    trigger_pairs: &Mutex<HashMap<WorldHandle, PhysicsTriggerPairMap>>,
    triggers: &Mutex<HashMap<WorldHandle, Vec<PhysicsTriggerEvent>>>,
) {
    recover_lock(synced_worlds).remove(&world);
    recover_lock(contacts).remove(&world);
    recover_lock(trigger_pairs).remove(&world);
    recover_lock(triggers).remove(&world);
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

#[cfg(test)]
#[path = "tests/world_sync.rs"]
mod tests;
