use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::error::CoreError;
use crate::core::lifecycle::ServiceKind;

#[derive(Clone, Debug)]
pub struct RegistryName {
    // `value` remains the equality, hash, borrow, and serde authority; the
    // cached fields only avoid re-parsing the validated registry string.
    value: String,
    module_end: usize,
    service_start: usize,
    kind: ServiceKind,
}

impl RegistryName {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let Some((module_end, kind_end)) = registry_separator_offsets(&value) else {
            return Err(CoreError::InvalidRegistryName(value));
        };
        if !is_canonical_segment(&value[..module_end]) {
            return Err(CoreError::InvalidRegistryName(value));
        }

        let kind_start = module_end + 1;
        let kind_segment = &value.as_bytes()[kind_start..kind_end];
        let Some(kind) = ServiceKind::from_registry_segment_bytes(kind_segment) else {
            return Err(CoreError::InvalidRegistryName(value));
        };

        let service_start = kind_end + 1;
        let service = &value[service_start..];
        if !is_canonical_segment(service) {
            return Err(CoreError::InvalidRegistryName(value));
        }

        Ok(Self {
            value,
            module_end,
            service_start,
            kind,
        })
    }

    pub fn from_parts(module: &str, kind: ServiceKind, service: &str) -> Self {
        assert!(
            is_canonical_dot_free_segment(module),
            "registry name module segments must be non-empty, trim-clean, and dot-free"
        );
        assert!(
            is_canonical_dot_free_segment(service),
            "registry name service segments must be non-empty, trim-clean, and dot-free"
        );
        let kind_segment = kind.as_str();
        let service_start = module.len() + kind_segment.len() + 2;
        let mut value =
            String::with_capacity(module.len() + kind_segment.len() + service.len() + 2);
        value.push_str(module);
        value.push('.');
        value.push_str(kind_segment);
        value.push('.');
        value.push_str(service);
        Self {
            value,
            module_end: module.len(),
            service_start,
            kind,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn module_name(&self) -> &str {
        &self.value[..self.module_end]
    }

    pub fn service_kind(&self) -> ServiceKind {
        self.kind
    }

    pub fn service_name(&self) -> &str {
        &self.value[self.service_start..]
    }
}

impl PartialEq for RegistryName {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for RegistryName {}

impl Hash for RegistryName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Borrow<str> for RegistryName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Serialize for RegistryName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RegistryName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for RegistryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

fn is_canonical_segment(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first.is_whitespace() {
        return false;
    }
    !chars.next_back().unwrap_or(first).is_whitespace()
}

fn is_canonical_dot_free_segment(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first.is_whitespace() || first == '.' {
        return false;
    }

    let mut last = first;
    for ch in chars {
        if ch == '.' {
            return false;
        }
        last = ch;
    }

    !last.is_whitespace()
}

fn registry_separator_offsets(value: &str) -> Option<(usize, usize)> {
    let mut first_separator = None;
    let mut second_separator = None;
    for (index, byte) in value.bytes().enumerate() {
        if byte != b'.' {
            continue;
        }
        if first_separator.is_none() {
            first_separator = Some(index);
            continue;
        }
        if second_separator.is_none() {
            second_separator = Some(index);
            continue;
        }
        return None;
    }
    Some((first_separator?, second_separator?))
}
