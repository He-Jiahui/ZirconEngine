use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderSyncState, PhysicsContactEvent,
        PhysicsJointType, PhysicsTriggerEvent,
    },
    scene::physics::PhysicsJointConstraintMetadata,
};
use zircon_runtime::core::math::{Real, Transform};

use super::{
    BodyHandle, ConstraintHandle, PhysicsBackendError, PhysicsBackendObjectKind, ShapeHandle,
};

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

#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintDesc {
    pub joint_type: PhysicsJointType,
    pub body_a: BodyHandle,
    pub body_b: Option<BodyHandle>,
    pub anchor_a: Transform,
    pub anchor_b: Transform,
    pub metadata: PhysicsJointConstraintMetadata,
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
}

impl BodyCommand {
    pub(crate) fn body(&self) -> BodyHandle {
        match *self {
            Self::SetLinearVelocity { body, .. }
            | Self::SetAngularVelocity { body, .. }
            | Self::ApplyForce { body, .. }
            | Self::ApplyImpulse { body, .. }
            | Self::Teleport { body, .. }
            | Self::SetBodyType { body, .. } => body,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhysicsEventBuffer {
    pub contacts: Vec<PhysicsContactEvent>,
    pub triggers: Vec<PhysicsTriggerEvent>,
}

impl ConstraintDesc {
    pub fn handles(&self) -> impl Iterator<Item = BodyHandle> {
        std::iter::once(self.body_a).chain(self.body_b)
    }
}

impl From<ConstraintHandle> for u64 {
    fn from(handle: ConstraintHandle) -> Self {
        handle.raw()
    }
}
