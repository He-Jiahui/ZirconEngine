//! 以稠密参数投影评估当前状态的第一个可用转换；混合空间三角提示只属于当前实例和状态槽。
use zircon_runtime::asset::AssetReference;
use zircon_runtime::core::framework::animation::{
    AnimationParameterMap, AnimationParameterValue, AnimationStateTransitionEvaluation,
};

use crate::{TransitionDesc, TransitionState};

use super::{
    CompiledAnimationStateMachine, CompiledStateMachineEvaluation, StateMachineBlendSamplingState,
};

pub(crate) type StateMachineParameterValues = Box<[Option<AnimationParameterValue>]>;

impl CompiledAnimationStateMachine {
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn parameter_count(&self) -> usize {
        self.parameter_names.len()
    }

    pub(crate) fn graph_samples_for_state_with_blend_sampling<'a>(
        &'a self,
        name: &str,
        parameter_values: &[Option<AnimationParameterValue>],
        sampling: &mut StateMachineBlendSamplingState,
    ) -> Option<super::CompiledGraphSamples<'a>> {
        let slot = self.state_slots.get(name)?;
        sampling.ensure_state_count(self.states.len());
        let hint = sampling.triangle_hint_mut(slot.index())?;
        Some(self.states[slot.index()].graph_samples_with_hint(parameter_values, Some(hint)))
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

    /// 选择第一个满足条件的转换请求；退出时间、交叉淡入和触发器提交由实例管线处理。
    pub fn evaluate<'a>(
        &'a self,
        current: Option<&str>,
        parameters: &AnimationParameterMap,
    ) -> CompiledStateMachineEvaluation<'a> {
        let values = self.project_parameters(parameters);
        self.evaluate_internal(current, &values, None)
    }

    pub(crate) fn evaluate_with_blend_sampling<'a>(
        &'a self,
        current: Option<&str>,
        parameter_values: &[Option<AnimationParameterValue>],
        sampling: &mut StateMachineBlendSamplingState,
    ) -> CompiledStateMachineEvaluation<'a> {
        sampling.ensure_state_count(self.states.len());
        self.evaluate_internal(current, parameter_values, Some(sampling))
    }

    fn evaluate_internal<'a>(
        &'a self,
        current: Option<&str>,
        values: &[Option<AnimationParameterValue>],
        mut sampling: Option<&mut StateMachineBlendSamplingState>,
    ) -> CompiledStateMachineEvaluation<'a> {
        let active = current
            .and_then(|name| self.state_slots.get(name).copied())
            .unwrap_or(self.entry);
        let state = &self.states[active.index()];
        let compiled_transition = self.transitions[active.index()]
            .iter()
            .find(|transition| transition.conditions.evaluate(values));
        let transition = compiled_transition.map(|transition| AnimationStateTransitionEvaluation {
            from_state: state.name.clone(),
            to_state: self.states[transition.to.index()].name.clone(),
            duration_seconds: transition.desc.duration_seconds(),
        });
        let consumed_triggers =
            compiled_transition.map(|transition| transition.consumed_triggers.clone());
        let graph_samples = match sampling
            .as_deref_mut()
            .and_then(|sampling| sampling.triangle_hint_mut(active.index()))
        {
            Some(hint) => state.graph_samples_with_hint(values, Some(hint)),
            None => state.graph_samples(values),
        };
        CompiledStateMachineEvaluation {
            active_state: &state.name,
            clip: state.clip(),
            sub_machine: state.sub_machine(),
            graph_samples,
            transition,
            transition_desc: compiled_transition.map(|transition| transition.desc),
            consumed_triggers,
        }
    }

    pub(crate) fn project_parameters(
        &self,
        parameters: &AnimationParameterMap,
    ) -> StateMachineParameterValues {
        self.parameter_names
            .iter()
            .map(|name| parameters.get(name).cloned())
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    pub(crate) fn parameter_layout(&self) -> &std::sync::Arc<[String]> {
        &self.parameter_names
    }
}

#[cfg(test)]
#[path = "tests/evaluate.rs"]
mod tests;
