use std::fmt;
use std::sync::Arc;

use crate::{NnModelAsset, NnTensorKind};

const MAX_TENSOR_SLOTS: usize = u16::MAX as usize + 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnWeightUploadPlan {
    pub resource_name: String,
    pub bytes: Arc<[u8]>,
    offsets: Arc<[Option<u64>]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NnWeightUploadPlanError {
    InvalidModel(String),
    TensorIndexOverflow,
}

impl fmt::Display for NnWeightUploadPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for NnWeightUploadPlanError {}

impl NnWeightUploadPlan {
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

    pub fn offset_for_tensor(&self, tensor: u16) -> Option<u64> {
        self.offsets.get(usize::from(tensor)).copied().flatten()
    }
}

#[cfg(test)]
mod performance_tests;
