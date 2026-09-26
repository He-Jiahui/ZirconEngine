use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use super::{
    ProjectActivationOperationIdError, ProjectActivationOperationSequence, ProjectLaunchInstanceId,
};

/// 跨启动请求与恢复记录传递的操作身份，由来源进程、单调序号和 nonce 共同确定。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct ProjectActivationOperationId {
    origin_instance: ProjectLaunchInstanceId,
    sequence: ProjectActivationOperationSequence,
    nonce: Uuid,
}

impl ProjectActivationOperationId {
    /// 构造或恢复传输身份时拒绝 nil nonce；来源和序号已由各自类型验证。
    pub fn try_from_parts(
        origin_instance: ProjectLaunchInstanceId,
        sequence: ProjectActivationOperationSequence,
        nonce: Uuid,
    ) -> Result<Self, ProjectActivationOperationIdError> {
        if nonce.is_nil() {
            return Err(ProjectActivationOperationIdError::NilNonce);
        }
        Ok(Self {
            origin_instance,
            sequence,
            nonce,
        })
    }

    pub const fn origin_instance(self) -> ProjectLaunchInstanceId {
        self.origin_instance
    }

    pub const fn sequence(self) -> ProjectActivationOperationSequence {
        self.sequence
    }

    pub const fn nonce(self) -> Uuid {
        self.nonce
    }
}

// 传输入口先解析严格字段形状，再走公开构造器复核 nonce 不变量。
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectActivationOperationIdWire {
    origin_instance: ProjectLaunchInstanceId,
    sequence: ProjectActivationOperationSequence,
    nonce: Uuid,
}

impl<'de> Deserialize<'de> for ProjectActivationOperationId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProjectActivationOperationIdWire::deserialize(deserializer)?;
        Self::try_from_parts(wire.origin_instance, wire.sequence, wire.nonce)
            .map_err(serde::de::Error::custom)
    }
}
