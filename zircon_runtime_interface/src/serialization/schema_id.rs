use std::borrow::Cow;
use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

pub const MAX_SCHEMA_ID_BYTES: usize = 128;

/// Stable identity for one versioned payload family.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct SchemaId(Cow<'static, str>);

impl SchemaId {
    pub const fn new(value: &'static str) -> Self {
        match validate_schema_id(value) {
            Ok(()) => Self(Cow::Borrowed(value)),
            Err(_) => panic!("invalid SchemaId literal"),
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<String> for SchemaId {
    type Error = SchemaIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        validate_schema_id(&value)?;
        Ok(Self(Cow::Owned(value)))
    }
}

impl TryFrom<&str> for SchemaId {
    type Error = SchemaIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_schema_id(value)?;
        Ok(Self(Cow::Owned(value.to_string())))
    }
}

impl<'de> Deserialize<'de> for SchemaId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_from(value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaIdError {
    Empty,
    TooLong { max: usize, found: usize },
    MissingNamespace,
    EmptySegment { index: usize },
    InvalidSegmentStart { index: usize, found: char },
    InvalidSegmentEnd { index: usize, found: char },
    InvalidCharacter { index: usize, found: char },
    NonAscii { index: usize },
}

impl fmt::Display for SchemaIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("schema ID cannot be empty"),
            Self::TooLong { max, found } => {
                write!(formatter, "schema ID exceeds {max} bytes (found {found})")
            }
            Self::MissingNamespace => {
                formatter.write_str("schema ID must contain at least two namespace segments")
            }
            Self::EmptySegment { index } => {
                write!(formatter, "schema ID has an empty segment at byte {index}")
            }
            Self::InvalidSegmentStart { index, found } => write!(
                formatter,
                "schema ID segment at byte {index} must start with a lowercase ASCII letter, found {found:?}"
            ),
            Self::InvalidSegmentEnd { index, found } => write!(
                formatter,
                "schema ID segment cannot end with {found:?} at byte {index}"
            ),
            Self::InvalidCharacter { index, found } => write!(
                formatter,
                "schema ID contains invalid character {found:?} at byte {index}"
            ),
            Self::NonAscii { index } => {
                write!(formatter, "schema ID contains non-ASCII data at byte {index}")
            }
        }
    }
}

impl std::error::Error for SchemaIdError {}

const fn validate_schema_id(value: &str) -> Result<(), SchemaIdError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return Err(SchemaIdError::Empty);
    }
    if bytes.len() > MAX_SCHEMA_ID_BYTES {
        return Err(SchemaIdError::TooLong {
            max: MAX_SCHEMA_ID_BYTES,
            found: bytes.len(),
        });
    }

    let mut index = 0;
    let mut segment_start = true;
    let mut segment_ends_with_hyphen = false;
    let mut has_namespace_separator = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if !byte.is_ascii() {
            return Err(SchemaIdError::NonAscii { index });
        }
        if byte == b'.' {
            if segment_start {
                return Err(SchemaIdError::EmptySegment { index });
            }
            if segment_ends_with_hyphen {
                return Err(SchemaIdError::InvalidSegmentEnd {
                    index: index - 1,
                    found: '-',
                });
            }
            has_namespace_separator = true;
            segment_start = true;
            segment_ends_with_hyphen = false;
            index += 1;
            continue;
        }
        if segment_start {
            if byte < b'a' || byte > b'z' {
                return Err(SchemaIdError::InvalidSegmentStart {
                    index,
                    found: byte as char,
                });
            }
            segment_start = false;
            index += 1;
            continue;
        }
        if (byte >= b'a' && byte <= b'z') || (byte >= b'0' && byte <= b'9') {
            segment_ends_with_hyphen = false;
        } else if byte == b'-' {
            segment_ends_with_hyphen = true;
        } else {
            return Err(SchemaIdError::InvalidCharacter {
                index,
                found: byte as char,
            });
        }
        index += 1;
    }

    if segment_start {
        return Err(SchemaIdError::EmptySegment { index: bytes.len() });
    }
    if segment_ends_with_hyphen {
        return Err(SchemaIdError::InvalidSegmentEnd {
            index: bytes.len() - 1,
            found: '-',
        });
    }
    if !has_namespace_separator {
        return Err(SchemaIdError::MissingNamespace);
    }
    Ok(())
}
