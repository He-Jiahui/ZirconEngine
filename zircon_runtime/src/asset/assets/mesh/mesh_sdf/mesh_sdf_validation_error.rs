use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MeshSdfValidationError {
    UnsupportedSchema { expected: u32, actual: u32 },
    InvalidCookSettings,
    InvalidDimensions,
    VoxelCountOverflow,
    VoxelCountMismatch { expected: u64, actual: u64 },
    InvalidLocalBounds,
    InvalidVoxelSize,
    InvalidDistanceRange,
    MissingSourceHash,
    SourceHashMismatch,
    PayloadSizeOverflow,
    PayloadBudgetExceeded { actual: u64, budget: u64 },
}

impl fmt::Display for MeshSdfValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema { expected, actual } => {
                write!(
                    formatter,
                    "mesh SDF schema {actual} does not match {expected}"
                )
            }
            Self::InvalidCookSettings => formatter.write_str("mesh SDF cook settings are invalid"),
            Self::InvalidDimensions => formatter.write_str("mesh SDF dimensions are invalid"),
            Self::VoxelCountOverflow => formatter.write_str("mesh SDF voxel count overflowed"),
            Self::VoxelCountMismatch { expected, actual } => write!(
                formatter,
                "mesh SDF voxel count mismatch: expected {expected}, got {actual}"
            ),
            Self::InvalidLocalBounds => formatter.write_str("mesh SDF local bounds are invalid"),
            Self::InvalidVoxelSize => formatter.write_str("mesh SDF voxel size is invalid"),
            Self::InvalidDistanceRange => formatter.write_str("mesh SDF distance range is invalid"),
            Self::MissingSourceHash => formatter.write_str("mesh SDF source hash is missing"),
            Self::SourceHashMismatch => {
                formatter.write_str("mesh SDF source hash does not match mesh geometry")
            }
            Self::PayloadSizeOverflow => formatter.write_str("mesh SDF payload size overflowed"),
            Self::PayloadBudgetExceeded { actual, budget } => write!(
                formatter,
                "mesh SDF payload uses {actual} bytes, exceeding budget {budget}"
            ),
        }
    }
}

impl std::error::Error for MeshSdfValidationError {}
