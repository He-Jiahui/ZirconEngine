use zircon_runtime::core::framework::animation::compiler::{
    compile_animation_source, AnimationCompileProduct, AnimationCompileSource,
};
use zircon_runtime::core::framework::animation::{
    AnimationGraphAsset, AnimationSequenceAsset, AnimationStateMachineAsset,
};

use super::{AnimationAuthoringDocumentError, AnimationAuthoringDocumentKind};

#[derive(Clone, Debug)]
pub(crate) enum AnimationAuthoringAsset {
    Sequence(AnimationSequenceAsset),
    Graph(AnimationGraphAsset),
    StateMachine(AnimationStateMachineAsset),
}

impl AnimationAuthoringAsset {
    pub(crate) const fn kind(&self) -> AnimationAuthoringDocumentKind {
        match self {
            Self::Sequence(_) => AnimationAuthoringDocumentKind::Sequence,
            Self::Graph(_) => AnimationAuthoringDocumentKind::Graph,
            Self::StateMachine(_) => AnimationAuthoringDocumentKind::StateMachine,
        }
    }

    pub(crate) fn from_bytes(
        kind: AnimationAuthoringDocumentKind,
        bytes: &[u8],
    ) -> Result<Self, zircon_runtime::core::framework::animation::AnimationAssetError> {
        match kind {
            AnimationAuthoringDocumentKind::Sequence => {
                AnimationSequenceAsset::from_bytes(bytes).map(Self::Sequence)
            }
            AnimationAuthoringDocumentKind::Graph => {
                AnimationGraphAsset::from_bytes(bytes).map(Self::Graph)
            }
            AnimationAuthoringDocumentKind::StateMachine => {
                AnimationStateMachineAsset::from_bytes(bytes).map(Self::StateMachine)
            }
        }
    }

    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>, AnimationAuthoringDocumentError> {
        let result = match self {
            Self::Sequence(asset) => asset.to_bytes(),
            Self::Graph(asset) => asset.to_bytes(),
            Self::StateMachine(asset) => asset.to_bytes(),
        };
        result.map_err(|error| AnimationAuthoringDocumentError::Serialization {
            message: error.to_string(),
        })
    }

    pub(crate) fn compile(&self) -> AnimationCompileProduct {
        match self {
            Self::Sequence(asset) => {
                compile_animation_source(AnimationCompileSource::Sequence(asset))
            }
            Self::Graph(asset) => compile_animation_source(AnimationCompileSource::Graph(asset)),
            Self::StateMachine(asset) => {
                compile_animation_source(AnimationCompileSource::StateMachine(asset))
            }
        }
    }

    pub(crate) fn as_sequence(&self) -> Option<&AnimationSequenceAsset> {
        match self {
            Self::Sequence(asset) => Some(asset),
            Self::Graph(_) | Self::StateMachine(_) => None,
        }
    }

    pub(crate) fn as_graph(&self) -> Option<&AnimationGraphAsset> {
        match self {
            Self::Graph(asset) => Some(asset),
            Self::Sequence(_) | Self::StateMachine(_) => None,
        }
    }

    pub(crate) fn as_state_machine(&self) -> Option<&AnimationStateMachineAsset> {
        match self {
            Self::StateMachine(asset) => Some(asset),
            Self::Sequence(_) | Self::Graph(_) => None,
        }
    }
}
