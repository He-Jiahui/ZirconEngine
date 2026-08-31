use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::resource::stable_uuid_from_components;

use super::ReflectFieldIdParseError;

const REFLECT_FIELD_ID_NAMESPACE: &str = "zircon-reflect-field-id";

/// Stable 128-bit identity for one reflected field, independent of its current name and slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ReflectFieldId(Uuid);

impl ReflectFieldId {
    /// Generates the initial ID from codegen-owned stable keys.
    ///
    /// Renames must retain both keys; current field and display names are not identity inputs.
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
