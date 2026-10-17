use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use super::ProjectLaunchInstanceIdError;

/// Stable process-instance component of a project activation operation identity.
/// 同一启动进程内保持稳定的激活来源；新进程必须产生新身份，避免序号重启后混淆操作。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ProjectLaunchInstanceId(Uuid);

impl ProjectLaunchInstanceId {
    /// 为新的启动进程实例生成 UUID v4 身份。
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 从传输 UUID 恢复启动实例身份；仅拒绝 nil 值，成功时保留给定 UUID。
    pub fn try_from_uuid(value: Uuid) -> Result<Self, ProjectLaunchInstanceIdError> {
        if value.is_nil() {
            return Err(ProjectLaunchInstanceIdError::Nil);
        }
        Ok(Self(value))
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl<'de> Deserialize<'de> for ProjectLaunchInstanceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from_uuid(Uuid::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl Default for ProjectLaunchInstanceId {
    fn default() -> Self {
        Self::new()
    }
}
