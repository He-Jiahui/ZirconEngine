use std::fmt;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use thiserror::Error;

use crate::ui::component::UiValueKind;

pub const UI_BINDING_CONVERSION_ID_MAX_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingConversionProviderErrorCode {
    InvalidValue,
    OutOfRange,
    Unsupported,
}

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("binding conversion provider failed with {code:?}: {detail}")]
pub struct UiBindingConversionProviderError {
    pub code: UiBindingConversionProviderErrorCode,
    pub detail: String,
}

impl UiBindingConversionProviderError {
    pub fn new(code: UiBindingConversionProviderErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiBindingConversionIdentityError {
    #[error("binding conversion identity cannot be empty")]
    Empty,
    #[error(
        "binding conversion identity uses {actual_bytes} bytes, exceeding the {maximum_bytes}-byte limit"
    )]
    TooLong {
        actual_bytes: usize,
        maximum_bytes: usize,
    },
    #[error("binding conversion identity contains an empty segment at index {segment_index}")]
    EmptySegment { segment_index: usize },
    #[error(
        "binding conversion identity contains invalid character `{character}` at byte {byte_index}"
    )]
    InvalidCharacter { character: char, byte_index: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct UiBindingConversionId(String);

impl UiBindingConversionId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, UiBindingConversionIdentityError> {
        let value = value.into();
        validate_conversion_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UiBindingConversionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for UiBindingConversionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("binding conversion provider generation must be non-zero")]
pub struct UiBindingConversionProviderGenerationError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct UiBindingConversionProviderGeneration(u64);

impl UiBindingConversionProviderGeneration {
    pub const fn try_new(value: u64) -> Result<Self, UiBindingConversionProviderGenerationError> {
        if value == 0 {
            Err(UiBindingConversionProviderGenerationError)
        } else {
            Ok(Self(value))
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for UiBindingConversionProviderGeneration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::try_new(value).map_err(D::Error::custom)
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UiBindingConversionSlot(u32);

impl UiBindingConversionSlot {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiBindingConversionHandle {
    slot: UiBindingConversionSlot,
    provider_generation: UiBindingConversionProviderGeneration,
}

impl UiBindingConversionHandle {
    pub const fn new(
        slot: UiBindingConversionSlot,
        provider_generation: UiBindingConversionProviderGeneration,
    ) -> Self {
        Self {
            slot,
            provider_generation,
        }
    }

    pub const fn slot(self) -> UiBindingConversionSlot {
        self.slot
    }

    pub const fn provider_generation(self) -> UiBindingConversionProviderGeneration {
        self.provider_generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingConversionSignature {
    pub source: UiValueKind,
    pub destination: UiValueKind,
}

impl UiBindingConversionSignature {
    pub const fn new(source: UiValueKind, destination: UiValueKind) -> Self {
        Self {
            source,
            destination,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingConversionDescriptor {
    pub id: UiBindingConversionId,
    pub provider_generation: UiBindingConversionProviderGeneration,
    pub signature: UiBindingConversionSignature,
}

impl UiBindingConversionDescriptor {
    pub const fn new(
        id: UiBindingConversionId,
        provider_generation: UiBindingConversionProviderGeneration,
        signature: UiBindingConversionSignature,
    ) -> Self {
        Self {
            id,
            provider_generation,
            signature,
        }
    }
}

fn validate_conversion_id(value: &str) -> Result<(), UiBindingConversionIdentityError> {
    if value.is_empty() {
        return Err(UiBindingConversionIdentityError::Empty);
    }
    if value.len() > UI_BINDING_CONVERSION_ID_MAX_BYTES {
        return Err(UiBindingConversionIdentityError::TooLong {
            actual_bytes: value.len(),
            maximum_bytes: UI_BINDING_CONVERSION_ID_MAX_BYTES,
        });
    }
    for (segment_index, segment) in value.split('.').enumerate() {
        if segment.is_empty() {
            return Err(UiBindingConversionIdentityError::EmptySegment { segment_index });
        }
    }
    for (byte_index, character) in value.char_indices() {
        if !(character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')) {
            return Err(UiBindingConversionIdentityError::InvalidCharacter {
                character,
                byte_index,
            });
        }
    }
    Ok(())
}
