use std::error::Error;
use std::fmt;

use crate::{AnimationClipCompileError, PoseBufferError};
use zircon_runtime::core::resource::ResourceId;

use super::{AnimationChannelDataRole, AnimationTransformChannel};

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationEvaluationError {
    Compile(AnimationClipCompileError),
    NonFiniteSkeletonTransform {
        bone_index: usize,
        channel: AnimationTransformChannel,
    },
    ZeroLengthSkeletonRotation {
        bone_index: usize,
    },
    InvalidChannelValueType {
        track_index: usize,
        channel: AnimationTransformChannel,
        key_index: usize,
        role: AnimationChannelDataRole,
    },
    NonFiniteChannelTime {
        track_index: usize,
        channel: AnimationTransformChannel,
        key_index: usize,
    },
    NonIncreasingChannelTime {
        track_index: usize,
        channel: AnimationTransformChannel,
        previous_key_index: usize,
        key_index: usize,
    },
    NonFiniteChannelValue {
        track_index: usize,
        channel: AnimationTransformChannel,
        key_index: usize,
        role: AnimationChannelDataRole,
    },
    ZeroLengthChannelRotation {
        track_index: usize,
        key_index: usize,
    },
    PoseBuffer(PoseBufferError),
    MissingPreparedSkeleton {
        skeleton: ResourceId,
    },
    MissingPreparedClip {
        skeleton: ResourceId,
        clip: ResourceId,
    },
    MissingCompiledTrackTarget {
        track_index: usize,
    },
    PoseShapeMismatch {
        index: usize,
        len: usize,
    },
    ValidatedChannelTypeMismatch {
        track_index: usize,
        channel: AnimationTransformChannel,
    },
}

impl fmt::Display for AnimationEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compile(error) => error.fmt(formatter),
            Self::NonFiniteSkeletonTransform {
                bone_index,
                channel,
            } => write!(
                formatter,
                "animation skeleton bone {bone_index} has non-finite {channel}"
            ),
            Self::ZeroLengthSkeletonRotation { bone_index } => write!(
                formatter,
                "animation skeleton bone {bone_index} has a zero-length rotation"
            ),
            Self::InvalidChannelValueType {
                track_index,
                channel,
                key_index,
                role,
            } => write!(
                formatter,
                "animation clip track {track_index} {channel} key {key_index} has an invalid {role} type"
            ),
            Self::NonFiniteChannelTime {
                track_index,
                channel,
                key_index,
            } => write!(
                formatter,
                "animation clip track {track_index} {channel} key {key_index} has a non-finite time"
            ),
            Self::NonIncreasingChannelTime {
                track_index,
                channel,
                previous_key_index,
                key_index,
            } => write!(
                formatter,
                "animation clip track {track_index} {channel} key {key_index} is not later than key {previous_key_index}"
            ),
            Self::NonFiniteChannelValue {
                track_index,
                channel,
                key_index,
                role,
            } => write!(
                formatter,
                "animation clip track {track_index} {channel} key {key_index} has a non-finite {role}"
            ),
            Self::ZeroLengthChannelRotation {
                track_index,
                key_index,
            } => write!(
                formatter,
                "animation clip track {track_index} rotation key {key_index} has a zero-length value"
            ),
            Self::PoseBuffer(error) => error.fmt(formatter),
            Self::MissingPreparedSkeleton { skeleton } => write!(
                formatter,
                "animation evaluator did not retain prepared skeleton {skeleton}"
            ),
            Self::MissingPreparedClip { skeleton, clip } => write!(
                formatter,
                "animation evaluator did not retain prepared clip {clip} for skeleton {skeleton}"
            ),
            Self::MissingCompiledTrackTarget { track_index } => write!(
                formatter,
                "compiled animation track {track_index} has no target row"
            ),
            Self::PoseShapeMismatch { index, len } => write!(
                formatter,
                "animation pose row {index} is outside prepared pose length {len}"
            ),
            Self::ValidatedChannelTypeMismatch {
                track_index,
                channel,
            } => write!(
                formatter,
                "validated animation track {track_index} returned the wrong {channel} value type"
            ),
        }
    }
}

impl Error for AnimationEvaluationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compile(error) => Some(error),
            Self::PoseBuffer(error) => Some(error),
            _ => None,
        }
    }
}

impl From<AnimationClipCompileError> for AnimationEvaluationError {
    fn from(error: AnimationClipCompileError) -> Self {
        Self::Compile(error)
    }
}

impl From<PoseBufferError> for AnimationEvaluationError {
    fn from(error: PoseBufferError) -> Self {
        Self::PoseBuffer(error)
    }
}
