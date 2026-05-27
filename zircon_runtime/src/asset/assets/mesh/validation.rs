use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MeshValidationError {
    MissingPositionAttribute,
    InvalidPositionAttributeFormat,
    InvalidAttributeFormat {
        attribute: String,
        expected: &'static str,
    },
    AttributeLengthMismatch {
        attribute: String,
        expected: usize,
        actual: usize,
    },
    MorphTargetAttributeLengthMismatch {
        target_index: usize,
        attribute: String,
        expected: usize,
        actual: usize,
    },
    IndexOutOfRange {
        max_index: u32,
        vertex_count: usize,
    },
}

impl fmt::Display for MeshValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPositionAttribute => {
                write!(formatter, "mesh is missing required position attribute")
            }
            Self::InvalidPositionAttributeFormat => {
                write!(
                    formatter,
                    "mesh position attribute must use float32x3 values"
                )
            }
            Self::InvalidAttributeFormat {
                attribute,
                expected,
            } => write!(
                formatter,
                "mesh attribute `{attribute}` must use {expected} values"
            ),
            Self::AttributeLengthMismatch {
                attribute,
                expected,
                actual,
            } => write!(
                formatter,
                "mesh attribute `{attribute}` has {actual} values but expected {expected}"
            ),
            Self::MorphTargetAttributeLengthMismatch {
                target_index,
                attribute,
                expected,
                actual,
            } => write!(
                formatter,
                "mesh morph target {target_index} attribute `{attribute}` has {actual} values but expected {expected}"
            ),
            Self::IndexOutOfRange {
                max_index,
                vertex_count,
            } => write!(
                formatter,
                "mesh index buffer references vertex {max_index} but only {vertex_count} vertices are present"
            ),
        }
    }
}

impl std::error::Error for MeshValidationError {}
