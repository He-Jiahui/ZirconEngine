//! 整理 provider 创建描述符、可复制刚体命令和事件缓冲区；ConstraintDesc 从物理约束模块重导出。

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

/// 将场景刚体、collider、world 与当前 provider 创建的 shape handle 组合成一次创建输入。
#[derive(Clone, Debug, PartialEq)]
pub struct BodyDesc {
    pub world: WorldHandle,
    pub shape: ShapeHandle,
    pub body: PhysicsBodySyncState,
    pub collider: PhysicsColliderSyncState,
}

// from_sync 只检查 body/collider 的 entity 一致性；有限值、shape 及 provider 约束在 create_body 路径检查。
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

/// 传给 PhysicsBackend 的刚体操作值；manager 队列会检查有限数值，直接 trait 调用仍须由后端合同覆盖。
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

/// 接收 drain_events 追加内容的接触与触发事件集合。
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
