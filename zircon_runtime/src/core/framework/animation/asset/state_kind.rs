use serde::{Deserialize, Serialize};

use super::error::AnimationAssetError;
use super::reference::{push_unique_reference, AnimationAssetReferenceBinary};
use crate::core::math::{Real, Vec2};
use crate::core::resource::AssetReference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationBlendSpace1DSampleAsset {
    pub position: Real,
    pub graph: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationBlendSpace1DAsset {
    pub parameter: String,
    pub samples: Vec<AnimationBlendSpace1DSampleAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationBlendSpace2DSampleAsset {
    pub position: Vec2,
    pub graph: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationBlendSpace2DAsset {
    pub parameter: String,
    pub samples: Vec<AnimationBlendSpace2DSampleAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationStateKindAsset {
    Clip { clip: AssetReference },
    BlendSpace1D(AnimationBlendSpace1DAsset),
    BlendSpace2D(AnimationBlendSpace2DAsset),
    SubMachine { state_machine: AssetReference },
    GraphRef { graph: AssetReference },
}

impl AnimationStateKindAsset {
    pub fn graph_reference(&self) -> Option<&AssetReference> {
        match self {
            Self::GraphRef { graph } => Some(graph),
            _ => None,
        }
    }

    pub(crate) fn push_direct_references(&self, references: &mut Vec<AssetReference>) {
        match self {
            Self::Clip { clip } => push_unique_reference(references, clip.clone()),
            Self::BlendSpace1D(blend) => {
                for sample in &blend.samples {
                    push_unique_reference(references, sample.graph.clone());
                }
            }
            Self::BlendSpace2D(blend) => {
                for sample in &blend.samples {
                    push_unique_reference(references, sample.graph.clone());
                }
            }
            Self::SubMachine { state_machine } => {
                push_unique_reference(references, state_machine.clone());
            }
            Self::GraphRef { graph } => push_unique_reference(references, graph.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct AnimationBlendSpace1DSampleBinaryAsset {
    position: Real,
    graph: AnimationAssetReferenceBinary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct AnimationBlendSpace2DSampleBinaryAsset {
    position: Vec2,
    graph: AnimationAssetReferenceBinary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) enum AnimationStateKindBinaryAsset {
    Clip {
        clip: AnimationAssetReferenceBinary,
    },
    BlendSpace1D {
        parameter: String,
        samples: Vec<AnimationBlendSpace1DSampleBinaryAsset>,
    },
    BlendSpace2D {
        parameter: String,
        samples: Vec<AnimationBlendSpace2DSampleBinaryAsset>,
    },
    SubMachine {
        state_machine: AnimationAssetReferenceBinary,
    },
    GraphRef {
        graph: AnimationAssetReferenceBinary,
    },
}

impl From<&AnimationStateKindAsset> for AnimationStateKindBinaryAsset {
    fn from(value: &AnimationStateKindAsset) -> Self {
        match value {
            AnimationStateKindAsset::Clip { clip } => Self::Clip {
                clip: AnimationAssetReferenceBinary::from(clip),
            },
            AnimationStateKindAsset::BlendSpace1D(blend) => Self::BlendSpace1D {
                parameter: blend.parameter.clone(),
                samples: blend
                    .samples
                    .iter()
                    .map(|sample| AnimationBlendSpace1DSampleBinaryAsset {
                        position: sample.position,
                        graph: AnimationAssetReferenceBinary::from(&sample.graph),
                    })
                    .collect(),
            },
            AnimationStateKindAsset::BlendSpace2D(blend) => Self::BlendSpace2D {
                parameter: blend.parameter.clone(),
                samples: blend
                    .samples
                    .iter()
                    .map(|sample| AnimationBlendSpace2DSampleBinaryAsset {
                        position: sample.position,
                        graph: AnimationAssetReferenceBinary::from(&sample.graph),
                    })
                    .collect(),
            },
            AnimationStateKindAsset::SubMachine { state_machine } => Self::SubMachine {
                state_machine: AnimationAssetReferenceBinary::from(state_machine),
            },
            AnimationStateKindAsset::GraphRef { graph } => Self::GraphRef {
                graph: AnimationAssetReferenceBinary::from(graph),
            },
        }
    }
}

impl TryFrom<AnimationStateKindBinaryAsset> for AnimationStateKindAsset {
    type Error = AnimationAssetError;

    fn try_from(value: AnimationStateKindBinaryAsset) -> Result<Self, Self::Error> {
        match value {
            AnimationStateKindBinaryAsset::Clip { clip } => Ok(Self::Clip {
                clip: clip.try_into()?,
            }),
            AnimationStateKindBinaryAsset::BlendSpace1D { parameter, samples } => {
                Ok(Self::BlendSpace1D(AnimationBlendSpace1DAsset {
                    parameter,
                    samples: samples
                        .into_iter()
                        .map(|sample| {
                            Ok(AnimationBlendSpace1DSampleAsset {
                                position: sample.position,
                                graph: sample.graph.try_into()?,
                            })
                        })
                        .collect::<Result<Vec<_>, AnimationAssetError>>()?,
                }))
            }
            AnimationStateKindBinaryAsset::BlendSpace2D { parameter, samples } => {
                Ok(Self::BlendSpace2D(AnimationBlendSpace2DAsset {
                    parameter,
                    samples: samples
                        .into_iter()
                        .map(|sample| {
                            Ok(AnimationBlendSpace2DSampleAsset {
                                position: sample.position,
                                graph: sample.graph.try_into()?,
                            })
                        })
                        .collect::<Result<Vec<_>, AnimationAssetError>>()?,
                }))
            }
            AnimationStateKindBinaryAsset::SubMachine { state_machine } => Ok(Self::SubMachine {
                state_machine: state_machine.try_into()?,
            }),
            AnimationStateKindBinaryAsset::GraphRef { graph } => Ok(Self::GraphRef {
                graph: graph.try_into()?,
            }),
        }
    }
}
