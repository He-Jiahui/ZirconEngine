use crate::core::framework::animation::compiler::AnimationCompiledParameterKind;
use crate::core::framework::animation::{
    AnimationBlendSpace1DAsset, AnimationBlendSpace2DAsset, AnimationBlendSpace2DSampleAsset,
    AnimationConditionOperatorAsset, AnimationParameterValue, AnimationStateAsset,
    AnimationStateKindAsset, AnimationStateMachineAsset, AnimationStateMachineLayerAsset,
    AnimationStateMachineLayerBlendModeAsset, AnimationStateTransitionAsset,
    AnimationTransitionConditionAsset, AnimationTransitionInterruptionPolicyAsset,
};
use crate::core::math::Vec2;
use crate::core::resource::{AssetReference, ResourceLocator};

use super::compile_animation_state_machine;

fn reference(path: &str) -> AssetReference {
    AssetReference::from_locator(ResourceLocator::parse(path).expect("fixture locator is valid"))
}

fn graph_state(name: &str) -> AnimationStateAsset {
    AnimationStateAsset {
        name: name.to_string(),
        kind: AnimationStateKindAsset::GraphRef {
            graph: reference("res://animation/locomotion.graph"),
        },
    }
}

fn transition(
    from_state: &str,
    to_state: &str,
    condition: AnimationTransitionConditionAsset,
) -> AnimationStateTransitionAsset {
    AnimationStateTransitionAsset {
        from_state: from_state.to_string(),
        to_state: to_state.to_string(),
        duration_seconds: 0.25,
        exit_time: Some(0.5),
        interruption: AnimationTransitionInterruptionPolicyAsset::Both,
        conditions: vec![condition],
    }
}

#[test]
fn state_machine_compiler_preserves_transition_order_and_resolves_dense_slots() {
    let machine = AnimationStateMachineAsset {
        name: Some("Locomotion".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![graph_state("Idle"), graph_state("Run")],
        transitions: vec![
            transition(
                "Idle",
                "Run",
                AnimationTransitionConditionAsset {
                    parameter: "moving".to_string(),
                    operator: AnimationConditionOperatorAsset::Equal,
                    value: Some(AnimationParameterValue::Bool(true)),
                },
            ),
            transition(
                "Idle",
                "Run",
                AnimationTransitionConditionAsset {
                    parameter: "speed".to_string(),
                    operator: AnimationConditionOperatorAsset::Greater,
                    value: Some(AnimationParameterValue::Scalar(0.1)),
                },
            ),
        ],
        layers: vec![AnimationStateMachineLayerAsset {
            name: "UpperBody".to_string(),
            state_machine: reference("res://animation/upper_body.state_machine"),
            weight: 0.5,
            blend_mode: AnimationStateMachineLayerBlendModeAsset::Additive,
            mask_weights: vec![1.0, 0.5],
        }],
    };

    let compilation = compile_animation_state_machine(&machine);

    assert!(compilation.diagnostics().is_empty());
    let artifact = compilation
        .artifact()
        .expect("valid state machine must produce an artifact");
    assert_eq!(artifact.entry_state(), 0);
    assert_eq!(artifact.states().len(), 2);
    assert_eq!(artifact.transitions().len(), 2);
    assert_eq!(artifact.transitions()[0].from_state(), 0);
    assert_eq!(artifact.transitions()[0].to_state(), 1);
    assert_eq!(artifact.layers().len(), 1);
    assert_eq!(artifact.parameters()[0].name(), "moving");
    assert_eq!(
        artifact.parameters()[0].kind(),
        AnimationCompiledParameterKind::Bool
    );
    assert_eq!(artifact.parameters()[1].name(), "speed");
    assert_eq!(
        artifact.parameters()[1].kind(),
        AnimationCompiledParameterKind::Numeric
    );
}

#[test]
fn state_machine_compiler_rejects_invalid_structure_conditions_and_layers() {
    let machine = AnimationStateMachineAsset {
        name: None,
        entry_state: "Missing".to_string(),
        states: vec![graph_state("Idle"), graph_state("Idle")],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Missing".to_string(),
            to_state: "Idle".to_string(),
            duration_seconds: f32::NAN,
            exit_time: Some(-1.0),
            interruption: AnimationTransitionInterruptionPolicyAsset::None,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "trigger".to_string(),
                operator: AnimationConditionOperatorAsset::Triggered,
                value: Some(AnimationParameterValue::Bool(true)),
            }],
        }],
        layers: vec![AnimationStateMachineLayerAsset {
            name: String::new(),
            state_machine: reference("res://animation/upper_body.state_machine"),
            weight: f32::INFINITY,
            blend_mode: AnimationStateMachineLayerBlendModeAsset::Override,
            mask_weights: vec![-0.1],
        }],
    };

    let compilation = compile_animation_state_machine(&machine);

    assert!(compilation.artifact().is_none());
    let codes: Vec<_> = compilation
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code())
        .collect();
    assert!(codes.contains(&"ZR-ANIM-COMP-STATE-002"));
    assert!(codes.contains(&"ZR-ANIM-COMP-STATE-003"));
    assert!(codes.contains(&"ZR-ANIM-COMP-STATE-004"));
    assert!(codes.contains(&"ZR-ANIM-COMP-STATE-005"));
    assert!(codes.contains(&"ZR-ANIM-COMP-STATE-006"));
    assert!(codes.contains(&"ZR-ANIM-COMP-STATE-008"));
    assert!(codes.contains(&"ZR-ANIM-COMP-STATE-011"));
    assert!(codes.contains(&"ZR-ANIM-COMP-STATE-012"));
}

#[test]
fn state_machine_compiler_rejects_parameter_type_conflicts() {
    let machine = AnimationStateMachineAsset {
        name: None,
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                kind: AnimationStateKindAsset::BlendSpace1D(AnimationBlendSpace1DAsset {
                    parameter: "speed".to_string(),
                    samples: vec![],
                }),
            },
            graph_state("Run"),
        ],
        transitions: vec![transition(
            "Idle",
            "Run",
            AnimationTransitionConditionAsset {
                parameter: "speed".to_string(),
                operator: AnimationConditionOperatorAsset::Triggered,
                value: None,
            },
        )],
        layers: vec![],
    };

    let compilation = compile_animation_state_machine(&machine);

    assert!(compilation.artifact().is_none());
    assert!(compilation
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == "ZR-ANIM-COMP-STATE-010"));
}

#[test]
fn state_machine_compiler_matches_runtime_blend_space_admission_rules() {
    let machine = AnimationStateMachineAsset {
        name: None,
        entry_state: "OneDimensional".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "OneDimensional".to_string(),
                kind: AnimationStateKindAsset::BlendSpace1D(AnimationBlendSpace1DAsset {
                    parameter: "speed".to_string(),
                    samples: vec![],
                }),
            },
            AnimationStateAsset {
                name: "TwoDimensional".to_string(),
                kind: AnimationStateKindAsset::BlendSpace2D(AnimationBlendSpace2DAsset {
                    parameter: "direction".to_string(),
                    samples: vec![
                        AnimationBlendSpace2DSampleAsset {
                            position: Vec2::from_array([0.0, 0.0]),
                            graph: reference("res://animation/idle.graph"),
                        },
                        AnimationBlendSpace2DSampleAsset {
                            position: Vec2::from_array([1.0, 1.0]),
                            graph: reference("res://animation/walk.graph"),
                        },
                        AnimationBlendSpace2DSampleAsset {
                            position: Vec2::from_array([2.0, 2.0]),
                            graph: reference("res://animation/run.graph"),
                        },
                    ],
                }),
            },
        ],
        transitions: vec![],
        layers: vec![],
    };

    let compilation = compile_animation_state_machine(&machine);

    assert!(compilation.artifact().is_none());
    assert!(compilation
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == "ZR-ANIM-COMP-STATE-016"));
    assert!(compilation
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == "ZR-ANIM-COMP-STATE-017"));
}

#[test]
fn state_machine_compiler_accepts_small_non_collinear_blend_space() {
    let machine = blend_space_machine([[0.0, 0.0], [1.0e-20, 0.0], [0.0, 1.0e-20]]);

    let compilation = compile_animation_state_machine(&machine);

    assert!(compilation.artifact().is_some());
    assert!(compilation.diagnostics().is_empty());
}

#[test]
fn state_machine_compiler_treats_signed_zero_positions_as_duplicates() {
    let machine = blend_space_machine([[0.0, 0.0], [-0.0, 0.0], [1.0, 0.0]]);

    let compilation = compile_animation_state_machine(&machine);

    assert!(compilation.artifact().is_none());
    assert!(compilation
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == "ZR-ANIM-COMP-STATE-015"));
}

fn blend_space_machine<const N: usize>(positions: [[f32; 2]; N]) -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: None,
        entry_state: "Blend".to_string(),
        states: vec![AnimationStateAsset {
            name: "Blend".to_string(),
            kind: AnimationStateKindAsset::BlendSpace2D(AnimationBlendSpace2DAsset {
                parameter: "direction".to_string(),
                samples: positions
                    .into_iter()
                    .enumerate()
                    .map(|(index, position)| AnimationBlendSpace2DSampleAsset {
                        position: Vec2::from_array(position),
                        graph: reference(&format!("res://animation/sample-{index}.graph")),
                    })
                    .collect(),
            }),
        }],
        transitions: vec![],
        layers: vec![],
    }
}
