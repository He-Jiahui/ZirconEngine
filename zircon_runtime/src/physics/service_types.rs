use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::framework::physics::{
    PhysicsBackendState, PhysicsBackendStatus, PhysicsBodySyncState, PhysicsBodyType,
    PhysicsColliderShape, PhysicsColliderSyncState, PhysicsContactEvent, PhysicsJointSyncState,
    PhysicsJointType, PhysicsMaterialMetadata, PhysicsMaterialSyncState, PhysicsRayCastHit,
    PhysicsRayCastQuery, PhysicsSettings, PhysicsSimulationMode, PhysicsWorldStepPlan,
    PhysicsWorldSyncState,
};
use crate::core::framework::scene::WorldHandle;
use crate::core::math::{Real, Transform, Vec3};
use crate::core::{CoreError, CoreHandle};
use crate::scene::components::{ColliderShape, JointKind, RigidBodyType};
use crate::scene::world::World;

use super::PhysicsInterface;

pub const JOLT_ENABLED: bool = cfg!(feature = "jolt");

pub type PhysicsTickPlan = PhysicsWorldStepPlan;

#[derive(Clone, Debug, Default)]
pub struct PhysicsDriver;

#[derive(Clone, Debug)]
pub struct DefaultPhysicsManager {
    core: Option<CoreHandle>,
    settings: Arc<Mutex<PhysicsSettings>>,
    default_material: PhysicsMaterialMetadata,
    accumulators: Arc<Mutex<HashMap<WorldHandle, f32>>>,
    synced_worlds: Arc<Mutex<HashMap<WorldHandle, PhysicsWorldSyncState>>>,
    contacts: Arc<Mutex<HashMap<WorldHandle, Vec<PhysicsContactEvent>>>>,
}

impl Default for DefaultPhysicsManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultPhysicsManager {
    pub fn new(core: Option<CoreHandle>) -> Self {
        let settings = core
            .as_ref()
            .and_then(|core| core.load_config(super::PHYSICS_SETTINGS_CONFIG_KEY).ok())
            .unwrap_or_else(default_settings);
        Self {
            core,
            settings: Arc::new(Mutex::new(settings)),
            default_material: PhysicsMaterialMetadata::default(),
            accumulators: Arc::new(Mutex::new(HashMap::new())),
            synced_worlds: Arc::new(Mutex::new(HashMap::new())),
            contacts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn store_settings(&self, settings: PhysicsSettings) -> Result<(), CoreError> {
        *self
            .settings
            .lock()
            .expect("physics settings mutex poisoned") = settings.clone();
        if let Some(core) = &self.core {
            core.store_config(super::PHYSICS_SETTINGS_CONFIG_KEY, &settings)?;
        }
        Ok(())
    }

    pub fn advance_clock(&self, world: WorldHandle, delta_seconds: f32) -> PhysicsTickPlan {
        const STEP_EPSILON_SCALE: f32 = 1.0e-4;

        let settings = self
            .settings
            .lock()
            .expect("physics settings mutex poisoned")
            .clone();
        let step_seconds = if settings.fixed_hz == 0 {
            0.0
        } else {
            1.0 / settings.fixed_hz as f32
        };
        if settings.simulation_mode == PhysicsSimulationMode::Disabled || step_seconds <= 0.0 {
            return PhysicsWorldStepPlan {
                steps: 0,
                step_seconds,
                remaining_seconds: 0.0,
            };
        }

        let mut accumulators = self
            .accumulators
            .lock()
            .expect("physics accumulator mutex poisoned");
        let accumulator = accumulators.entry(world).or_insert(0.0);
        *accumulator += delta_seconds.max(0.0);

        let max_substeps = settings.max_substeps.max(1);
        let step_epsilon = step_seconds * STEP_EPSILON_SCALE;
        let mut steps = 0;
        while steps < max_substeps && *accumulator + step_epsilon >= step_seconds {
            *accumulator = (*accumulator - step_seconds).max(0.0);
            steps += 1;
        }
        if accumulator.abs() < step_epsilon {
            *accumulator = 0.0;
        }

        PhysicsWorldStepPlan {
            steps,
            step_seconds,
            remaining_seconds: *accumulator,
        }
    }
}

pub fn build_world_sync_state(world_handle: WorldHandle, world: &World) -> PhysicsWorldSyncState {
    let mut sync = PhysicsWorldSyncState {
        world: world_handle,
        ..PhysicsWorldSyncState::default()
    };

    for node in world.nodes() {
        let entity_transform = world.world_transform(node.id).unwrap_or(node.transform);

        if let Some(rigid_body) = node.rigid_body.as_ref() {
            sync.bodies.push(PhysicsBodySyncState {
                entity: node.id,
                body_type: match rigid_body.body_type {
                    RigidBodyType::Static => PhysicsBodyType::Static,
                    RigidBodyType::Dynamic => PhysicsBodyType::Dynamic,
                    RigidBodyType::Kinematic => PhysicsBodyType::Kinematic,
                },
                transform: entity_transform,
                mass: rigid_body.mass,
                linear_damping: rigid_body.linear_damping,
                angular_damping: rigid_body.angular_damping,
                gravity_scale: rigid_body.gravity_scale,
                can_sleep: rigid_body.can_sleep,
                lock_translation: rigid_body.lock_translation,
                lock_rotation: rigid_body.lock_rotation,
            });
        }

        if let Some(collider) = node.collider.as_ref() {
            let transform = combine_transforms(entity_transform, collider.local_transform);
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

        if let Some(joint) = node.joint.as_ref() {
            sync.joints.push(PhysicsJointSyncState {
                entity: node.id,
                kind: match joint.joint_type {
                    JointKind::Fixed => PhysicsJointType::Fixed,
                    JointKind::Distance => PhysicsJointType::Distance,
                    JointKind::Hinge => PhysicsJointType::Hinge,
                },
                connected_entity: joint.connected_entity,
                anchor: joint.anchor.to_array(),
                axis: joint.axis.to_array(),
                limits: joint.limits,
                collide_connected: joint.collide_connected,
            });
        }
    }

    sync
}

fn default_settings() -> PhysicsSettings {
    PhysicsSettings {
        backend: if JOLT_ENABLED {
            "jolt".to_string()
        } else {
            "unconfigured".to_string()
        },
        simulation_mode: if JOLT_ENABLED {
            PhysicsSimulationMode::Simulate
        } else {
            PhysicsSimulationMode::Disabled
        },
        ..PhysicsSettings::default()
    }
}

impl crate::core::framework::physics::PhysicsManager for DefaultPhysicsManager {
    fn backend_name(&self) -> String {
        self.settings().backend
    }

    fn settings(&self) -> PhysicsSettings {
        self.settings
            .lock()
            .expect("physics settings mutex poisoned")
            .clone()
    }

    fn default_material(&self) -> PhysicsMaterialMetadata {
        self.default_material.clone()
    }

    fn backend_status(&self) -> PhysicsBackendStatus {
        physics_backend_status(&self.settings())
    }

    fn plan_world_step(&self, world: WorldHandle, delta_seconds: Real) -> PhysicsWorldStepPlan {
        self.advance_clock(world, delta_seconds)
    }

    fn sync_world(&self, sync: PhysicsWorldSyncState) {
        let contacts = compute_contact_events(&sync);
        self.synced_worlds
            .lock()
            .expect("physics sync mutex poisoned")
            .insert(sync.world, sync.clone());
        self.contacts
            .lock()
            .expect("physics contact mutex poisoned")
            .insert(sync.world, contacts);
    }

    fn synchronized_world(&self, world: WorldHandle) -> Option<PhysicsWorldSyncState> {
        self.synced_worlds
            .lock()
            .expect("physics sync mutex poisoned")
            .get(&world)
            .cloned()
    }

    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Option<PhysicsRayCastHit> {
        let direction = Vec3::from_array(query.direction).normalize_or_zero();
        if direction.length_squared() <= Real::EPSILON || query.max_distance <= 0.0 {
            return None;
        }

        self.synchronized_world(query.world)?
            .colliders
            .iter()
            .filter(|collider| query.include_sensors || !collider.sensor)
            .filter(|collider| {
                query
                    .collision_mask
                    .is_none_or(|mask| collider.collision_mask & mask != 0)
            })
            .filter_map(|collider| {
                ray_cast_collider(
                    Vec3::from_array(query.origin),
                    direction,
                    query.max_distance,
                    collider,
                )
            })
            .min_by(|left, right| {
                left.distance
                    .partial_cmp(&right.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn drain_contacts(&self, world: WorldHandle) -> Vec<PhysicsContactEvent> {
        self.contacts
            .lock()
            .expect("physics contact mutex poisoned")
            .remove(&world)
            .unwrap_or_default()
    }
}

impl PhysicsInterface for DefaultPhysicsManager {
    fn store_settings(&self, settings: PhysicsSettings) -> Result<(), CoreError> {
        self.store_settings(settings)
    }

    fn advance_clock(&self, world: WorldHandle, delta_seconds: Real) -> PhysicsTickPlan {
        self.advance_clock(world, delta_seconds)
    }
}

fn physics_backend_status(settings: &PhysicsSettings) -> PhysicsBackendStatus {
    let requested_backend = settings.backend.clone();
    let feature_gate = requested_backend
        .eq_ignore_ascii_case("jolt")
        .then_some("jolt".to_string());
    if settings.simulation_mode == PhysicsSimulationMode::Disabled {
        return PhysicsBackendStatus {
            requested_backend,
            active_backend: None,
            state: PhysicsBackendState::Disabled,
            detail: Some("physics simulation is disabled".to_string()),
            simulation_mode: settings.simulation_mode,
            feature_gate,
        };
    }

    if requested_backend.eq_ignore_ascii_case("jolt") && !JOLT_ENABLED {
        return PhysicsBackendStatus {
            requested_backend,
            active_backend: None,
            state: PhysicsBackendState::Unavailable,
            detail: Some(
                "feature `jolt` is not enabled; physics runs in downgrade mode".to_string(),
            ),
            simulation_mode: settings.simulation_mode,
            feature_gate,
        };
    }

    if requested_backend.trim().is_empty() || requested_backend.eq_ignore_ascii_case("unconfigured")
    {
        return PhysicsBackendStatus {
            requested_backend,
            active_backend: None,
            state: PhysicsBackendState::Unavailable,
            detail: Some("no physics backend is configured".to_string()),
            simulation_mode: settings.simulation_mode,
            feature_gate,
        };
    }

    PhysicsBackendStatus {
        active_backend: Some(requested_backend.clone()),
        requested_backend,
        state: PhysicsBackendState::Ready,
        detail: None,
        simulation_mode: settings.simulation_mode,
        feature_gate,
    }
}

fn compute_contact_events(sync: &PhysicsWorldSyncState) -> Vec<PhysicsContactEvent> {
    let mut contacts = Vec::new();
    for left_index in 0..sync.colliders.len() {
        for right_index in left_index + 1..sync.colliders.len() {
            let left = &sync.colliders[left_index];
            let right = &sync.colliders[right_index];
            if !colliders_overlap(left, right) {
                continue;
            }

            let left_center = left.transform.translation;
            let right_center = right.transform.translation;
            let mut normal = (right_center - left_center).normalize_or_zero();
            if normal.length_squared() <= Real::EPSILON {
                normal = Vec3::Y;
            }
            let point = (left_center + right_center) * 0.5;
            contacts.push(PhysicsContactEvent {
                world: sync.world,
                entity: left.entity,
                other_entity: right.entity,
                point: point.to_array(),
                normal: normal.to_array(),
            });
        }
    }
    contacts
}

fn colliders_overlap(left: &PhysicsColliderSyncState, right: &PhysicsColliderSyncState) -> bool {
    let (left_min, left_max) = collider_aabb(left);
    let (right_min, right_max) = collider_aabb(right);
    left_min.x <= right_max.x
        && left_max.x >= right_min.x
        && left_min.y <= right_max.y
        && left_max.y >= right_min.y
        && left_min.z <= right_max.z
        && left_max.z >= right_min.z
}

fn collider_aabb(collider: &PhysicsColliderSyncState) -> (Vec3, Vec3) {
    let center = collider.transform.translation;
    let scale = collider.transform.scale.abs();
    let half_extents = match collider.shape {
        PhysicsColliderShape::Box { half_extents } => Vec3::from_array(half_extents) * scale,
        PhysicsColliderShape::Sphere { radius } => Vec3::splat(radius * scale.max_element()),
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => Vec3::new(
            radius * scale.x.abs(),
            (radius + half_height) * scale.y.abs(),
            radius * scale.z.abs(),
        ),
    };
    (center - half_extents, center + half_extents)
}

fn ray_cast_collider(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    collider: &PhysicsColliderSyncState,
) -> Option<PhysicsRayCastHit> {
    match collider.shape {
        PhysicsColliderShape::Box { .. } | PhysicsColliderShape::Capsule { .. } => {
            let (min, max) = collider_aabb(collider);
            ray_cast_aabb(origin, direction, max_distance, collider.entity, min, max)
        }
        PhysicsColliderShape::Sphere { radius } => {
            let scale = collider.transform.scale.max_element().abs();
            ray_cast_sphere(
                origin,
                direction,
                max_distance,
                collider.entity,
                collider.transform.translation,
                radius * scale,
            )
        }
    }
}

fn ray_cast_aabb(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    min: Vec3,
    max: Vec3,
) -> Option<PhysicsRayCastHit> {
    let mut t_min = 0.0;
    let mut t_max = max_distance;
    let mut normal = Vec3::ZERO;

    for axis in 0..3 {
        let origin_axis = origin[axis];
        let direction_axis = direction[axis];
        if direction_axis.abs() <= Real::EPSILON {
            if origin_axis < min[axis] || origin_axis > max[axis] {
                return None;
            }
            continue;
        }

        let inv_dir = 1.0 / direction_axis;
        let mut near = (min[axis] - origin_axis) * inv_dir;
        let mut far = (max[axis] - origin_axis) * inv_dir;
        let mut axis_normal = match axis {
            0 => -Vec3::X,
            1 => -Vec3::Y,
            _ => -Vec3::Z,
        };
        if near > far {
            std::mem::swap(&mut near, &mut far);
            axis_normal = -axis_normal;
        }
        if near > t_min {
            t_min = near;
            normal = axis_normal;
        }
        t_max = t_max.min(far);
        if t_min > t_max {
            return None;
        }
    }

    if t_min < 0.0 || t_min > max_distance {
        return None;
    }

    let position = origin + direction * t_min;
    Some(PhysicsRayCastHit {
        entity,
        distance: t_min,
        position: position.to_array(),
        normal: normal.to_array(),
    })
}

fn ray_cast_sphere(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    center: Vec3,
    radius: Real,
) -> Option<PhysicsRayCastHit> {
    let offset = origin - center;
    let a = direction.length_squared();
    let b = 2.0 * offset.dot(direction);
    let c = offset.length_squared() - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let distance = (-b - sqrt_discriminant) / (2.0 * a);
    if !(0.0..=max_distance).contains(&distance) {
        return None;
    }

    let position = origin + direction * distance;
    let normal = (position - center).normalize_or_zero();
    Some(PhysicsRayCastHit {
        entity,
        distance,
        position: position.to_array(),
        normal: normal.to_array(),
    })
}

fn combine_transforms(parent: Transform, local: Transform) -> Transform {
    Transform {
        translation: parent.translation + local.translation,
        rotation: parent.rotation * local.rotation,
        scale: parent.scale * local.scale,
    }
}
