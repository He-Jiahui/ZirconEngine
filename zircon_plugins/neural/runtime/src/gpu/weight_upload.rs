use std::collections::BTreeMap;
use std::fmt;

use crate::{NnModelAsset, NnTensorKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnWeightUploadPlan {
    pub resource_name: String,
    pub bytes: Vec<u8>,
    offsets: BTreeMap<u16, u64>,
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
        let mut offsets = BTreeMap::new();
        for (index, tensor) in model.tensors.iter().enumerate() {
            if tensor.kind == NnTensorKind::Weight {
                offsets.insert(
                    u16::try_from(index)
                        .map_err(|_| NnWeightUploadPlanError::TensorIndexOverflow)?,
                    tensor.weight_offset,
                );
            }
        }
        Ok(Self {
            resource_name: resource_name.into(),
            bytes: model.weights.clone(),
            offsets,
        })
    }

    pub fn offset_for_tensor(&self, tensor: u16) -> Option<u64> {
        self.offsets.get(&tensor).copied()
    }
}
