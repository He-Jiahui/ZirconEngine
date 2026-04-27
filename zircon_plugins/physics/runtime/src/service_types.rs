use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::physics::{
    PhysicsBackendState, PhysicsBackendStatus, PhysicsBodySyncState, PhysicsBodyType,
    PhysicsColliderShape, PhysicsColliderSyncState, PhysicsContactEvent, PhysicsJointSyncState,
    PhysicsJointType, PhysicsMaterialMetadata, PhysicsMaterialSyncState, PhysicsRayCastHit,
    PhysicsRayCastQuery, PhysicsSettings, PhysicsSimulationMode, PhysicsWorldStepPlan,
    PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};
use zircon_runtime::core::{CoreError, CoreHandle};
use zircon_runtime::scene::components::{ColliderShape, JointKind, RigidBodyType};
use zircon_runtime::scene::world::World;

use super::PhysicsInterface;
use query_contact::{collider_matches_query, compute_contact_events, ray_cast_collider};

mod query_contact;

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
        let delta_seconds = if delta_seconds.is_finite() {
            delta_seconds.max(0.0)
        } else {
            0.0
        };
        *accumulator += delta_seconds;

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

pub fn integrate_builtin_physics_steps(world: &mut World, plan: PhysicsWorldStepPlan) {
    const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

    if plan.steps == 0 || !plan.step_seconds.is_finite() || plan.step_seconds <= 0.0 {
        return;
    }

    let entities = world.nodes().iter().map(|node| node.id).collect::<Vec<_>>();
    for _ in 0..plan.steps {
        for entity in &entities {
            let Some(mut rigid_body) = world.rigid_body(*entity).cloned() else {
                continue;
            };
            if rigid_body.body_type == RigidBodyType::Static {
                continue;
            }
            if !rigid_body_step_input_is_finite(&rigid_body) {
                continue;
            }
            let Some(mut transform) = world.find_node(*entity).map(|node| node.transform) else {
                continue;
            };

            let mut velocity = match rigid_body.body_type {
                RigidBodyType::Dynamic => {
                    let damping = (1.0 - rigid_body.linear_damping.max(0.0) * plan.step_seconds)
                        .clamp(0.0, 1.0);
                    (rigid_body.linear_velocity
                        + GRAVITY * rigid_body.gravity_scale * plan.step_seconds)
                        * damping
                }
                RigidBodyType::Kinematic => rigid_body.linear_velocity,
                RigidBodyType::Static => unreachable!(),
            };
            for axis in 0..3 {
                if rigid_body.lock_translation[axis] {
                    velocity[axis] = 0.0;
                } else {
                    transform.translation[axis] += velocity[axis] * plan.step_seconds;
                }
            }
            rigid_body.linear_velocity = velocity;

            let mut angular_velocity = match rigid_body.body_type {
                RigidBodyType::Dynamic => {
                    rigid_body.angular_velocity
                        * (1.0 - rigid_body.angular_damping.max(0.0) * plan.step_seconds)
                            .clamp(0.0, 1.0)
                }
                RigidBodyType::Kinematic => rigid_body.angular_velocity,
                RigidBodyType::Static => unreachable!(),
            };
            for axis in 0..3 {
                if rigid_body.lock_rotation[axis] {
                    angular_velocity[axis] = 0.0;
                }
            }
            let rotation_step = angular_velocity * plan.step_seconds;
            if rotation_step.length_squared() > Real::EPSILON {
                transform.rotation =
                    (Quat::from_scaled_axis(rotation_step) * transform.rotation).normalize();
            }
            rigid_body.angular_velocity = angular_velocity;

            let _ = world.update_transform(*entity, transform);
            let _ = world.set_rigid_body(*entity, Some(rigid_body));
        }
    }
}

fn rigid_body_step_input_is_finite(
    rigid_body: &zircon_runtime::scene::components::RigidBodyComponent,
) -> bool {
    vec3_is_finite(rigid_body.linear_velocity)
        && vec3_is_finite(rigid_body.angular_velocity)
        && rigid_body.linear_damping.is_finite()
        && rigid_body.angular_damping.is_finite()
        && rigid_body.gravity_scale.is_finite()
}

fn vec3_is_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

fn array3_is_finite(value: [Real; 3]) -> bool {
    value[0].is_finite() && value[1].is_finite() && value[2].is_finite()
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

impl zircon_runtime::core::framework::physics::PhysicsManager for DefaultPhysicsManager {
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
        let settings = self.settings();
        let contacts = compute_contact_events(&sync, &settings);
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
        if !query.max_distance.is_finite()
            || !array3_is_finite(query.origin)
            || !array3_is_finite(query.direction)
        {
            return None;
        }
        let direction = Vec3::from_array(query.direction).normalize_or_zero();
        if direction.length_squared() <= Real::EPSILON || query.max_distance <= 0.0 {
            return None;
        }

        self.synchronized_world(query.world)?
            .colliders
            .iter()
            .filter(|collider| collider_matches_query(query, collider))
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

fn combine_transforms(parent: Transform, local: Transform) -> Transform {
    Transform {
        translation: parent.translation + parent.rotation * (parent.scale * local.translation),
        rotation: parent.rotation * local.rotation,
        scale: parent.scale * local.scale,
    }
}
