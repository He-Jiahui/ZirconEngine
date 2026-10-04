use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationBlendSpace2DAsset, AnimationBlendSpace2DSampleAsset, AnimationConditionOperatorAsset,
    AnimationParameterMap, AnimationParameterValue, AnimationStateAsset, AnimationStateKindAsset,
    AnimationStateMachineAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
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
