use serde::{Deserialize, Serialize};

use crate::core::resource::AssetReference;

use super::AnimationParameterMap;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateMachineEvaluation {
    pub parameters: AnimationParameterMap,
    pub active_state: Option<String>,
    pub transitioned: bool,
    pub graph: Option<AssetReference>,
}
