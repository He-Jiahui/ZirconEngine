use crate::ops::{NnOp, NnOpCode};
use crate::NN_WEIGHT_ALIGNMENT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NnDataType {
    F32 = 0,
    F16 = 1,
}

impl TryFrom<u8> for NnDataType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::F32),
            1 => Ok(Self::F16),
            _ => Err(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NnTensorKind {
    Input = 0,
    Output = 1,
    Intermediate = 2,
    Weight = 3,
}

impl TryFrom<u8> for NnTensorKind {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Input),
            1 => Ok(Self::Output),
            2 => Ok(Self::Intermediate),
            3 => Ok(Self::Weight),
            _ => Err(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnTensorDesc {
    pub dtype: NnDataType,
    pub kind: NnTensorKind,
    pub rank: u8,
    pub shape: [u32; 4],
    pub weight_offset: u64,
}

impl NnTensorDesc {
    pub const fn new(dtype: NnDataType, kind: NnTensorKind, rank: u8, shape: [u32; 4]) -> Self {
        Self {
            dtype,
            kind,
            rank,
            shape,
            weight_offset: 0,
        }
    }

    pub const fn with_weight_offset(mut self, weight_offset: u64) -> Self {
        self.weight_offset = weight_offset;
        self
    }

    pub fn element_count(&self) -> Option<u64> {
        self.shape.iter().try_fold(1_u64, |count, dimension| {
            count.checked_mul(u64::from(*dimension))
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NnModelAsset {
    pub tensors: Vec<NnTensorDesc>,
    pub ops: Vec<NnOp>,
    pub weights: Vec<u8>,
}

impl NnModelAsset {
    pub fn contains_f16_weights(&self) -> bool {
        self.tensors
            .iter()
            .any(|tensor| tensor.kind == NnTensorKind::Weight && tensor.dtype == NnDataType::F16)
    }

    pub fn weight_tensor_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.tensors
            .iter()
            .enumerate()
            .filter_map(|(index, tensor)| (tensor.kind == NnTensorKind::Weight).then_some(index))
    }

    pub fn op_codes(&self) -> impl Iterator<Item = NnOpCode> + '_ {
        self.ops.iter().map(|op| op.code)
    }

    pub(crate) fn requires_weight_alignment(offset: u64) -> bool {
        offset % NN_WEIGHT_ALIGNMENT == 0
    }
}
