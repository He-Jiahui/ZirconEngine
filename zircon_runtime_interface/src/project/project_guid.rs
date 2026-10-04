use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use super::ProjectGuidParseError;

/// Stable project-owned GUID. Missing values must be handled by explicit migration policy.
/// 项目持久化身份；缺失值只能经显式迁移处理，不能在读取清单时隐式补造。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ProjectGuid(Uuid);

impl ProjectGuid {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn try_from_uuid(value: Uuid) -> Result<Self, ProjectGuidParseError> {
        if value.is_nil() {
            return Err(ProjectGuidParseError::Nil);
        }
        Ok(Self(value))
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Display for ProjectGuid {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl FromStr for ProjectGuid {
    type Err = ProjectGuidParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value)
            .map_err(|source| ProjectGuidParseError::InvalidUuid { source })
            .and_then(Self::try_from_uuid)
    }
}

// 持久化入口复用 nil 检查，防止 Serde 直接构造无效项目身份。
impl<'de> Deserialize<'de> for ProjectGuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from_uuid(Uuid::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
