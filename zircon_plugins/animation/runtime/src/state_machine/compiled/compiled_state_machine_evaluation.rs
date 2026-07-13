use zircon_runtime::asset::AssetReference;
use zircon_runtime::core::framework::animation::AnimationStateTransitionEvaluation;
use zircon_runtime::core::math::Real;

use crate::TransitionDesc;

#[derive(Clone, Debug)]
pub struct CompiledStateMachineEvaluation<'a> {
    pub(super) active_state: &'a str,
    pub(super) clip: Option<&'a AssetReference>,
    pub(super) sub_machine: Option<&'a AssetReference>,
    pub(super) graph_samples: super::CompiledGraphSamples<'a>,
    pub(super) transition: Option<AnimationStateTransitionEvaluation>,
    pub(super) transition_desc: Option<TransitionDesc>,
}

impl CompiledStateMachineEvaluation<'_> {
    pub fn active_state(&self) -> &str {
        self.active_state
    }

    pub fn graph_samples(&self) -> impl Iterator<Item = (&AssetReference, Real)> + '_ {
        self.graph_samples.iter().filter_map(|sample| *sample)
    }

    pub fn clip(&self) -> Option<&AssetReference> {
        self.clip
    }

    pub fn sub_machine(&self) -> Option<&AssetReference> {
        self.sub_machine
    }

    pub fn transition(&self) -> Option<&AnimationStateTransitionEvaluation> {
        self.transition.as_ref()
    }

    pub fn transition_desc(&self) -> Option<TransitionDesc> {
        self.transition_desc
    }
}
