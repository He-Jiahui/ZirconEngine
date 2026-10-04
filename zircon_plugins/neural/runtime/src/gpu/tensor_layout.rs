use std::fmt;

use crate::{NnDataType, NnTensorDesc};

/// 记录描述符形状及其紧密连续载荷字节数，不包含 GPU 缓冲区对齐或填充。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnTensorLayout {
    /// 原样保留描述符的四个形状分量；本计算不会按 rank 截断。
    pub dimensions: [u32; 4],
    /// 四个 shape 分量的乘积；rank 不会使任何槽位被忽略。
    pub element_count: u64,
    /// 单个元素的载荷字节数：F32 为 4，F16 为 2。
    pub element_size_bytes: u64,
    /// 紧密连续载荷的总字节数；不计 GPU 缓冲区对齐或填充。
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
    /// 从单个描述符计算紧密连续载荷布局。
    ///
    /// 只检查四个 shape 分量非零，以及元素数和载荷字节数乘法不溢出。
    /// 不检查 rank 或模型图拓扑，也不替代 `NnModelAsset::validate`。
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
