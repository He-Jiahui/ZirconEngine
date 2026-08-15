use super::NnOpCode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NnGemmAttrs {
    pub alpha: f32,
    pub beta: f32,
    pub transpose_a: bool,
    pub transpose_b: bool,
}

impl Default for NnGemmAttrs {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 0.0,
            transpose_a: false,
            transpose_b: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NnConv2dAttrs {
    pub stride: [u32; 2],
    pub padding: [u32; 4],
    pub dilation: [u32; 2],
    pub groups: u32,
}

impl Default for NnConv2dAttrs {
    fn default() -> Self {
        Self {
            stride: [1, 1],
            padding: [0, 0, 0, 0],
            dilation: [1, 1],
            groups: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NnPool2dAttrs {
    pub kernel: [u32; 2],
    pub stride: [u32; 2],
    pub padding: [u32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub enum NnOpAttrs {
    None,
    Gemm(NnGemmAttrs),
    Conv2d(NnConv2dAttrs),
    Pool2d(NnPool2dAttrs),
    BatchNorm { epsilon: f32 },
    LayerNorm { epsilon: f32 },
    Upsample2d { scale: [u32; 2] },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NnOpAttrsError {
    UnexpectedSize { expected: usize, actual: usize },
    UnexpectedAttrsForOp { code: NnOpCode },
}

impl NnOpAttrs {
    pub(crate) fn encode(&self, code: NnOpCode) -> Result<Vec<u8>, NnOpAttrsError> {
        match (code, self) {
            (NnOpCode::Gemm, Self::Gemm(attrs)) => {
                let mut bytes = Vec::with_capacity(12);
                bytes.extend_from_slice(&attrs.alpha.to_le_bytes());
                bytes.extend_from_slice(&attrs.beta.to_le_bytes());
                bytes.push(u8::from(attrs.transpose_a));
                bytes.push(u8::from(attrs.transpose_b));
                bytes.extend_from_slice(&[0, 0]);
                Ok(bytes)
            }
            (NnOpCode::Conv2d | NnOpCode::DepthwiseConv2d, Self::Conv2d(attrs)) => {
                let mut bytes = Vec::with_capacity(36);
                write_u32s(&mut bytes, &attrs.stride);
                write_u32s(&mut bytes, &attrs.padding);
                write_u32s(&mut bytes, &attrs.dilation);
                bytes.extend_from_slice(&attrs.groups.to_le_bytes());
                Ok(bytes)
            }
            (NnOpCode::MaxPool2d | NnOpCode::AvgPool2d, Self::Pool2d(attrs)) => {
                let mut bytes = Vec::with_capacity(32);
                write_u32s(&mut bytes, &attrs.kernel);
                write_u32s(&mut bytes, &attrs.stride);
                write_u32s(&mut bytes, &attrs.padding);
                Ok(bytes)
            }
            (NnOpCode::BatchNorm, Self::BatchNorm { epsilon })
            | (NnOpCode::LayerNorm, Self::LayerNorm { epsilon }) => {
                Ok(epsilon.to_le_bytes().to_vec())
            }
            (NnOpCode::Upsample2d, Self::Upsample2d { scale }) => {
                let mut bytes = Vec::with_capacity(8);
                write_u32s(&mut bytes, scale);
                Ok(bytes)
            }
            (_, Self::None) if expects_no_attrs(code) => Ok(Vec::new()),
            _ => Err(NnOpAttrsError::UnexpectedAttrsForOp { code }),
        }
    }

    pub(crate) fn decode(code: NnOpCode, bytes: &[u8]) -> Result<Self, NnOpAttrsError> {
        match code {
            NnOpCode::Gemm => {
                expect_size(bytes, 12)?;
                Ok(Self::Gemm(NnGemmAttrs {
                    alpha: read_f32(bytes, 0),
                    beta: read_f32(bytes, 4),
                    transpose_a: bytes[8] != 0,
                    transpose_b: bytes[9] != 0,
                }))
            }
            NnOpCode::Conv2d | NnOpCode::DepthwiseConv2d => {
                expect_size(bytes, 36)?;
                Ok(Self::Conv2d(NnConv2dAttrs {
                    stride: read_u32s(bytes, 0),
                    padding: read_u32s(bytes, 8),
                    dilation: read_u32s(bytes, 24),
                    groups: read_u32(bytes, 32),
                }))
            }
            NnOpCode::MaxPool2d | NnOpCode::AvgPool2d => {
                expect_size(bytes, 32)?;
                Ok(Self::Pool2d(NnPool2dAttrs {
                    kernel: read_u32s(bytes, 0),
                    stride: read_u32s(bytes, 8),
                    padding: read_u32s(bytes, 16),
                }))
            }
            NnOpCode::BatchNorm => decode_epsilon(bytes).map(|epsilon| Self::BatchNorm { epsilon }),
            NnOpCode::LayerNorm => decode_epsilon(bytes).map(|epsilon| Self::LayerNorm { epsilon }),
            NnOpCode::Upsample2d => {
                expect_size(bytes, 8)?;
                Ok(Self::Upsample2d {
                    scale: read_u32s(bytes, 0),
                })
            }
            _ if expects_no_attrs(code) => {
                expect_size(bytes, 0)?;
                Ok(Self::None)
            }
            _ => Err(NnOpAttrsError::UnexpectedAttrsForOp { code }),
        }
    }
}

fn expects_no_attrs(code: NnOpCode) -> bool {
    matches!(
        code,
        NnOpCode::Add
            | NnOpCode::Mul
            | NnOpCode::Sub
            | NnOpCode::Div
            | NnOpCode::Relu
            | NnOpCode::Sigmoid
            | NnOpCode::Tanh
            | NnOpCode::Silu
            | NnOpCode::Concat
            | NnOpCode::Slice
            | NnOpCode::Reshape
    )
}

fn write_u32s<const N: usize>(bytes: &mut Vec<u8>, values: &[u32; N]) {
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn expect_size(bytes: &[u8], expected: usize) -> Result<(), NnOpAttrsError> {
    if bytes.len() == expected {
        Ok(())
    } else {
        Err(NnOpAttrsError::UnexpectedSize {
            expected,
            actual: bytes.len(),
        })
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_f32(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_u32s<const N: usize>(bytes: &[u8], offset: usize) -> [u32; N] {
    std::array::from_fn(|index| read_u32(bytes, offset + index * 4))
}

fn decode_epsilon(bytes: &[u8]) -> Result<f32, NnOpAttrsError> {
    expect_size(bytes, 4)?;
    Ok(read_f32(bytes, 0))
}
