use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use super::ProjectGuidParseError;

/// Stable project-owned GUID. Missing values must be handled by explicit migration policy.
/// 项目持久化身份；缺失值只能经显式迁移处理，不能在读取清单时隐式补造。
/// 该类型拒绝 nil UUID，但不校验 UUID 版本/变体，也不判断跨项目重复。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ProjectGuid(Uuid);

impl ProjectGuid {
    /// 通过 `Uuid::new_v4` 为新项目生成随机标识；不会查询既有项目值。
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 将已有 UUID 包装为项目身份，仅拒绝 nil 值。
    ///
    /// # Errors
    /// UUID 为 nil 时返回 [`ProjectGuidParseError::Nil`]；版本、变体和重复策略由调用方负责。
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

    /// 解析 `Uuid::parse_str` 支持的文本表示，并在解析后应用 nil 检查。
    ///
    /// # Errors
    /// 文本不是受支持的 UUID 表示，或解析结果为 nil 时返回错误。
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

impl Default for ProjectGuid {
    fn default() -> Self {
        Self::new()
    }
}
