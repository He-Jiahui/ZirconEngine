use std::fmt;

use crate::{NnDataType, NnTensorDesc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnTensorLayout {
    pub dimensions: [u32; 4],
    pub element_count: u64,
    pub element_size_bytes: u64,
    pub storage_size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NnTensorLayoutError {
    ZeroDimension,
    ElementCountOverflow,
    StorageSizeOverflow,
}

impl fmt::Display for NnTensorLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for NnTensorLayoutError {}

impl NnTensorLayout {
    pub fn from_descriptor(descriptor: &NnTensorDesc) -> Result<Self, NnTensorLayoutError> {
        if descriptor.shape.contains(&0) {
            return Err(NnTensorLayoutError::ZeroDimension);
        }
        let element_count = descriptor
            .element_count()
            .ok_or(NnTensorLayoutError::ElementCountOverflow)?;
        let element_size_bytes = match descriptor.dtype {
            NnDataType::F32 => 4,
            NnDataType::F16 => 2,
        };
        let storage_size_bytes = element_count
            .checked_mul(element_size_bytes)
            .ok_or(NnTensorLayoutError::StorageSizeOverflow)?;
        Ok(Self {
            dimensions: descriptor.shape,
            element_count,
            element_size_bytes,
            storage_size_bytes,
        })
    }
}
