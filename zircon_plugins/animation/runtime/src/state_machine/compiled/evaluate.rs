use zircon_runtime::asset::AssetReference;
use zircon_runtime::core::framework::animation::{
    AnimationParameterMap, AnimationStateTransitionEvaluation,
};

use crate::{TransitionDesc, TransitionState};

use super::{CompiledAnimationStateMachine, CompiledStateMachineEvaluation};

impl CompiledAnimationStateMachine {
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn parameter_count(&self) -> usize {
        self.parameter_names.len()
    }

    pub(crate) fn graph_samples_for_state<'a>(
        &'a self,
        name: &str,
        parameters: &AnimationParameterMap,
    ) -> Option<super::CompiledGraphSamples<'a>> {
        let slot = self.state_slots.get(name)?;
        let values = self.parameter_values(parameters);
        Some(self.states[slot.index()].graph_samples(&values))
    }

    pub(crate) fn clip_for_state<'a>(&'a self, name: &str) -> Option<&'a AssetReference> {
        let slot = self.state_slots.get(name)?;
        self.states[slot.index()].clip()
    }

    pub(crate) fn sub_machine_for_state<'a>(&'a self, name: &str) -> Option<&'a AssetReference> {
        let slot = self.state_slots.get(name)?;
        self.states[slot.index()].sub_machine()
    }

    pub(crate) fn transition_state(&self, name: &str) -> Option<TransitionState> {
        let slot = self.state_slots.get(name)?;
        Some(TransitionState::new(u32::try_from(slot.index()).ok()?))
    }

    pub(crate) fn transition_desc(&self, from: &str, to: &str) -> Option<TransitionDesc> {
        let from = self.state_slots.get(from)?;
        let to = self.state_slots.get(to)?;
        self.transitions[from.index()]
            .iter()
            .find(|transition| transition.to == *to)
            .map(|transition| transition.desc)
    }

    pub fn evaluate<'a>(
        &'a self,
        current: Option<&str>,
        parameters: &AnimationParameterMap,
    ) -> CompiledStateMachineEvaluation<'a> {
        let active = current
            .and_then(|name| self.state_slots.get(name).copied())
            .unwrap_or(self.entry);
        let values = self.parameter_values(parameters);
        let state = &self.states[active.index()];
        let compiled_transition = self.transitions[active.index()]
            .iter()
            .find(|transition| transition.conditions.evaluate(&values));
        let transition = compiled_transition.map(|transition| AnimationStateTransitionEvaluation {
            from_state: state.name.clone(),
            to_state: self.states[transition.to.index()].name.clone(),
            duration_seconds: transition.desc.duration_seconds(),
        });
        CompiledStateMachineEvaluation {
            active_state: &state.name,
            clip: state.clip(),
            sub_machine: state.sub_machine(),
            graph_samples: state.graph_samples(&values),
            transition,
            transition_desc: compiled_transition.map(|transition| transition.desc),
        }
    }

    fn parameter_values<'a>(
        &self,
        parameters: &'a AnimationParameterMap,
    ) -> Vec<Option<&'a zircon_runtime::core::framework::animation::AnimationParameterValue>> {
        self.parameter_names
            .iter()
            .map(|name| parameters.get(name))
            .collect()
    }
}
