use serde::de::{DeserializeOwned, Error as DeError};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::framework::animation::{AnimationParameterValue, AnimationTrackPath};
use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::core::math::Real;

use crate::asset::{AssetReference, AssetUri, AssetUuid};

const ANIMATION_BINARY_MAGIC: [u8; 8] = *b"ZRANIM01";
const ANIMATION_BINARY_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AnimationBinaryAssetKind {
    Skeleton,
    Clip,
    Sequence,
    Graph,
    StateMachine,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationBinaryDocument<T> {
    magic: [u8; 8],
    version: u32,
    kind: AnimationBinaryAssetKind,
    payload: T,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationBinaryHeader {
    magic: [u8; 8],
    version: u32,
    kind: AnimationBinaryAssetKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationAssetReferenceBinary {
    uuid: String,
    url: String,
}

impl From<&AssetReference> for AnimationAssetReferenceBinary {
    fn from(value: &AssetReference) -> Self {
        Self {
            uuid: value.uuid.to_string(),
            url: value.locator.to_string(),
        }
    }
}

impl TryFrom<AnimationAssetReferenceBinary> for AssetReference {
    type Error = String;

    fn try_from(value: AnimationAssetReferenceBinary) -> Result<Self, Self::Error> {
        let uuid = value
            .uuid
            .parse::<AssetUuid>()
            .map_err(|error| error.to_string())?;
        let locator = AssetUri::parse(&value.url).map_err(|error| error.to_string())?;
        Ok(AssetReference::new(uuid, locator))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationInterpolationAsset {
    Step,
    Hermite,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationChannelValueAsset {
    Bool(bool),
    Integer(i32),
    Scalar(Real),
    Vec2([Real; 2]),
    Vec3([Real; 3]),
    Vec4([Real; 4]),
    Quaternion([Real; 4]),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationChannelValueBinary {
    tag: u8,
    bool_value: bool,
    integer_value: i32,
    scalar_values: [Real; 4],
    arity: u8,
}

impl From<&AnimationChannelValueAsset> for AnimationChannelValueBinary {
    fn from(value: &AnimationChannelValueAsset) -> Self {
        match value {
            AnimationChannelValueAsset::Bool(bool_value) => Self {
                tag: 0,
                bool_value: *bool_value,
                integer_value: 0,
                scalar_values: [0.0; 4],
                arity: 0,
            },
            AnimationChannelValueAsset::Integer(integer_value) => Self {
                tag: 1,
                bool_value: false,
                integer_value: *integer_value,
                scalar_values: [0.0; 4],
                arity: 0,
            },
            AnimationChannelValueAsset::Scalar(value) => Self {
                tag: 2,
                bool_value: false,
                integer_value: 0,
                scalar_values: [*value, 0.0, 0.0, 0.0],
                arity: 1,
            },
            AnimationChannelValueAsset::Vec2(value) => Self {
                tag: 3,
                bool_value: false,
                integer_value: 0,
                scalar_values: [value[0], value[1], 0.0, 0.0],
                arity: 2,
            },
            AnimationChannelValueAsset::Vec3(value) => Self {
                tag: 4,
                bool_value: false,
                integer_value: 0,
                scalar_values: [value[0], value[1], value[2], 0.0],
                arity: 3,
            },
            AnimationChannelValueAsset::Vec4(value) => Self {
                tag: 5,
                bool_value: false,
                integer_value: 0,
                scalar_values: *value,
                arity: 4,
            },
            AnimationChannelValueAsset::Quaternion(value) => Self {
                tag: 6,
                bool_value: false,
                integer_value: 0,
                scalar_values: *value,
                arity: 4,
            },
        }
    }
}

impl TryFrom<AnimationChannelValueBinary> for AnimationChannelValueAsset {
    type Error = String;

    fn try_from(value: AnimationChannelValueBinary) -> Result<Self, Self::Error> {
        match value.tag {
            0 => Ok(Self::Bool(value.bool_value)),
            1 => Ok(Self::Integer(value.integer_value)),
            2 => Ok(Self::Scalar(value.scalar_values[0])),
            3 => Ok(Self::Vec2([value.scalar_values[0], value.scalar_values[1]])),
            4 => Ok(Self::Vec3([
                value.scalar_values[0],
                value.scalar_values[1],
                value.scalar_values[2],
            ])),
            5 => Ok(Self::Vec4(value.scalar_values)),
            6 => Ok(Self::Quaternion(value.scalar_values)),
            other => Err(format!("unknown animation channel value tag {other}")),
        }
    }
}

impl Serialize for AnimationChannelValueAsset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        AnimationChannelValueBinary::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnimationChannelValueAsset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        AnimationChannelValueBinary::deserialize(deserializer)?
            .try_into()
            .map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationChannelKeyAsset {
    pub time_seconds: Real,
    pub value: AnimationChannelValueAsset,
    pub in_tangent: Option<AnimationChannelValueAsset>,
    pub out_tangent: Option<AnimationChannelValueAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationChannelAsset {
    pub interpolation: AnimationInterpolationAsset,
    pub keys: Vec<AnimationChannelKeyAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSkeletonBoneAsset {
    pub name: String,
    pub parent_index: Option<u32>,
    pub local_translation: [Real; 3],
    pub local_rotation: [Real; 4],
    pub local_scale: [Real; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSkeletonAsset {
    pub name: Option<String>,
    pub bones: Vec<AnimationSkeletonBoneAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationClipBoneTrackAsset {
    pub bone_name: String,
    /// Stable optional retargeting id; legacy assets can continue to use `bone_name` alone.
    #[serde(default)]
    pub target_id: Option<String>,
    pub translation: AnimationChannelAsset,
    pub rotation: AnimationChannelAsset,
    pub scale: AnimationChannelAsset,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationEventTrackAsset {
    #[serde(default)]
    pub target_id: Option<String>,
    pub event: String,
    pub time_seconds: Real,
    #[serde(default)]
    pub payload: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationClipAsset {
    pub name: Option<String>,
    pub skeleton: AssetReference,
    pub duration_seconds: Real,
    pub tracks: Vec<AnimationClipBoneTrackAsset>,
    #[serde(default)]
    pub event_tracks: Vec<AnimationEventTrackAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationClipBinaryAsset {
    name: Option<String>,
    skeleton: AnimationAssetReferenceBinary,
    duration_seconds: Real,
    tracks: Vec<AnimationClipBoneTrackAsset>,
    #[serde(default)]
    event_tracks: Vec<AnimationEventTrackAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationClipBinaryAssetV1 {
    name: Option<String>,
    skeleton: AnimationAssetReferenceBinary,
    duration_seconds: Real,
    tracks: Vec<AnimationClipBoneTrackAssetV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationClipBoneTrackAssetV1 {
    bone_name: String,
    translation: AnimationChannelAsset,
    rotation: AnimationChannelAsset,
    scale: AnimationChannelAsset,
}

impl From<&AnimationClipAsset> for AnimationClipBinaryAsset {
    fn from(value: &AnimationClipAsset) -> Self {
        Self {
            name: value.name.clone(),
            skeleton: AnimationAssetReferenceBinary::from(&value.skeleton),
            duration_seconds: value.duration_seconds,
            tracks: value.tracks.clone(),
            event_tracks: value.event_tracks.clone(),
        }
    }
}

impl TryFrom<AnimationClipBinaryAsset> for AnimationClipAsset {
    type Error = String;

    fn try_from(value: AnimationClipBinaryAsset) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            skeleton: value.skeleton.try_into()?,
            duration_seconds: value.duration_seconds,
            tracks: value.tracks,
            event_tracks: value.event_tracks,
        })
    }
}

impl TryFrom<AnimationClipBinaryAssetV1> for AnimationClipBinaryAsset {
    type Error = String;

    fn try_from(value: AnimationClipBinaryAssetV1) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            skeleton: value.skeleton,
            duration_seconds: value.duration_seconds,
            tracks: value
                .tracks
                .into_iter()
                .map(|track| AnimationClipBoneTrackAsset {
                    bone_name: track.bone_name,
                    target_id: None,
                    translation: track.translation,
                    rotation: track.rotation,
                    scale: track.scale,
                })
                .collect(),
            event_tracks: Vec::new(),
        })
    }
}

impl TryFrom<AnimationClipBinaryAssetV1> for AnimationClipAsset {
    type Error = String;

    fn try_from(value: AnimationClipBinaryAssetV1) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            skeleton: value.skeleton.try_into()?,
            duration_seconds: value.duration_seconds,
            tracks: value
                .tracks
                .into_iter()
                .map(|track| AnimationClipBoneTrackAsset {
                    bone_name: track.bone_name,
                    target_id: None,
                    translation: track.translation,
                    rotation: track.rotation,
                    scale: track.scale,
                })
                .collect(),
            event_tracks: Vec::new(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSequenceTrackAsset {
    pub property_path: ComponentPropertyPath,
    pub channel: AnimationChannelAsset,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSequenceBindingAsset {
    pub entity_path: EntityPath,
    /// Stable optional scene target id; `entity_path` remains the path fallback for existing assets.
    #[serde(default)]
    pub target_id: Option<String>,
    pub tracks: Vec<AnimationSequenceTrackAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationSequenceAssetV1 {
    name: Option<String>,
    duration_seconds: Real,
    frames_per_second: Real,
    bindings: Vec<AnimationSequenceBindingAssetV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationSequenceBindingAssetV1 {
    entity_path: EntityPath,
    tracks: Vec<AnimationSequenceTrackAsset>,
}

impl From<AnimationSequenceAssetV1> for AnimationSequenceAsset {
    fn from(value: AnimationSequenceAssetV1) -> Self {
        Self {
            name: value.name,
            duration_seconds: value.duration_seconds,
            frames_per_second: value.frames_per_second,
            bindings: value
                .bindings
                .into_iter()
                .map(|binding| AnimationSequenceBindingAsset {
                    entity_path: binding.entity_path,
                    target_id: None,
                    tracks: binding.tracks,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSequenceAsset {
    pub name: Option<String>,
    pub duration_seconds: Real,
    pub frames_per_second: Real,
    pub bindings: Vec<AnimationSequenceBindingAsset>,
}

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
    type Error = String;

    fn try_from(value: AnimationGraphNodeBinary) -> Result<Self, Self::Error> {
        match value.tag {
            0 => Ok(Self::Clip {
                id: value.id,
                clip: value
                    .clip
                    .ok_or_else(|| {
                        "animation graph clip node is missing clip reference".to_string()
                    })?
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
            other => Err(format!("unknown animation graph node tag {other}")),
        }
    }
}

impl TryFrom<AnimationGraphNodeBinaryV1> for AnimationGraphNodeAsset {
    type Error = String;

    fn try_from(value: AnimationGraphNodeBinaryV1) -> Result<Self, Self::Error> {
        match value.tag {
            0 => Ok(Self::Clip {
                id: value.id,
                clip: value
                    .clip
                    .ok_or_else(|| {
                        "animation graph clip node is missing clip reference".to_string()
                    })?
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
            other => Err(format!("unknown animation graph node tag {other}")),
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
    type Error = String;

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateAsset {
    pub name: String,
    pub graph: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationStateBinaryAsset {
    name: String,
    graph: AnimationAssetReferenceBinary,
}

impl From<&AnimationStateAsset> for AnimationStateBinaryAsset {
    fn from(value: &AnimationStateAsset) -> Self {
        Self {
            name: value.name.clone(),
            graph: AnimationAssetReferenceBinary::from(&value.graph),
        }
    }
}

impl TryFrom<AnimationStateBinaryAsset> for AnimationStateAsset {
    type Error = String;

    fn try_from(value: AnimationStateBinaryAsset) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            graph: value.graph.try_into()?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationConditionOperatorAsset {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Triggered,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationTransitionConditionAsset {
    pub parameter: String,
    pub operator: AnimationConditionOperatorAsset,
    pub value: Option<AnimationParameterValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateTransitionAsset {
    pub from_state: String,
    pub to_state: String,
    pub duration_seconds: Real,
    pub conditions: Vec<AnimationTransitionConditionAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateMachineAsset {
    pub name: Option<String>,
    pub entry_state: String,
    pub states: Vec<AnimationStateAsset>,
    pub transitions: Vec<AnimationStateTransitionAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationStateMachineBinaryAsset {
    name: Option<String>,
    entry_state: String,
    states: Vec<AnimationStateBinaryAsset>,
    transitions: Vec<AnimationStateTransitionAsset>,
}

impl From<&AnimationStateMachineAsset> for AnimationStateMachineBinaryAsset {
    fn from(value: &AnimationStateMachineAsset) -> Self {
        Self {
            name: value.name.clone(),
            entry_state: value.entry_state.clone(),
            states: value
                .states
                .iter()
                .map(AnimationStateBinaryAsset::from)
                .collect(),
            transitions: value.transitions.clone(),
        }
    }
}

impl TryFrom<AnimationStateMachineBinaryAsset> for AnimationStateMachineAsset {
    type Error = String;

    fn try_from(value: AnimationStateMachineBinaryAsset) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            entry_state: value.entry_state,
            states: value
                .states
                .into_iter()
                .map(AnimationStateAsset::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            transitions: value.transitions,
        })
    }
}

impl AnimationSkeletonAsset {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        decode_binary_asset(AnimationBinaryAssetKind::Skeleton, bytes)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        encode_binary_asset(AnimationBinaryAssetKind::Skeleton, self)
    }
}

impl AnimationClipAsset {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        decode_binary_asset_with_fallback::<AnimationClipBinaryAsset, AnimationClipBinaryAssetV1>(
            AnimationBinaryAssetKind::Clip,
            bytes,
        )
        .and_then(AnimationClipAsset::try_from)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        encode_binary_asset(
            AnimationBinaryAssetKind::Clip,
            &AnimationClipBinaryAsset::from(self),
        )
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        vec![self.skeleton.clone()]
    }
}

impl AnimationSequenceAsset {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        decode_binary_asset_with_fallback::<AnimationSequenceAsset, AnimationSequenceAssetV1>(
            AnimationBinaryAssetKind::Sequence,
            bytes,
        )
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        encode_binary_asset(AnimationBinaryAssetKind::Sequence, self)
    }

    pub fn track_paths(&self) -> Vec<AnimationTrackPath> {
        self.bindings
            .iter()
            .flat_map(|binding| {
                binding.tracks.iter().cloned().map(|track| {
                    AnimationTrackPath::new(binding.entity_path.clone(), track.property_path)
                })
            })
            .collect()
    }

    pub fn target_track_paths(&self) -> Vec<(Option<String>, AnimationTrackPath)> {
        self.bindings
            .iter()
            .flat_map(|binding| {
                binding.tracks.iter().cloned().map(|track| {
                    (
                        binding.target_id.clone(),
                        AnimationTrackPath::new(binding.entity_path.clone(), track.property_path),
                    )
                })
            })
            .collect()
    }
}

impl AnimationGraphAsset {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        decode_binary_asset_with_fallback::<AnimationGraphAsset, AnimationGraphAssetV1>(
            AnimationBinaryAssetKind::Graph,
            bytes,
        )
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
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

impl AnimationStateMachineAsset {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        decode_binary_asset::<AnimationStateMachineBinaryAsset>(
            AnimationBinaryAssetKind::StateMachine,
            bytes,
        )
        .and_then(AnimationStateMachineAsset::try_from)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        encode_binary_asset(
            AnimationBinaryAssetKind::StateMachine,
            &AnimationStateMachineBinaryAsset::from(self),
        )
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::new();
        for state in &self.states {
            push_unique_reference(&mut references, state.graph.clone());
        }
        references
    }
}

fn push_unique_reference(references: &mut Vec<AssetReference>, reference: AssetReference) {
    if !references
        .iter()
        .any(|existing| existing.uuid == reference.uuid && existing.locator == reference.locator)
    {
        references.push(reference);
    }
}

fn encode_binary_asset<T>(kind: AnimationBinaryAssetKind, payload: &T) -> Result<Vec<u8>, String>
where
    T: Serialize + Clone,
{
    bincode::serialize(&AnimationBinaryDocument {
        magic: ANIMATION_BINARY_MAGIC,
        version: ANIMATION_BINARY_VERSION,
        kind,
        payload: payload.clone(),
    })
    .map_err(|error| error.to_string())
}

fn decode_binary_asset<T>(kind: AnimationBinaryAssetKind, bytes: &[u8]) -> Result<T, String>
where
    T: DeserializeOwned,
{
    match decode_binary_document_asset(kind, bytes) {
        Ok(payload) => Ok(payload),
        Err(document_error) => decode_binary_stream_asset(kind, bytes).map_err(|stream_error| {
            format!("{document_error}; animation stream decode failed: {stream_error}")
        }),
    }
}

fn decode_binary_asset_with_fallback<T, Legacy>(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> Result<T, String>
where
    T: DeserializeOwned,
    Legacy: DeserializeOwned + TryInto<T>,
    <Legacy as TryInto<T>>::Error: std::fmt::Display,
{
    match decode_binary_asset(kind, bytes) {
        Ok(payload) => Ok(payload),
        Err(primary_error) => {
            let legacy = decode_binary_asset::<Legacy>(kind, bytes)
                .and_then(|payload| payload.try_into().map_err(|error| error.to_string()))
                .map_err(|legacy_error| {
                    format!("{primary_error}; legacy animation asset decode failed: {legacy_error}")
                })?;
            Ok(legacy)
        }
    }
}

fn decode_binary_document_asset<T>(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let document: AnimationBinaryDocument<T> =
        bincode::deserialize(bytes).map_err(|error| error.to_string())?;
    validate_binary_header(kind, document.magic, document.version, document.kind)?;
    Ok(document.payload)
}

fn decode_binary_stream_asset<T>(kind: AnimationBinaryAssetKind, bytes: &[u8]) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let mut cursor = std::io::Cursor::new(bytes);
    let header: AnimationBinaryHeader =
        bincode::deserialize_from(&mut cursor).map_err(|error| error.to_string())?;
    validate_binary_header(kind, header.magic, header.version, header.kind)?;

    bincode::deserialize_from(&mut cursor).map_err(|error| error.to_string())
}

fn validate_binary_header(
    kind: AnimationBinaryAssetKind,
    magic: [u8; 8],
    version: u32,
    actual_kind: AnimationBinaryAssetKind,
) -> Result<(), String> {
    if magic != ANIMATION_BINARY_MAGIC {
        return Err("invalid animation asset magic".to_string());
    }
    if version != ANIMATION_BINARY_VERSION {
        return Err(format!("unsupported animation asset version {}", version));
    }
    if actual_kind != kind {
        return Err(format!(
            "animation asset kind mismatch: expected {:?}, found {:?}",
            kind, actual_kind
        ));
    }
    Ok(())
}
