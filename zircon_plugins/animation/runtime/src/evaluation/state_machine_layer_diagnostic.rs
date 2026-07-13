use std::error::Error;
use std::fmt;

use zircon_runtime::scene::EntityId;

use super::{PoseBlendError, PoseBufferError};

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationStateMachineLayerDiagnostic {
    pub entity: EntityId,
    pub layer: String,
    pub error: AnimationStateMachineLayerError,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationStateMachineLayerError {
    BoneCountMismatch {
        base: usize,
        layer: usize,
    },
    BoneNameMismatch {
        index: usize,
        base: String,
        layer: String,
    },
    BasePose(PoseBufferError),
    LayerPose(PoseBufferError),
    Blend(PoseBlendError),
}

impl fmt::Display for AnimationStateMachineLayerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BoneCountMismatch { base, layer } => {
                write!(
                    formatter,
                    "base pose has {base} bones but layer pose has {layer}"
                )
            }
            Self::BoneNameMismatch { index, base, layer } => write!(
                formatter,
                "base bone `{base}` and layer bone `{layer}` differ at row {index}"
            ),
            Self::BasePose(error) => write!(formatter, "invalid base layer pose: {error}"),
            Self::LayerPose(error) => write!(formatter, "invalid source layer pose: {error}"),
            Self::Blend(error) => error.fmt(formatter),
        }
    }
}

impl Error for AnimationStateMachineLayerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BasePose(error) | Self::LayerPose(error) => Some(error),
            Self::Blend(error) => Some(error),
            Self::BoneCountMismatch { .. } | Self::BoneNameMismatch { .. } => None,
        }
    }
}
