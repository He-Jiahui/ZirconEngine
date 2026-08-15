use std::fmt;

use crate::ops::NnOpAttrsError;
use crate::{NnDataType, NnModelAsset, NnTensorKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NnModelValidationError {
    MissingTensors,
    InvalidRank { tensor: usize, rank: u8 },
    ZeroDimension { tensor: usize },
    TensorElementCountOverflow { tensor: usize },
    NonWeightOffset { tensor: usize, offset: u64 },
    UnalignedWeightOffset { tensor: usize, offset: u64 },
    WeightOffsetOutsideBlob { tensor: usize, offset: u64 },
    WeightRangeOutsideBlob { tensor: usize, end: u64 },
    MixedWeightPrecision,
    MissingOpInputs { op: usize },
    MissingOpOutputs { op: usize },
    InvalidTensorReference { op: usize, tensor: u16 },
    InvalidOpAttrs { op: usize, error: NnOpAttrsError },
}

impl fmt::Display for NnModelValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for NnModelValidationError {}

impl NnModelAsset {
    pub fn validate(&self) -> Result<(), NnModelValidationError> {
        if self.tensors.is_empty() {
            return Err(NnModelValidationError::MissingTensors);
        }

        let f16_weights = self.contains_f16_weights();
        for (tensor_index, tensor) in self.tensors.iter().enumerate() {
            if !(1..=4).contains(&tensor.rank) {
                return Err(NnModelValidationError::InvalidRank {
                    tensor: tensor_index,
                    rank: tensor.rank,
                });
            }
            if tensor.shape.contains(&0) {
                return Err(NnModelValidationError::ZeroDimension {
                    tensor: tensor_index,
                });
            }
            if tensor.element_count().is_none() {
                return Err(NnModelValidationError::TensorElementCountOverflow {
                    tensor: tensor_index,
                });
            }

            match tensor.kind {
                NnTensorKind::Weight => {
                    if !NnModelAsset::requires_weight_alignment(tensor.weight_offset) {
                        return Err(NnModelValidationError::UnalignedWeightOffset {
                            tensor: tensor_index,
                            offset: tensor.weight_offset,
                        });
                    }
                    if tensor.weight_offset > self.weights.len() as u64 {
                        return Err(NnModelValidationError::WeightOffsetOutsideBlob {
                            tensor: tensor_index,
                            offset: tensor.weight_offset,
                        });
                    }
                    let element_size = match tensor.dtype {
                        NnDataType::F32 => 4_u64,
                        NnDataType::F16 => 2_u64,
                    };
                    let required_bytes = tensor
                        .element_count()
                        .and_then(|count| count.checked_mul(element_size))
                        .ok_or(NnModelValidationError::TensorElementCountOverflow {
                            tensor: tensor_index,
                        })?;
                    let end = tensor.weight_offset.checked_add(required_bytes).ok_or(
                        NnModelValidationError::WeightRangeOutsideBlob {
                            tensor: tensor_index,
                            end: u64::MAX,
                        },
                    )?;
                    if end > self.weights.len() as u64 {
                        return Err(NnModelValidationError::WeightRangeOutsideBlob {
                            tensor: tensor_index,
                            end,
                        });
                    }
                    if (tensor.dtype == NnDataType::F16) != f16_weights {
                        return Err(NnModelValidationError::MixedWeightPrecision);
                    }
                }
                _ if tensor.weight_offset != 0 => {
                    return Err(NnModelValidationError::NonWeightOffset {
                        tensor: tensor_index,
                        offset: tensor.weight_offset,
                    });
                }
                _ => {}
            }
        }

        for (op_index, op) in self.ops.iter().enumerate() {
            if op.inputs.is_empty() {
                return Err(NnModelValidationError::MissingOpInputs { op: op_index });
            }
            if op.outputs.is_empty() {
                return Err(NnModelValidationError::MissingOpOutputs { op: op_index });
            }
            for tensor in op.inputs.iter().chain(&op.outputs) {
                if usize::from(*tensor) >= self.tensors.len() {
                    return Err(NnModelValidationError::InvalidTensorReference {
                        op: op_index,
                        tensor: *tensor,
                    });
                }
            }
            op.attrs
                .encode(op.code)
                .map_err(|error| NnModelValidationError::InvalidOpAttrs {
                    op: op_index,
                    error,
                })?;
        }

        Ok(())
    }
}
