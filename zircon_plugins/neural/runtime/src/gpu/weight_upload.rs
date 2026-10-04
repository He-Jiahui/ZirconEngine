use std::fmt;
use std::sync::Arc;

use crate::{NnModelAsset, NnTensorKind};

// 算子通过 u16 tensor id 寻址，因此权重偏移表最多需要 65,536 个槽。
const MAX_TENSOR_SLOTS: usize = u16::MAX as usize + 1;

/// 已验证模型的共享权重字节和 tensor offset 计划；不执行 GPU 上传。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnWeightUploadPlan {
    /// 在 compute binding 中引用此权重资源时使用的名称。
    pub resource_name: String,
    /// 模型的完整权重 blob，克隆计划时共享底层字节。
    pub bytes: Arc<[u8]>,
    offsets: Arc<[Option<u64>]>,
}

/// 创建权重资源计划时遇到的模型或 tensor id 错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NnWeightUploadPlanError {
    /// 模型结构未通过校验。
    InvalidModel(String),
    /// 某个权重描述符的索引无法表示为 u16 tensor id。
    TensorIndexOverflow,
}

impl fmt::Display for NnWeightUploadPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for NnWeightUploadPlanError {}

impl NnWeightUploadPlan {
    /// 校验模型并记录每个权重 tensor 的 blob 偏移。
    ///
    /// 每个权重描述符都必须能用 u16 tensor id 表示；高索引非权重描述符不写入偏移表。
    pub fn from_model(
        model: &NnModelAsset,
        resource_name: impl Into<String>,
    ) -> Result<Self, NnWeightUploadPlanError> {
        model
            .validate()
            .map_err(|error| NnWeightUploadPlanError::InvalidModel(error.to_string()))?;
        let mut offsets = vec![None; model.tensors.len().min(MAX_TENSOR_SLOTS)];
        for (index, tensor) in model.tensors.iter().enumerate() {
            if tensor.kind == NnTensorKind::Weight {
                let tensor_id = u16::try_from(index)
                    .map_err(|_| NnWeightUploadPlanError::TensorIndexOverflow)?;
                offsets[usize::from(tensor_id)] = Some(tensor.weight_offset);
            }
        }
        Ok(Self {
            resource_name: resource_name.into(),
            bytes: Arc::from(model.weights.as_slice()),
            offsets: Arc::from(offsets),
        })
    }

    /// 返回权重 tensor 在共享 blob 中的起始字节偏移；非权重 tensor 返回 None。
    pub fn offset_for_tensor(&self, tensor: u16) -> Option<u64> {
        self.offsets.get(usize::from(tensor)).copied().flatten()
    }
}

#[cfg(test)]
#[path = "weight_upload/tests/performance_tests.rs"]
mod performance_tests;
