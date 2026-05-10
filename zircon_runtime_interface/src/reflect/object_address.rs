use serde::{Deserialize, Serialize};

use super::{type_path::validate_non_empty_type_path, ReflectError};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ReflectObjectAddress {
    Component { entity: u64, type_path: String },
    Resource { type_path: String },
}

impl ReflectObjectAddress {
    pub fn component(entity: u64, type_path: impl Into<String>) -> Result<Self, ReflectError> {
        let type_path = type_path.into();
        validate_non_empty_type_path(&type_path)?;
        Ok(Self::Component { entity, type_path })
    }

    pub fn resource(type_path: impl Into<String>) -> Result<Self, ReflectError> {
        let type_path = type_path.into();
        validate_non_empty_type_path(&type_path)?;
        Ok(Self::Resource { type_path })
    }

    pub fn type_path(&self) -> &str {
        match self {
            Self::Component { type_path, .. } | Self::Resource { type_path } => type_path,
        }
    }
}
