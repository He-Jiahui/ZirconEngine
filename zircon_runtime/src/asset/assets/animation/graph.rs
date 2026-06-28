use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::binary::{
    decode_binary_asset_with_v1_payload_fallback, encode_binary_asset, AnimationBinaryAssetKind,
};
use super::error::{AnimationAssetError, AnimationAssetResult};
use super::reference::{push_unique_reference, AnimationAssetReferenceBinary};
use crate::asset::AssetReference;
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::math::Real;

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationGraphNodeAsset {
    Clip {
        id: String,
        clip: AssetReference,
        playback_speed: Real,
        looping: bool,
    },
    Blend {
        id: String,
        inputs: Vec<String>,
        weight_parameter: Option<String>,
    },
    Additive {
        id: String,
        base: String,
        additive: String,
        weight_parameter: Option<String>,
    },
    Mask {
        id: String,
        input: String,
        target_ids: Vec<String>,
    },
    Output {
        source: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationGraphNodeBinary {
    tag: u8,
    id: String,
    clip: Option<AnimationAssetReferenceBinary>,
    playback_speed: Real,
    looping: bool,
    inputs: Vec<String>,
    weight_parameter: Option<String>,
    source: String,
    #[serde(default)]
    base: String,
    #[serde(default)]
    additive: String,
    #[serde(default)]
    input: String,
    #[serde(default)]
    target_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationGraphNodeBinaryV1 {
    tag: u8,
    id: String,
    clip: Option<AnimationAssetReferenceBinary>,
    playback_speed: Real,
    looping: bool,
    inputs: Vec<String>,
    weight_parameter: Option<String>,
    source: String,
}

impl From<&AnimationGraphNodeAsset> for AnimationGraphNodeBinary {
    fn from(value: &AnimationGraphNodeAsset) -> Self {
        match value {
            AnimationGraphNodeAsset::Clip {
                id,
                clip,
                playback_speed,
                looping,
            } => Self {
                tag: 0,
                id: id.clone(),
                clip: Some(AnimationAssetReferenceBinary::from(clip)),
                playback_speed: *playback_speed,
                looping: *looping,
                inputs: Vec::new(),
                weight_parameter: None,
                source: String::new(),
                base: String::new(),
                additive: String::new(),
                input: String::new(),
                target_ids: Vec::new(),
            },
            AnimationGraphNodeAsset::Blend {
                id,
                inputs,
                weight_parameter,
            } => Self {
                tag: 1,
                id: id.clone(),
                clip: None,
                playback_speed: 1.0,
                looping: false,
                inputs: inputs.clone(),
                weight_parameter: weight_parameter.clone(),
                source: String::new(),
                base: String::new(),
                additive: String::new(),
                input: String::new(),
                target_ids: Vec::new(),
            },
            AnimationGraphNodeAsset::Additive {
                id,
                base,
                additive,
                weight_parameter,
            } => Self {
                tag: 3,
                id: id.clone(),
                clip: None,
                playback_speed: 1.0,
                looping: false,
                inputs: Vec::new(),
                weight_parameter: weight_parameter.clone(),
                source: String::new(),
                base: base.clone(),
                additive: additive.clone(),
                input: String::new(),
                target_ids: Vec::new(),
            },
            AnimationGraphNodeAsset::Mask {
                id,
                input,
                target_ids,
            } => Self {
                tag: 4,
                id: id.clone(),
                clip: None,
                playback_speed: 1.0,
                looping: false,
                inputs: Vec::new(),
                weight_parameter: None,
                source: String::new(),
                base: String::new(),
                additive: String::new(),
                input: input.clone(),
                target_ids: target_ids.clone(),
            },
            AnimationGraphNodeAsset::Output { source } => Self {
                tag: 2,
                id: String::new(),
                clip: None,
                playback_speed: 1.0,
                looping: false,
                inputs: Vec::new(),
                weight_parameter: None,
                source: source.clone(),
                base: String::new(),
                additive: String::new(),
                input: String::new(),
                target_ids: Vec::new(),
            },
        }
    }
}

impl TryFrom<AnimationGraphNodeBinary> for AnimationGraphNodeAsset {
    type Error = AnimationAssetError;

    fn try_from(value: AnimationGraphNodeBinary) -> Result<Self, Self::Error> {
        match value.tag {
            0 => Ok(Self::Clip {
                id: value.id,
                clip: value
                    .clip
                    .ok_or(AnimationAssetError::MissingGraphClipReference)?
                    .try_into()?,
                playback_speed: value.playback_speed,
                looping: value.looping,
            }),
            1 => Ok(Self::Blend {
                id: value.id,
                inputs: value.inputs,
                weight_parameter: value.weight_parameter,
            }),
            2 => Ok(Self::Output {
                source: value.source,
            }),
            3 => Ok(Self::Additive {
                id: value.id,
                base: value.base,
                additive: value.additive,
                weight_parameter: value.weight_parameter,
            }),
            4 => Ok(Self::Mask {
                id: value.id,
                input: value.input,
                target_ids: value.target_ids,
            }),
            tag => Err(AnimationAssetError::UnknownGraphNodeTag { tag }),
        }
    }
}

impl TryFrom<AnimationGraphNodeBinaryV1> for AnimationGraphNodeAsset {
    type Error = AnimationAssetError;

    fn try_from(value: AnimationGraphNodeBinaryV1) -> Result<Self, Self::Error> {
        match value.tag {
            0 => Ok(Self::Clip {
                id: value.id,
                clip: value
                    .clip
                    .ok_or(AnimationAssetError::MissingGraphClipReference)?
                    .try_into()?,
                playback_speed: value.playback_speed,
                looping: value.looping,
            }),
            1 => Ok(Self::Blend {
                id: value.id,
                inputs: value.inputs,
                weight_parameter: value.weight_parameter,
            }),
            2 => Ok(Self::Output {
                source: value.source,
            }),
            tag => Err(AnimationAssetError::UnknownGraphNodeTag { tag }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationGraphAssetV1 {
    name: Option<String>,
    parameters: Vec<AnimationGraphParameterAsset>,
    nodes: Vec<AnimationGraphNodeBinaryV1>,
}

impl TryFrom<AnimationGraphAssetV1> for AnimationGraphAsset {
    type Error = AnimationAssetError;

    fn try_from(value: AnimationGraphAssetV1) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            parameters: value.parameters,
            nodes: value
                .nodes
                .into_iter()
                .map(AnimationGraphNodeAsset::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl Serialize for AnimationGraphNodeAsset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        AnimationGraphNodeBinary::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnimationGraphNodeAsset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        AnimationGraphNodeBinary::deserialize(deserializer)?
            .try_into()
            .map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationGraphAsset {
    pub name: Option<String>,
    pub parameters: Vec<AnimationGraphParameterAsset>,
    pub nodes: Vec<AnimationGraphNodeAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationGraphParameterAsset {
    pub name: String,
    pub default_value: AnimationParameterValue,
}

impl AnimationGraphAsset {
    pub fn from_bytes(bytes: &[u8]) -> AnimationAssetResult<Self> {
        decode_binary_asset_with_v1_payload_fallback::<AnimationGraphAsset, AnimationGraphAssetV1>(
            AnimationBinaryAssetKind::Graph,
            bytes,
        )
    }

    pub fn to_bytes(&self) -> AnimationAssetResult<Vec<u8>> {
        encode_binary_asset(AnimationBinaryAssetKind::Graph, self)
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::new();
        for node in &self.nodes {
            if let AnimationGraphNodeAsset::Clip { clip, .. } = node {
                push_unique_reference(&mut references, clip.clone());
            }
        }
        references
    }
}
