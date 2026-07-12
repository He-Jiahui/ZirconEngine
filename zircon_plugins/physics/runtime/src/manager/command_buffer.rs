use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use zircon_runtime::core::framework::physics::PhysicsBodyType;
#[cfg(feature = "backend-jolt")]
use zircon_runtime::core::framework::physics::PhysicsWorldSyncState;
use zircon_runtime::core::framework::scene::physics::{PhysicsCcdMode, PhysicsSleepPolicy};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::{Real, Transform, Vec3};
use zircon_runtime::scene::components::RigidBodyType;
use zircon_runtime::scene::world::World;

use crate::backend::resolve_body_mass;
#[cfg(feature = "backend-jolt")]
use crate::backend::{BodyCommand, BodyHandle};

use super::poison_recovery::recover_lock;
use super::validation::{array3_is_finite, transform_is_finite};
use super::world_sync::collider_shape_to_physics;
use super::DefaultPhysicsManager;

const MAX_PENDING_BODY_COMMANDS_PER_WORLD: usize = 4_096;

pub(super) type PhysicsBodyCommandQueues = HashMap<WorldHandle, Vec<PhysicsBodyCommand>>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PhysicsBodyCommand {
    SetLinearVelocity {
        world: WorldHandle,
        entity: EntityId,
        velocity: [Real; 3],
    },
    SetAngularVelocity {
        world: WorldHandle,
        entity: EntityId,
        velocity: [Real; 3],
    },
    ApplyForce {
        world: WorldHandle,
        entity: EntityId,
        force: [Real; 3],
    },
    ApplyImpulse {
        world: WorldHandle,
        entity: EntityId,
        impulse: [Real; 3],
    },
    Teleport {
        world: WorldHandle,
        entity: EntityId,
        transform: Transform,
    },
    SetBodyType {
        world: WorldHandle,
        entity: EntityId,
        body_type: PhysicsBodyType,
    },
    SetCcdMode {
        world: WorldHandle,
        entity: EntityId,
        mode: PhysicsCcdMode,
    },
    SetSleepPolicy {
        world: WorldHandle,
        entity: EntityId,
        policy: PhysicsSleepPolicy,
    },
}

impl PhysicsBodyCommand {
    pub const fn world(self) -> WorldHandle {
        match self {
            Self::SetLinearVelocity { world, .. }
            | Self::SetAngularVelocity { world, .. }
            | Self::ApplyForce { world, .. }
            | Self::ApplyImpulse { world, .. }
            | Self::Teleport { world, .. }
            | Self::SetBodyType { world, .. }
            | Self::SetCcdMode { world, .. }
            | Self::SetSleepPolicy { world, .. } => world,
        }
    }

    pub const fn entity(self) -> EntityId {
        match self {
            Self::SetLinearVelocity { entity, .. }
            | Self::SetAngularVelocity { entity, .. }
            | Self::ApplyForce { entity, .. }
            | Self::ApplyImpulse { entity, .. }
            | Self::Teleport { entity, .. }
            | Self::SetBodyType { entity, .. }
            | Self::SetCcdMode { entity, .. }
            | Self::SetSleepPolicy { entity, .. } => entity,
        }
    }

    #[cfg(feature = "backend-jolt")]
    pub(super) fn to_backend(self, body: BodyHandle) -> BodyCommand {
        match self {
            Self::SetLinearVelocity { velocity, .. } => {
                BodyCommand::SetLinearVelocity { body, velocity }
            }
            Self::SetAngularVelocity { velocity, .. } => {
                BodyCommand::SetAngularVelocity { body, velocity }
            }
            Self::ApplyForce { force, .. } => BodyCommand::ApplyForce { body, force },
            Self::ApplyImpulse { impulse, .. } => BodyCommand::ApplyImpulse { body, impulse },
            Self::Teleport { transform, .. } => BodyCommand::Teleport { body, transform },
            Self::SetBodyType { body_type, .. } => BodyCommand::SetBodyType { body, body_type },
            Self::SetCcdMode { mode, .. } => BodyCommand::SetCcdMode { body, mode },
            Self::SetSleepPolicy { policy, .. } => BodyCommand::SetSleepPolicy { body, policy },
        }
    }

    fn is_valid(self) -> bool {
        match self {
            Self::SetLinearVelocity { velocity, .. }
            | Self::SetAngularVelocity { velocity, .. } => array3_is_finite(velocity),
            Self::ApplyForce { force, .. } => array3_is_finite(force),
            Self::ApplyImpulse { impulse, .. } => array3_is_finite(impulse),
            Self::Teleport { transform, .. } => transform_is_finite(transform),
            Self::SetBodyType { .. } | Self::SetCcdMode { .. } | Self::SetSleepPolicy { .. } => {
                true
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PhysicsCommandError {
    NonFiniteInput {
        world: WorldHandle,
        entity: EntityId,
    },
    QueueFull {
        world: WorldHandle,
        capacity: usize,
    },
}

impl fmt::Display for PhysicsCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteInput { world, entity } => write!(
                formatter,
                "physics body command for entity {entity} in {world:?} contains non-finite input"
            ),
            Self::QueueFull { world, capacity } => write!(
                formatter,
                "physics body command queue for {world:?} reached its {capacity}-command capacity"
            ),
        }
    }
}

impl Error for PhysicsCommandError {}

impl DefaultPhysicsManager {
    pub fn queue_body_command(
        &self,
        command: PhysicsBodyCommand,
    ) -> Result<(), PhysicsCommandError> {
        if !command.is_valid() {
            return Err(PhysicsCommandError::NonFiniteInput {
                world: command.world(),
                entity: command.entity(),
            });
        }
        let world = command.world();
        let mut queues = recover_lock(&self.body_commands);
        let queue = queues.entry(world).or_default();
        if queue.len() >= MAX_PENDING_BODY_COMMANDS_PER_WORLD {
            return Err(PhysicsCommandError::QueueFull {
                world,
                capacity: MAX_PENDING_BODY_COMMANDS_PER_WORLD,
            });
        }
        queue.push(command);
        Ok(())
    }

    pub(super) fn drain_body_commands(&self, world: WorldHandle) -> Vec<PhysicsBodyCommand> {
        recover_lock(&self.body_commands)
            .remove(&world)
            .unwrap_or_default()
    }

    pub(super) fn clear_body_commands(&self, world: WorldHandle) {
        recover_lock(&self.body_commands).remove(&world);
    }
}

#[cfg(feature = "backend-jolt")]
pub(super) fn apply_commands_to_sync(
    sync: &mut PhysicsWorldSyncState,
    commands: &[PhysicsBodyCommand],
) {
    let body_indices = sync
        .bodies
        .iter()
        .enumerate()
        .map(|(index, body)| (body.entity, index))
        .collect::<HashMap<_, _>>();
    let collider_indices = sync
        .colliders
        .iter()
        .enumerate()
        .map(|(index, collider)| (collider.entity, index))
        .collect::<HashMap<_, _>>();

    for command in commands {
        let Some(body_index) = body_indices.get(&command.entity()).copied() else {
            continue;
        };
        let body = &mut sync.bodies[body_index];
        match *command {
            PhysicsBodyCommand::SetLinearVelocity { velocity, .. } => {
                body.linear_velocity = velocity;
            }
            PhysicsBodyCommand::SetAngularVelocity { velocity, .. } => {
                body.angular_velocity = velocity;
            }
            PhysicsBodyCommand::Teleport { transform, .. } => {
                body.transform = transform;
                if let Some(collider_index) = collider_indices.get(&body.entity).copied() {
                    sync.colliders[collider_index].transform = transform;
                }
            }
            PhysicsBodyCommand::SetBodyType { body_type, .. } => {
                body.body_type = body_type;
            }
            PhysicsBodyCommand::SetCcdMode { mode, .. } => {
                body.ccd_mode = mode;
            }
            PhysicsBodyCommand::SetSleepPolicy { policy, .. } => {
                body.sleep_policy = policy;
            }
            PhysicsBodyCommand::ApplyForce { .. } | PhysicsBodyCommand::ApplyImpulse { .. } => {}
        }
    }
}

pub(super) fn apply_commands_to_scene(
    world: &mut World,
    commands: &[PhysicsBodyCommand],
    step_seconds: Real,
) {
    for command in commands {
        if let PhysicsBodyCommand::Teleport {
            entity, transform, ..
        } = *command
        {
            let _ = world.update_transform(entity, transform);
            continue;
        }

        let entity = command.entity();
        let Some(mut body) = world.rigid_body(entity).cloned() else {
            continue;
        };
        match *command {
            PhysicsBodyCommand::SetLinearVelocity { velocity, .. } => {
                body.linear_velocity = Vec3::from_array(velocity);
            }
            PhysicsBodyCommand::SetAngularVelocity { velocity, .. } => {
                body.angular_velocity = Vec3::from_array(velocity);
            }
            PhysicsBodyCommand::ApplyForce { force, .. } => {
                if let Some(mass) = resolved_scene_body_mass(world, entity, &body) {
                    body.mass = mass;
                    body.linear_velocity += Vec3::from_array(force) * (step_seconds / mass);
                }
            }
            PhysicsBodyCommand::ApplyImpulse { impulse, .. } => {
                if let Some(mass) = resolved_scene_body_mass(world, entity, &body) {
                    body.mass = mass;
                    body.linear_velocity += Vec3::from_array(impulse) / mass;
                }
            }
            PhysicsBodyCommand::SetBodyType { body_type, .. } => {
                body.body_type = match body_type {
                    PhysicsBodyType::Static => RigidBodyType::Static,
                    PhysicsBodyType::Dynamic => RigidBodyType::Dynamic,
                    PhysicsBodyType::Kinematic => RigidBodyType::Kinematic,
                };
            }
            PhysicsBodyCommand::SetCcdMode { mode, .. } => {
                body.ccd_mode = mode;
            }
            PhysicsBodyCommand::SetSleepPolicy { policy, .. } => {
                body.sleep_policy = policy;
            }
            PhysicsBodyCommand::Teleport { .. } => {}
        }
        let _ = world.set_rigid_body(entity, Some(body));
    }
}

fn resolved_scene_body_mass(
    world: &World,
    entity: EntityId,
    body: &zircon_runtime::scene::components::RigidBodyComponent,
) -> Option<Real> {
    let collider = world.collider(entity)?;
    resolve_body_mass(
        &collider_shape_to_physics(&collider.shape),
        body.mass,
        body.mass_properties,
    )
    .ok()
    .map(|resolved| resolved.mass)
}
