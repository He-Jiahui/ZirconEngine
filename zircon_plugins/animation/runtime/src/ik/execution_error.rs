use std::error::Error;
use std::fmt;

use zircon_runtime::asset::AssetId;
use zircon_runtime::core::framework::animation::AnimationTargetId;

use super::AnimationIkError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationIkExecutionError {
    MissingPose,
    MissingSkeletonBinding,
    MissingSkeletonAsset { skeleton: AssetId },
    MissingCompiledTargets { skeleton: AssetId },
    UnresolvedTarget { target: AnimationTargetId },
    PoseShapeMismatch { expected: usize, actual: usize },
    InvalidSkeletonHierarchy,
    InvalidTwoBoneChain,
    Solver(AnimationIkError),
}

impl fmt::Display for AnimationIkExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPose => write!(formatter, "IK target entity has no evaluated pose"),
            Self::MissingSkeletonBinding => {
                write!(formatter, "IK target entity has no skeleton binding")
            }
            Self::MissingSkeletonAsset { skeleton } => {
                write!(formatter, "IK skeleton asset {skeleton:?} is not loaded")
            }
            Self::MissingCompiledTargets { skeleton } => {
                write!(
                    formatter,
                    "IK skeleton {skeleton:?} has no compiled targets"
                )
            }
            Self::UnresolvedTarget { target } => {
                write!(formatter, "IK target {target} is absent from the skeleton")
            }
            Self::PoseShapeMismatch { expected, actual } => write!(
                formatter,
                "IK pose has {actual} bones but its skeleton has {expected}"
            ),
            Self::InvalidSkeletonHierarchy => {
                write!(formatter, "IK pose has an invalid skeleton hierarchy")
            }
            Self::InvalidTwoBoneChain => {
                write!(
                    formatter,
                    "two-bone IK requires a direct root -> mid -> tip chain"
                )
            }
            Self::Solver(error) => error.fmt(formatter),
        }
    }
}

impl Error for AnimationIkExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Solver(error) => Some(error),
            _ => None,
        }
    }
}
