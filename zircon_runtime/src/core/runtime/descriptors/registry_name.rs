use std::borrow::Borrow;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::core::error::CoreError;
use crate::core::lifecycle::ServiceKind;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegistryName(String);

impl RegistryName {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let mut parts = value.split('.');
        let Some(_module) = parts
            .next()
            .filter(|part| !part.is_empty() && part.trim() == *part)
        else {
            return Err(CoreError::InvalidRegistryName(value));
        };
        let Some(_kind) = parts.next().and_then(ServiceKind::from_registry_segment) else {
            return Err(CoreError::InvalidRegistryName(value));
        };
        let Some(_service) = parts
            .next()
            .filter(|part| !part.is_empty() && part.trim() == *part)
        else {
            return Err(CoreError::InvalidRegistryName(value));
        };
        if parts.next().is_some() {
            return Err(CoreError::InvalidRegistryName(value));
        }
        Ok(Self(value))
    }

    pub fn from_parts(module: &str, kind: ServiceKind, service: &str) -> Self {
        let kind = kind.as_str();
        let mut value = String::with_capacity(module.len() + kind.len() + service.len() + 2);
        value.push_str(module);
        value.push('.');
        value.push_str(kind);
        value.push('.');
        value.push_str(service);
        Self::new(value).expect("registry names built from parts must be Module.Kind.Service")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn module_name(&self) -> &str {
        self.0
            .split_once('.')
            .map(|(module, _rest)| module)
            .expect("validated registry names always contain a module segment")
    }

    pub fn service_kind(&self) -> ServiceKind {
        self.0
            .split('.')
            .nth(1)
            .and_then(ServiceKind::from_registry_segment)
            .expect("validated registry names always contain a canonical service kind segment")
    }

    pub fn service_name(&self) -> &str {
        self.0
            .rsplit_once('.')
            .map(|(_prefix, service)| service)
            .expect("validated registry names always contain a service segment")
    }
}

impl Borrow<str> for RegistryName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for RegistryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
