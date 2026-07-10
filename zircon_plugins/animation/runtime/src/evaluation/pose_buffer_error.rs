use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PoseBufferError {
    IndexOutOfBounds { index: usize, len: usize },
    NonFiniteTransform { index: usize },
    ZeroLengthRotation { index: usize },
    InvalidWeight { index: usize, weight: f32 },
}

impl fmt::Display for PoseBufferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexOutOfBounds { index, len } => {
                write!(formatter, "pose row {index} is outside buffer length {len}")
            }
            Self::NonFiniteTransform { index } => {
                write!(
                    formatter,
                    "pose row {index} contains a non-finite transform"
                )
            }
            Self::ZeroLengthRotation { index } => {
                write!(
                    formatter,
                    "pose row {index} contains a zero-length rotation"
                )
            }
            Self::InvalidWeight { index, weight } => write!(
                formatter,
                "pose row {index} weight {weight} must be finite and in [0, 1]"
            ),
        }
    }
}

impl Error for PoseBufferError {}
