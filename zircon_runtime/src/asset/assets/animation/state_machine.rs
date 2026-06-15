use serde::{Deserialize, Serialize};

use super::binary::{decode_binary_asset, encode_binary_asset, AnimationBinaryAssetKind};
use super::reference::{push_unique_reference, AnimationAssetReferenceBinary};
use crate::asset::AssetReference;
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::math::Real;

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
