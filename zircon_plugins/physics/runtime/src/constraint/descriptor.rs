use zircon_runtime::core::framework::physics::PhysicsJointSyncState;
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::{Transform, Vec3};

use crate::backend::{BodyHandle, PhysicsBackendError, PhysicsBackendObjectKind};

use super::JointParams;

#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintDesc {
    pub joint_type: zircon_runtime::core::framework::physics::PhysicsJointType,
    pub body_a: BodyHandle,
    pub body_b: Option<BodyHandle>,
    pub anchor_a: Transform,
    pub anchor_b: Transform,
    pub params: JointParams,
    pub collide_connected: bool,
}

impl ConstraintDesc {
    pub fn from_joint_sync(
        joint: &PhysicsJointSyncState,
        mut resolve_body: impl FnMut(EntityId) -> Option<BodyHandle>,
    ) -> Result<Self, PhysicsBackendError> {
        let body_a = resolve_body(joint.entity).ok_or_else(|| unresolved_body(joint.entity))?;
        let body_b = joint
            .connected_entity
            .map(|entity| resolve_body(entity).ok_or_else(|| unresolved_body(entity)))
            .transpose()?;
        let anchor = Transform::from_translation(Vec3::from_array(joint.anchor));
        Ok(Self {
            joint_type: joint.kind,
            body_a,
            body_b,
            anchor_a: anchor,
            anchor_b: anchor,
            params: JointParams::from_joint_sync(joint),
            collide_connected: joint.collide_connected,
        })
    }

    pub fn handles(&self) -> impl Iterator<Item = BodyHandle> {
        std::iter::once(self.body_a).chain(self.body_b)
    }
}

fn unresolved_body(entity: EntityId) -> PhysicsBackendError {
    PhysicsBackendError::InvalidDescriptor {
        kind: PhysicsBackendObjectKind::Constraint,
        detail: format!("joint entity {entity} does not resolve to a synchronized physics body"),
    }
}
