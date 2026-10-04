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

/// provider 执行无法完成转换时保留的分类与说明；Runtime 将其作为调用错误向上传递。
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

/// 经统一校验的转换身份；serde 解码也回到 try_new，避免绕开身份格式约束。
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

/// 转换 provider 的非零代次；替换或卸载后 Runtime 用新代次拒绝旧句柄。
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

/// Runtime 表槽位与 provider 代次的组合；查找时同时核对二者以识别失效句柄。
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

/// 转换器接受的源值种类和产出的目标值种类，注册及调用时据此核对类型契约。
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
    let mut segment_index = 0;
    let mut segment_has_content = false;
    let mut first_invalid = None;
    for (byte_index, character) in value.char_indices() {
        if character == '.' {
            if !segment_has_content {
                return Err(UiBindingConversionIdentityError::EmptySegment { segment_index });
            }
            segment_index += 1;
            segment_has_content = false;
            continue;
        }
        segment_has_content = true;
        if first_invalid.is_none()
            && !(character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        {
            first_invalid = Some((byte_index, character));
        }
    }
    if !segment_has_content {
        return Err(UiBindingConversionIdentityError::EmptySegment { segment_index });
    }
    if let Some((byte_index, character)) = first_invalid {
        return Err(UiBindingConversionIdentityError::InvalidCharacter {
            character,
            byte_index,
        });
    }
    Ok(())
}

#[cfg(test)]
#[path = "conversion/tests/validation_performance_tests.rs"]
mod validation_performance_tests;
