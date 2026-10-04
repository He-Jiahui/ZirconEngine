use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::resource::stable_uuid_from_components;

use super::ReflectFieldIdParseError;

const REFLECT_FIELD_ID_NAMESPACE: &str = "zircon-reflect-field-id";

/// Stable 128-bit identity for one reflected field, independent of its current name and slot.
/// 字段的稳定 128 位身份；目录按此身份定位当期槽位，名称只用于显示与旧数据导入。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ReflectFieldId(Uuid);

impl ReflectFieldId {
    /// Generates the initial ID from codegen-owned stable keys.
    ///
    /// Renames must retain both keys; current field and display names are not identity inputs.
    /// 从类型与字段的身份键生成确定性 ID；改名时须保留这两个键以维持旧引用。
    pub fn from_stable_keys(owner_key: &str, field_key: &str) -> Self {
        Self(stable_uuid_from_components(
            REFLECT_FIELD_ID_NAMESPACE,
            &[owner_key, field_key],
        ))
    }

    pub fn try_from_uuid(value: Uuid) -> Result<Self, ReflectFieldIdParseError> {
        if value.is_nil() {
            return Err(ReflectFieldIdParseError::Nil);
        }
        Ok(Self(value))
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Display for ReflectFieldId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl FromStr for ReflectFieldId {
    type Err = ReflectFieldIdParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value)
            .map_err(|source| ReflectFieldIdParseError::InvalidUuid { source })
            .and_then(Self::try_from_uuid)
    }
}

impl<'de> Deserialize<'de> for ReflectFieldId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from_uuid(Uuid::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
