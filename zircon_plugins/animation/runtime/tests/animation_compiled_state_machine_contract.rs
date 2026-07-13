use zircon_plugin_animation_runtime::{
    AnimationStateMachineCompileError, CompiledAnimationStateMachine, CompiledConditionExpression,
    ConditionExpression,
};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationBlendSpace1DAsset, AnimationBlendSpace1DSampleAsset, AnimationConditionOperatorAsset,
    AnimationStateAsset, AnimationStateKindAsset, AnimationStateMachineAsset,
    AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
    AnimationTransitionInterruptionPolicyAsset,
};
use zircon_runtime::core::framework::animation::{AnimationParameterMap, AnimationParameterValue};

#[test]
fn compiled_state_machine_keeps_dense_states_after_source_mutation() {
    let mut source = machine();
    let compiled = CompiledAnimationStateMachine::compile(&source).unwrap();
    source.entry_state = "Missing".into();
    source.states[0].name = "Renamed".into();
    source.transitions[0].to_state = "Missing".into();
    let evaluation = compiled.evaluate(
        Some("Idle"),
        &AnimationParameterMap::from([("speed".into(), AnimationParameterValue::Scalar(1.0))]),
    );
    assert_eq!(compiled.state_count(), 2);
    assert_eq!(compiled.parameter_count(), 1);
    assert_eq!(evaluation.active_state(), "Idle");
    assert_eq!(evaluation.transition().unwrap().to_state, "Run");
    let transition_desc = evaluation.transition_desc().unwrap();
    assert_eq!(transition_desc.exit_time(), Some(0.75));
    assert_eq!(
        transition_desc.interruption(),
        zircon_plugin_animation_runtime::InterruptionPolicy::Both
    );
}

#[test]
fn compiled_state_machine_rejects_duplicate_and_missing_states() {
    let mut duplicate = machine();
    duplicate.states[1].name = "Idle".into();
    assert!(matches!(
        CompiledAnimationStateMachine::compile(&duplicate),
        Err(AnimationStateMachineCompileError::DuplicateState { .. })
    ));
    let mut missing = machine();
    missing.transitions[0].to_state = "Missing".into();
    assert!(matches!(
        CompiledAnimationStateMachine::compile(&missing),
        Err(AnimationStateMachineCompileError::MissingState { .. })
    ));
}

#[test]
fn condition_expr_and_or_matrix() {
    let speed = condition(
        "speed",
        AnimationConditionOperatorAsset::GreaterEqual,
        Some(AnimationParameterValue::Scalar(0.5)),
    );
    let grounded = condition(
        "grounded",
        AnimationConditionOperatorAsset::Equal,
        Some(AnimationParameterValue::Bool(true)),
    );
    let blocked = condition(
        "blocked",
        AnimationConditionOperatorAsset::Equal,
        Some(AnimationParameterValue::Bool(true)),
    );
    let mut source = ConditionExpression::all([
        ConditionExpression::any([
            ConditionExpression::condition(speed),
            ConditionExpression::condition(grounded),
        ]),
        ConditionExpression::not(ConditionExpression::condition(blocked)),
    ]);
    let compiled = CompiledConditionExpression::compile(&source).unwrap();

    if let ConditionExpression::All(children) = &mut source {
        *children = Box::new([]);
    }

    let cases = [
        (0.6, false, false, true),
        (0.1, true, false, true),
        (0.6, false, true, false),
        (0.1, false, false, false),
    ];
    for (speed, grounded, blocked, expected) in cases {
        let parameters = AnimationParameterMap::from([
            ("speed".into(), AnimationParameterValue::Scalar(speed)),
            ("grounded".into(), AnimationParameterValue::Bool(grounded)),
            ("blocked".into(), AnimationParameterValue::Bool(blocked)),
        ]);
        assert_eq!(compiled.evaluate(&parameters), expected);
    }
    assert_eq!(compiled.parameter_count(), 3);
}

#[test]
fn compiled_blend_space_state_resolves_dense_graph_weights_after_source_mutation() {
    let mut source = machine();
    source.states[0].kind = AnimationStateKindAsset::BlendSpace1D(AnimationBlendSpace1DAsset {
        parameter: "speed".into(),
        samples: vec![
            AnimationBlendSpace1DSampleAsset {
                position: 0.0,
                graph: graph("idle"),
            },
            AnimationBlendSpace1DSampleAsset {
                position: 1.0,
                graph: graph("run"),
            },
        ],
    });
    let compiled = CompiledAnimationStateMachine::compile(&source).unwrap();
    source.states[0].kind = AnimationStateKindAsset::GraphRef {
        graph: graph("mutated"),
    };

    let evaluation = compiled.evaluate(
        Some("Idle"),
        &AnimationParameterMap::from([("speed".into(), AnimationParameterValue::Scalar(0.25))]),
    );
    let samples = evaluation
        .graph_samples()
        .map(|(graph, weight)| (graph.locator.to_string(), weight))
        .collect::<Vec<_>>();
    assert_eq!(
        samples,
        vec![
            ("res://animation/idle.zranim".to_string(), 0.75),
            ("res://animation/run.zranim".to_string(), 0.25),
        ]
    );
}

#[test]
fn compiled_clip_state_retains_dense_reference_after_source_mutation() {
    let mut source = machine();
    source.states[0].kind = AnimationStateKindAsset::Clip {
        clip: graph("idle-clip"),
    };
    let compiled = CompiledAnimationStateMachine::compile(&source).unwrap();
    source.states[0].kind = AnimationStateKindAsset::GraphRef {
        graph: graph("mutated"),
    };

    let evaluation = compiled.evaluate(Some("Idle"), &AnimationParameterMap::new());

    assert_eq!(
        evaluation.clip().unwrap().locator.to_string(),
        "res://animation/idle-clip.zranim"
    );
    assert_eq!(evaluation.graph_samples().count(), 0);
}

#[test]
fn compiled_sub_machine_state_retains_dense_reference_after_source_mutation() {
    let mut source = machine();
    source.states[0].kind = AnimationStateKindAsset::SubMachine {
        state_machine: graph("nested-machine"),
    };
    let compiled = CompiledAnimationStateMachine::compile(&source).unwrap();
    source.states[0].kind = AnimationStateKindAsset::GraphRef {
        graph: graph("mutated"),
    };

    let evaluation = compiled.evaluate(Some("Idle"), &AnimationParameterMap::new());

    assert_eq!(
        evaluation.sub_machine().unwrap().locator.to_string(),
        "res://animation/nested-machine.zranim"
    );
    assert_eq!(evaluation.graph_samples().count(), 0);
}

fn machine() -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("Locomotion".into()),
        entry_state: "Idle".into(),
        states: vec![state("Idle", "idle"), state("Run", "run")],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".into(),
            to_state: "Run".into(),
            duration_seconds: 0.2,
            exit_time: Some(0.75),
            interruption: AnimationTransitionInterruptionPolicyAsset::Both,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "speed".into(),
                operator: AnimationConditionOperatorAsset::Greater,
                value: Some(AnimationParameterValue::Scalar(0.5)),
            }],
        }],
        layers: Vec::new(),
    }
}

fn state(name: &str, graph_name: &str) -> AnimationStateAsset {
    AnimationStateAsset {
        name: name.into(),
        kind: AnimationStateKindAsset::GraphRef {
            graph: graph(graph_name),
        },
    }
}

fn graph(name: &str) -> AssetReference {
    AssetReference::from_locator(
        AssetUri::parse(&format!("res://animation/{name}.zranim")).unwrap(),
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
