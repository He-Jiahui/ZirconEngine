use serde::{Deserialize, Serialize};

use super::binary::{
    decode_binary_asset_with_v3_v2_v1_payload_fallback, encode_binary_asset,
    AnimationBinaryAssetKind,
};
use super::error::{AnimationAssetError, AnimationAssetResult};
use super::reference::AnimationAssetReferenceBinary;
use super::state_kind::{AnimationStateKindAsset, AnimationStateKindBinaryAsset};
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::math::Real;
use crate::core::resource::AssetReference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateAsset {
    pub name: String,
    pub kind: AnimationStateKindAsset,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationStateMachineLayerBlendModeAsset {
    #[default]
    Override,
    Additive,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateMachineLayerAsset {
    pub name: String,
    pub state_machine: AssetReference,
    #[serde(default = "default_layer_weight")]
    pub weight: Real,
    #[serde(default)]
    pub blend_mode: AnimationStateMachineLayerBlendModeAsset,
    #[serde(default)]
    pub mask_weights: Vec<Real>,
}

impl AnimationStateAsset {
    pub fn graph_ref(name: impl Into<String>, graph: AssetReference) -> Self {
        Self {
            name: name.into(),
            kind: AnimationStateKindAsset::GraphRef { graph },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationStateBinaryAsset {
    name: String,
    kind: AnimationStateKindBinaryAsset,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationStateBinaryAssetV2 {
    name: String,
    graph: AnimationAssetReferenceBinary,
}

impl From<&AnimationStateAsset> for AnimationStateBinaryAsset {
    fn from(value: &AnimationStateAsset) -> Self {
        Self {
            name: value.name.clone(),
            kind: AnimationStateKindBinaryAsset::from(&value.kind),
        }
    }
}

impl TryFrom<AnimationStateBinaryAsset> for AnimationStateAsset {
    type Error = AnimationAssetError;

    fn try_from(value: AnimationStateBinaryAsset) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            kind: value.kind.try_into()?,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationTransitionInterruptionPolicyAsset {
    #[default]
    None,
    CurrentToNext,
    NextToNext,
    Both,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateTransitionAsset {
    pub from_state: String,
    pub to_state: String,
    pub duration_seconds: Real,
    // Binary payloads are sequence encoded, so this field must always be
    // serialized. Skipping `None` would shift every following field.
    #[serde(default)]
    pub exit_time: Option<Real>,
    #[serde(default)]
    pub interruption: AnimationTransitionInterruptionPolicyAsset,
    pub conditions: Vec<AnimationTransitionConditionAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateMachineAsset {
    pub name: Option<String>,
    pub entry_state: String,
    pub states: Vec<AnimationStateAsset>,
    pub transitions: Vec<AnimationStateTransitionAsset>,
    #[serde(default)]
    pub layers: Vec<AnimationStateMachineLayerAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationStateMachineBinaryAsset {
    name: Option<String>,
    entry_state: String,
    states: Vec<AnimationStateBinaryAsset>,
    transitions: Vec<AnimationStateTransitionAsset>,
    layers: Vec<AnimationStateMachineLayerAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationStateMachineBinaryAssetV3 {
    name: Option<String>,
    entry_state: String,
    states: Vec<AnimationStateBinaryAsset>,
    transitions: Vec<AnimationStateTransitionAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationStateMachineBinaryAssetV1 {
    name: Option<String>,
    entry_state: String,
    states: Vec<AnimationStateBinaryAssetV2>,
    transitions: Vec<AnimationStateTransitionAssetV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationStateMachineBinaryAssetV2 {
    name: Option<String>,
    entry_state: String,
    states: Vec<AnimationStateBinaryAssetV2>,
    transitions: Vec<AnimationStateTransitionAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationStateTransitionAssetV1 {
    from_state: String,
    to_state: String,
    duration_seconds: Real,
    conditions: Vec<AnimationTransitionConditionAsset>,
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
            layers: value.layers.clone(),
        }
    }
}

impl TryFrom<AnimationStateMachineBinaryAsset> for AnimationStateMachineAsset {
    type Error = AnimationAssetError;

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
            layers: value.layers,
        })
    }
}

impl TryFrom<AnimationStateMachineBinaryAssetV3> for AnimationStateMachineBinaryAsset {
    type Error = AnimationAssetError;

    fn try_from(value: AnimationStateMachineBinaryAssetV3) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            entry_state: value.entry_state,
            states: value.states,
            transitions: value.transitions,
            layers: Vec::new(),
        })
    }
}

impl TryFrom<AnimationStateMachineBinaryAssetV2> for AnimationStateMachineBinaryAsset {
    type Error = AnimationAssetError;

    fn try_from(value: AnimationStateMachineBinaryAssetV2) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            entry_state: value.entry_state,
            states: value
                .states
                .into_iter()
                .map(|state| AnimationStateBinaryAsset {
                    name: state.name,
                    kind: AnimationStateKindBinaryAsset::GraphRef { graph: state.graph },
                })
                .collect(),
            transitions: value.transitions,
            layers: Vec::new(),
        })
    }
}

impl TryFrom<AnimationStateMachineBinaryAssetV1> for AnimationStateMachineBinaryAsset {
    type Error = AnimationAssetError;

    fn try_from(value: AnimationStateMachineBinaryAssetV1) -> Result<Self, Self::Error> {
        AnimationStateMachineBinaryAssetV2 {
            name: value.name,
            entry_state: value.entry_state,
            states: value.states,
            transitions: value
                .transitions
                .into_iter()
                .map(|transition| AnimationStateTransitionAsset {
                    from_state: transition.from_state,
                    to_state: transition.to_state,
                    duration_seconds: transition.duration_seconds,
                    exit_time: None,
                    interruption: AnimationTransitionInterruptionPolicyAsset::None,
                    conditions: transition.conditions,
                })
                .collect(),
        }
        .try_into()
    }
}

impl AnimationStateMachineAsset {
    pub fn from_bytes(bytes: &[u8]) -> AnimationAssetResult<Self> {
        decode_binary_asset_with_v3_v2_v1_payload_fallback::<
            AnimationStateMachineBinaryAsset,
            AnimationStateMachineBinaryAssetV3,
            AnimationStateMachineBinaryAssetV2,
            AnimationStateMachineBinaryAssetV1,
        >(AnimationBinaryAssetKind::StateMachine, bytes)
        .and_then(AnimationStateMachineAsset::try_from)
    }

    pub fn to_bytes(&self) -> AnimationAssetResult<Vec<u8>> {
        encode_binary_asset(
            AnimationBinaryAssetKind::StateMachine,
            &AnimationStateMachineBinaryAsset::from(self),
        )
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::new();
        for state in &self.states {
            state.kind.push_direct_references(&mut references);
        }
        for layer in &self.layers {
            super::reference::push_unique_reference(&mut references, layer.state_machine.clone());
        }
        references
    }
}

fn default_layer_weight() -> Real {
    1.0
}
