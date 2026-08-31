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
mod tests {
    use zircon_runtime::asset::{AssetReference, AssetUri};
    use zircon_runtime::core::framework::animation::{
        AnimationBlendSpace2DAsset, AnimationBlendSpace2DSampleAsset,
        AnimationConditionOperatorAsset, AnimationParameterMap, AnimationParameterValue,
        AnimationStateAsset, AnimationStateKindAsset, AnimationStateMachineAsset,
        AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
        AnimationTransitionInterruptionPolicyAsset,
    };
    use zircon_runtime::core::math::Vec2;

    use crate::state_machine::{
        compile_animation_state_machine_runtime, StateMachineBlendSamplingState,
    };

    #[test]
    fn one_shot_trigger_evaluation_reports_only_the_selected_transition_triggers() {
        let machine = AnimationStateMachineAsset {
            name: Some("one-shot trigger selection".into()),
            entry_state: "Idle".into(),
            states: vec![state("Idle"), state("Run"), state("Jump")],
            transitions: vec![
                AnimationStateTransitionAsset {
                    from_state: "Idle".into(),
                    to_state: "Run".into(),
                    duration_seconds: 0.2,
                    exit_time: None,
                    interruption: AnimationTransitionInterruptionPolicyAsset::None,
                    conditions: vec![condition(
                        "blocked",
                        AnimationConditionOperatorAsset::Triggered,
                        None,
                    )],
                },
                AnimationStateTransitionAsset {
                    from_state: "Idle".into(),
                    to_state: "Run".into(),
                    duration_seconds: 0.2,
                    exit_time: None,
                    interruption: AnimationTransitionInterruptionPolicyAsset::None,
                    conditions: vec![
                        condition("fire", AnimationConditionOperatorAsset::Triggered, None),
                        condition(
                            "grounded",
                            AnimationConditionOperatorAsset::Equal,
                            Some(AnimationParameterValue::Bool(true)),
                        ),
                    ],
                },
                AnimationStateTransitionAsset {
                    from_state: "Idle".into(),
                    to_state: "Jump".into(),
                    duration_seconds: 0.1,
                    exit_time: None,
                    interruption: AnimationTransitionInterruptionPolicyAsset::None,
                    conditions: vec![condition(
                        "jump",
                        AnimationConditionOperatorAsset::Triggered,
                        None,
                    )],
                },
            ],
            layers: Vec::new(),
        };
        let compiled = compile_animation_state_machine_runtime(&machine).unwrap();
        let parameters = AnimationParameterMap::from([
            ("fire".into(), AnimationParameterValue::Trigger),
            ("grounded".into(), AnimationParameterValue::Bool(true)),
            ("jump".into(), AnimationParameterValue::Trigger),
        ]);

        let evaluation = compiled.evaluate(Some("Idle"), &parameters);

        assert_eq!(
            evaluation
                .transition()
                .map(|transition| transition.to_state.as_str()),
            Some("Run")
        );
        assert_eq!(evaluation.consumed_triggers().collect::<Vec<_>>(), ["fire"]);
    }

    #[test]
    fn blend_space_evaluation_retains_triangle_hint_per_dense_state_slot() {
        let machine = AnimationStateMachineAsset {
            name: Some("retained blend sampling".into()),
            entry_state: "Blend".into(),
            states: vec![AnimationStateAsset {
                name: "Blend".into(),
                kind: AnimationStateKindAsset::BlendSpace2D(AnimationBlendSpace2DAsset {
                    parameter: "direction".into(),
                    samples: vec![
                        blend_sample([0.0, 0.0], "idle"),
                        blend_sample([1.0, 0.0], "right"),
                        blend_sample([0.0, 1.0], "forward"),
                    ],
                }),
            }],
            transitions: Vec::new(),
            layers: Vec::new(),
        };
        let compiled = compile_animation_state_machine_runtime(&machine).unwrap();
        let mut sampling = StateMachineBlendSamplingState::new(compiled.state_count());
        let first = AnimationParameterMap::from([(
            "direction".into(),
            AnimationParameterValue::Vec2([0.2, 0.2]),
        )]);

        let first_values = compiled.project_parameters(&first);
        let evaluation =
            compiled.evaluate_with_blend_sampling(Some("Blend"), &first_values, &mut sampling);

        assert_eq!(evaluation.graph_samples().count(), 3);
        assert!(sampling.triangle_hint(0).is_some());
        let retained = sampling.triangle_hint(0);
        let second = AnimationParameterMap::from([(
            "direction".into(),
            AnimationParameterValue::Vec2([0.25, 0.2]),
        )]);
        let second_values = compiled.project_parameters(&second);
        let evaluation =
            compiled.evaluate_with_blend_sampling(Some("Blend"), &second_values, &mut sampling);
        assert_eq!(evaluation.graph_samples().count(), 3);
        assert_eq!(sampling.triangle_hint(0), retained);
    }

    fn state(name: &str) -> AnimationStateAsset {
        AnimationStateAsset::graph_ref(
            name,
            AssetReference::from_locator(
                AssetUri::parse(&format!("res://animation/{name}.zranim")).unwrap(),
            ),
        )
    }

    fn condition(
        parameter: &str,
        operator: AnimationConditionOperatorAsset,
        value: Option<AnimationParameterValue>,
    ) -> AnimationTransitionConditionAsset {
        AnimationTransitionConditionAsset {
            parameter: parameter.into(),
            operator,
            value,
        }
    }

    fn blend_sample(position: [f32; 2], name: &str) -> AnimationBlendSpace2DSampleAsset {
        AnimationBlendSpace2DSampleAsset {
            position: Vec2::from_array(position),
            graph: AssetReference::from_locator(
                AssetUri::parse(&format!("res://animation/{name}.zranim")).unwrap(),
            ),
        }
    }
}
