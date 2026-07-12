use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderSyncState, PhysicsContactEvent,
        PhysicsTriggerEvent,
    },
    scene::physics::{PhysicsCcdMode, PhysicsSleepPolicy},
};
use zircon_runtime::core::math::{Real, Transform};

use super::{
    BodyHandle, ConstraintHandle, PhysicsBackendError, PhysicsBackendObjectKind, ShapeHandle,
};

pub use crate::constraint::ConstraintDesc;

#[derive(Clone, Debug, PartialEq)]
pub struct BodyDesc {
    pub world: WorldHandle,
    pub shape: ShapeHandle,
    pub body: PhysicsBodySyncState,
    pub collider: PhysicsColliderSyncState,
}

impl BodyDesc {
    pub fn from_sync(
        world: WorldHandle,
        shape: ShapeHandle,
        body: &PhysicsBodySyncState,
        collider: &PhysicsColliderSyncState,
    ) -> Result<Self, PhysicsBackendError> {
        if body.entity != collider.entity {
            return Err(PhysicsBackendError::InvalidDescriptor {
                kind: PhysicsBackendObjectKind::Body,
                detail: "body and collider must belong to the same entity".to_string(),
            });
        }
        Ok(Self {
            world,
            shape,
            body: body.clone(),
            collider: collider.clone(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BodyCommand {
    SetLinearVelocity {
        body: BodyHandle,
        velocity: [Real; 3],
    },
    SetAngularVelocity {
        body: BodyHandle,
        velocity: [Real; 3],
    },
    ApplyForce {
        body: BodyHandle,
        force: [Real; 3],
    },
    ApplyImpulse {
        body: BodyHandle,
        impulse: [Real; 3],
    },
    Teleport {
        body: BodyHandle,
        transform: Transform,
    },
    SetBodyType {
        body: BodyHandle,
        body_type: PhysicsBodyType,
    },
    SetCcdMode {
        body: BodyHandle,
        mode: PhysicsCcdMode,
    },
    SetSleepPolicy {
        body: BodyHandle,
        policy: PhysicsSleepPolicy,
    },
}

impl BodyCommand {
    pub(crate) fn body(&self) -> BodyHandle {
        match *self {
            Self::SetLinearVelocity { body, .. }
            | Self::SetAngularVelocity { body, .. }
            | Self::ApplyForce { body, .. }
            | Self::ApplyImpulse { body, .. }
            | Self::Teleport { body, .. }
            | Self::SetBodyType { body, .. }
            | Self::SetCcdMode { body, .. }
            | Self::SetSleepPolicy { body, .. } => body,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhysicsEventBuffer {
    pub contacts: Vec<PhysicsContactEvent>,
    pub triggers: Vec<PhysicsTriggerEvent>,
}

impl From<ConstraintHandle> for u64 {
    fn from(handle: ConstraintHandle) -> Self {
        handle.raw()
    }
}
