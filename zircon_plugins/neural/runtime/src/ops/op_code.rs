#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum NnOpCode {
    Gemm = 1,
    Conv2d = 2,
    DepthwiseConv2d = 3,
    Add = 16,
    Mul = 17,
    Sub = 18,
    Div = 19,
    Relu = 32,
    Sigmoid = 33,
    Tanh = 34,
    Silu = 35,
    BatchNorm = 48,
    LayerNorm = 49,
    MaxPool2d = 64,
    AvgPool2d = 65,
    Upsample2d = 66,
    Concat = 80,
    Slice = 81,
    Reshape = 82,
}

impl NnOpCode {
    pub const fn is_view(self) -> bool {
        matches!(self, Self::Concat | Self::Slice | Self::Reshape)
    }
}

impl TryFrom<u16> for NnOpCode {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let code = match value {
            1 => Self::Gemm,
            2 => Self::Conv2d,
            3 => Self::DepthwiseConv2d,
            16 => Self::Add,
            17 => Self::Mul,
            18 => Self::Sub,
            19 => Self::Div,
            32 => Self::Relu,
            33 => Self::Sigmoid,
            34 => Self::Tanh,
            35 => Self::Silu,
            48 => Self::BatchNorm,
            49 => Self::LayerNorm,
            64 => Self::MaxPool2d,
            65 => Self::AvgPool2d,
            66 => Self::Upsample2d,
            80 => Self::Concat,
            81 => Self::Slice,
            82 => Self::Reshape,
            _ => return Err(value),
        };
        Ok(code)
    }
}
