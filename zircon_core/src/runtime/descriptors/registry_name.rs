use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::lifecycle::ServiceKind;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegistryName(String);

impl RegistryName {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let mut parts = value.split('.');
        let first = parts.next();
        let second = parts.next();
        let third = parts.next();
        if first.is_none()
            || second.is_none()
            || third.is_none()
            || first.is_some_and(str::is_empty)
            || second.is_some_and(str::is_empty)
            || third.is_some_and(str::is_empty)
        {
            return Err(CoreError::InvalidRegistryName(value));
        }
        Ok(Self(value))
    }

    pub fn from_parts(module: &str, kind: ServiceKind, service: &str) -> Self {
        Self(format!("{module}.{}.{}", kind.as_str(), service))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RegistryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
