#[cfg(feature = "backend-jolt")]
use zircon_runtime::core::framework::physics::PhysicsColliderSyncState;
use zircon_runtime::core::framework::physics::{PhysicsBodySyncState, PhysicsBodyType};

#[cfg(feature = "backend-jolt")]
use crate::backend::{BodyCommand, BodyHandle};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct BodySyncChange {
    teleport: bool,
    linear_velocity: bool,
    angular_velocity: bool,
    body_type: bool,
    recreate: bool,
}

impl BodySyncChange {
    #[cfg(test)]
    pub(super) fn requires_commands(self) -> bool {
        !self.recreate
            && (self.teleport || self.linear_velocity || self.angular_velocity || self.body_type)
    }

    pub(super) fn requires_recreation(self) -> bool {
        self.recreate
    }

    #[cfg(feature = "backend-jolt")]
    pub(super) fn commands(
        self,
        handle: BodyHandle,
        body: &PhysicsBodySyncState,
    ) -> Vec<BodyCommand> {
        if self.recreate {
            return Vec::new();
        }
        let mut commands = Vec::with_capacity(4);
        if self.teleport {
            commands.push(BodyCommand::Teleport {
                body: handle,
                transform: body.transform,
            });
        }
        if self.linear_velocity {
            commands.push(BodyCommand::SetLinearVelocity {
                body: handle,
                velocity: body.linear_velocity,
            });
        }
        if self.angular_velocity {
            commands.push(BodyCommand::SetAngularVelocity {
                body: handle,
                velocity: body.angular_velocity,
            });
        }
        if self.body_type {
            commands.push(BodyCommand::SetBodyType {
                body: handle,
                body_type: body.body_type,
            });
        }
        commands
    }
}

pub(super) fn detect_body_change(
    previous: &PhysicsBodySyncState,
    current: &PhysicsBodySyncState,
) -> BodySyncChange {
    let crosses_static_boundary = previous.body_type != current.body_type
        && (previous.body_type == PhysicsBodyType::Static
            || current.body_type == PhysicsBodyType::Static);
    BodySyncChange {
        teleport: previous.transform != current.transform,
        linear_velocity: previous.linear_velocity != current.linear_velocity,
        angular_velocity: previous.angular_velocity != current.angular_velocity,
        body_type: previous.body_type != current.body_type,
        recreate: previous.entity != current.entity
            || crosses_static_boundary
            || previous.mass != current.mass
            || previous.linear_damping != current.linear_damping
            || previous.angular_damping != current.angular_damping
            || previous.gravity_scale != current.gravity_scale
            || previous.can_sleep != current.can_sleep
            || previous.lock_translation != current.lock_translation
            || previous.lock_rotation != current.lock_rotation,
    }
}

#[cfg(feature = "backend-jolt")]
pub(super) fn collider_requires_recreation(
    previous: &PhysicsColliderSyncState,
    current: &PhysicsColliderSyncState,
) -> bool {
    previous.entity != current.entity
        || previous.shape != current.shape
        || previous.sensor != current.sensor
        || previous.layer != current.layer
        || previous.collision_group != current.collision_group
        || previous.collision_mask != current.collision_mask
        || previous.material != current.material
        || previous.material_override != current.material_override
}
