//! 定义可供 CPU 解释器和 GPU 规划器读取的模型描述及其拥有数据。

use crate::ops::{NnOp, NnOpCode};
use crate::NN_WEIGHT_ALIGNMENT;

/// 模型张量的元素类型；ZRNN v1 将 F32、F16 编码为 0、1，序列化能力不代表后端执行能力。
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

/// 张量描述符；`rank` 记录语义维数，`shape` 始终保存四个轴。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnTensorDesc {
    pub dtype: NnDataType,
    pub kind: NnTensorKind,
    /// 有效维数；模型验证要求为 1 到 4。
    pub rank: u8,
    /// 验证要求四轴均非零；ONNX 导入器将原维度右对齐并前置补 1 是导入约定。
    pub shape: [u32; 4],
    /// 仅 Weight 描述符使用的 `weights` 字节偏移；非权重描述符必须为 0。
    /// `NnModelAsset::validate` 检查对齐和范围，但允许多个权重范围重叠。
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

    /// 返回四个 shape 分量乘积；乘法溢出时返回 `None`。
    pub fn element_count(&self) -> Option<u64> {
        self.shape.iter().try_fold(1_u64, |count, dimension| {
            count.checked_mul(u64::from(*dimension))
        })
    }
}

/// 拥有模型张量表、按记录顺序排列的算子和权重字节 blob。结构验证不保证后端支持每个算子。
#[derive(Clone, Debug, PartialEq)]
pub struct NnModelAsset {
    /// 张量描述符表；算子以 u16 下标引用此表。
    pub tensors: Vec<NnTensorDesc>,
    /// 该序列参与拓扑验证，并由 CPU 解释器和 GPU 规划器按序读取。
    pub ops: Vec<NnOp>,
    pub weights: Vec<u8>,
}

impl NnModelAsset {
    pub fn contains_f16_weights(&self) -> bool {
        self.tensors
            .iter()
            .any(|tensor| tensor.kind == NnTensorKind::Weight && tensor.dtype == NnDataType::F16)
    }

    /// 返回 Weight 描述符的表内 `usize` 索引，不是算子中存储的 u16 ID。
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
