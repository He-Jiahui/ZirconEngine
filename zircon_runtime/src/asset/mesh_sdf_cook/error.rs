use std::fmt;

use crate::asset::MeshSdfValidationError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MeshSdfCookError {
    EmptyGeometry,
    InvalidTriangleIndexCount,
    IndexOutOfRange,
    NonFinitePosition,
    DegenerateGeometry,
    InvalidSettings,
    PayloadBudgetTooSmall,
    VoxelCountOverflow,
    SourceTriangleBudgetExceeded { actual: u64, budget: u64 },
    PrimitiveWorkBudgetExceeded { actual: u64, budget: u64 },
    ImportVoxelBudgetExceeded { actual: u64, budget: u64 },
    ImportPayloadBudgetExceeded { actual: u64, budget: u64 },
    ImportWorkBudgetExceeded { actual: u64, budget: u64 },
    Validation(MeshSdfValidationError),
}

impl fmt::Display for MeshSdfCookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGeometry => formatter.write_str("mesh SDF source geometry is empty"),
            Self::InvalidTriangleIndexCount => {
                formatter.write_str("mesh SDF source indices are not a triangle list")
            }
            Self::IndexOutOfRange => formatter.write_str("mesh SDF source index is out of range"),
            Self::NonFinitePosition => {
                formatter.write_str("mesh SDF source contains a non-finite position")
            }
            Self::DegenerateGeometry => {
                formatter.write_str("mesh SDF source has no non-degenerate triangles")
            }
            Self::InvalidSettings => formatter.write_str("mesh SDF cook settings are invalid"),
            Self::PayloadBudgetTooSmall => {
                formatter.write_str("mesh SDF payload budget cannot hold the minimum volume")
            }
            Self::VoxelCountOverflow => formatter.write_str("mesh SDF voxel count overflowed"),
            Self::SourceTriangleBudgetExceeded { actual, budget } => write!(
                formatter,
                "mesh SDF source uses {actual} triangles, exceeding the {budget}-triangle hard limit"
            ),
            Self::PrimitiveWorkBudgetExceeded { actual, budget } => write!(
                formatter,
                "mesh SDF primitive requires {actual} work units, exceeding the {budget}-unit hard limit"
            ),
            Self::ImportVoxelBudgetExceeded { actual, budget } => write!(
                formatter,
                "mesh SDF import requires {actual} voxels, exceeding the {budget}-voxel cumulative limit"
            ),
            Self::ImportPayloadBudgetExceeded { actual, budget } => write!(
                formatter,
                "mesh SDF import requires {actual} payload bytes, exceeding the {budget}-byte cumulative limit"
            ),
            Self::ImportWorkBudgetExceeded { actual, budget } => write!(
                formatter,
                "mesh SDF import requires {actual} work units, exceeding the {budget}-unit cumulative limit"
            ),
            Self::Validation(error) => write!(formatter, "cooked mesh SDF is invalid: {error}"),
        }
    }
}

impl std::error::Error for MeshSdfCookError {}

impl MeshSdfCookError {
    pub fn is_budget_exceeded(&self) -> bool {
        matches!(
            self,
            Self::SourceTriangleBudgetExceeded { .. }
                | Self::PrimitiveWorkBudgetExceeded { .. }
                | Self::ImportVoxelBudgetExceeded { .. }
                | Self::ImportPayloadBudgetExceeded { .. }
                | Self::ImportWorkBudgetExceeded { .. }
        )
    }
}

impl From<MeshSdfValidationError> for MeshSdfCookError {
    fn from(error: MeshSdfValidationError) -> Self {
        Self::Validation(error)
    }
}
