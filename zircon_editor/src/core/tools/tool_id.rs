use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

pub const MAX_TOOL_ID_BYTES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolIdError {
    Empty,
    TooLong {
        actual_bytes: usize,
        max_bytes: usize,
    },
    InvalidCharacter {
        index: usize,
        character: char,
    },
}

impl fmt::Display for ToolIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("tool id cannot be empty"),
            Self::TooLong {
                actual_bytes,
                max_bytes,
            } => write!(
                formatter,
                "tool id is {actual_bytes} bytes; maximum is {max_bytes} bytes"
            ),
            Self::InvalidCharacter { index, character } => write!(
                formatter,
                "tool id contains invalid character `{character}` at byte {index}"
            ),
        }
    }
}

impl std::error::Error for ToolIdError {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolId(Arc<str>);

impl ToolId {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, ToolIdError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(ToolIdError::Empty);
        }
        if value.len() > MAX_TOOL_ID_BYTES {
            return Err(ToolIdError::TooLong {
                actual_bytes: value.len(),
                max_bytes: MAX_TOOL_ID_BYTES,
            });
        }

        for (index, character) in value.char_indices() {
            let valid = character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_');
            if !valid {
                return Err(ToolIdError::InvalidCharacter { index, character });
            }
        }

        Ok(Self(Arc::from(value)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ToolId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ToolId {
    type Err = ToolIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl TryFrom<String> for ToolId {
    type Error = ToolIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl Serialize for ToolId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ToolId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(de::Error::custom)
    }
}
