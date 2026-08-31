use serde::{Deserialize, Serialize};

use super::ZrRuntimeIdentityFormatError;

/// Endianness is explicit because a C ABI table cannot safely infer it from a manifest version.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZrRuntimeEndianV1 {
    Little,
    Big,
}

/// Target data model that must match before an internal runtime library can be loaded.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZrRuntimeTargetModelV1 {
    pub architecture: String,
    pub operating_system: String,
    pub pointer_width: u8,
    pub endian: ZrRuntimeEndianV1,
}

impl ZrRuntimeTargetModelV1 {
    pub fn new(
        architecture: impl Into<String>,
        operating_system: impl Into<String>,
        pointer_width: u8,
        endian: ZrRuntimeEndianV1,
    ) -> Result<Self, ZrRuntimeIdentityFormatError> {
        let target = Self {
            architecture: architecture.into(),
            operating_system: operating_system.into(),
            pointer_width,
            endian,
        };
        target.validate()?;
        Ok(target)
    }

    pub fn current() -> Self {
        let endian = if cfg!(target_endian = "little") {
            ZrRuntimeEndianV1::Little
        } else {
            ZrRuntimeEndianV1::Big
        };
        Self {
            architecture: std::env::consts::ARCH.to_owned(),
            operating_system: std::env::consts::OS.to_owned(),
            pointer_width: usize::BITS as u8,
            endian,
        }
    }

    pub fn validate(&self) -> Result<(), ZrRuntimeIdentityFormatError> {
        if self.architecture.is_empty() || self.operating_system.is_empty() {
            return Err(ZrRuntimeIdentityFormatError::TargetNameMissing);
        }
        if !matches!(self.pointer_width, 32 | 64) {
            return Err(ZrRuntimeIdentityFormatError::PointerWidth {
                pointer_width: self.pointer_width,
            });
        }
        Ok(())
    }
}
