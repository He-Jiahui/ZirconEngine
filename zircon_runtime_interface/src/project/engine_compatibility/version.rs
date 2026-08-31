use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::ProjectEngineVersionParseError;

/// Canonical semantic version of the running editor engine selected for preflight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectEngineVersion(semver::Version);

impl ProjectEngineVersion {
    pub fn parse(value: impl Into<String>) -> Result<Self, ProjectEngineVersionParseError> {
        let value = value.into();
        semver::Version::parse(&value)
            .map(Self)
            .map_err(|source| ProjectEngineVersionParseError::Invalid { value, source })
    }

    pub(crate) fn as_semver(&self) -> &semver::Version {
        &self.0
    }
}

impl Display for ProjectEngineVersion {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Serialize for ProjectEngineVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ProjectEngineVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
