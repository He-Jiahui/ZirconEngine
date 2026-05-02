use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UiComponentApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl UiComponentApiVersion {
    pub const DEFAULT: Self = Self {
        major: 1,
        minor: 0,
        patch: 0,
    };

    pub const fn is_compatible_with(self, required: Self) -> bool {
        self.major == required.major && self.minor >= required.minor
    }
}

impl Default for UiComponentApiVersion {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for UiComponentApiVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for UiComponentApiVersion {
    type Err = UiComponentApiVersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split('.');
        let major = parse_part(parts.next(), value)?;
        let minor = parse_part(parts.next(), value)?;
        let patch = parse_part(parts.next(), value)?;
        if parts.next().is_some() {
            return Err(UiComponentApiVersionParseError {
                value: value.to_string(),
            });
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl Serialize for UiComponentApiVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for UiComponentApiVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("invalid ui component api version {value}")]
pub struct UiComponentApiVersionParseError {
    value: String,
}

fn parse_part(part: Option<&str>, original: &str) -> Result<u32, UiComponentApiVersionParseError> {
    part.filter(|value| !value.is_empty())
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| UiComponentApiVersionParseError {
            value: original.to_string(),
        })
}
