use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PoseBlendError {
    ShapeMismatch {
        destination_len: usize,
        source_len: usize,
    },
    InvalidWeight {
        weight: f32,
    },
}

impl fmt::Display for PoseBlendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShapeMismatch {
                destination_len,
                source_len,
            } => write!(
                formatter,
                "cannot blend pose buffers with lengths {destination_len} and {source_len}"
            ),
            Self::InvalidWeight { weight } => {
                write!(
                    formatter,
                    "pose blend weight {weight} must be finite and in [0, 1]"
                )
            }
        }
    }
}

impl Error for PoseBlendError {}
