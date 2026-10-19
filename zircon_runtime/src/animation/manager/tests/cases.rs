use std::panic::{self, AssertUnwindSafe};

use crate::core::framework::animation::{AnimationManager, AnimationPlaybackSettings};

use super::DefaultAnimationManager;

#[test]
fn animation_manager_playback_settings_recover_poisoned_lock() {
    let manager = DefaultAnimationManager::default();
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.lock_playback_settings();
        panic!("poison animation playback settings");
    }));

    let mut playback_settings = AnimationPlaybackSettings::default();
    playback_settings.enabled = false;
    manager
        .store_playback_settings(playback_settings.clone())
        .expect("store playback settings after poisoned lock");

    assert_eq!(manager.playback_settings(), playback_settings);
}

#[test]
fn raw_state_machine_numeric_conditions_reject_bool_and_trigger_operands() {
    use crate::core::framework::animation::{
        AnimationConditionOperatorAsset as Operator, AnimationParameterMap,
        AnimationParameterValue as Value, AnimationStateAsset, AnimationStateMachineAsset,
        AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
    };
    use crate::core::resource::{AssetReference, ResourceLocator};

    let manager = DefaultAnimationManager::default();
    let graph = AssetReference::from_locator(
        ResourceLocator::parse("res://animation/condition.graph.zranim").unwrap(),
    );
    let evaluate = |current: Value, operator, expected| {
        let machine = AnimationStateMachineAsset {
            name: None,
            entry_state: "idle".into(),
            states: vec![
                AnimationStateAsset::graph_ref("idle", graph.clone()),
                AnimationStateAsset::graph_ref("target", graph.clone()),
            ],
            transitions: vec![AnimationStateTransitionAsset {
                from_state: "idle".into(),
                to_state: "target".into(),
                duration_seconds: 0.0,
                exit_time: None,
                interruption: Default::default(),
                conditions: vec![AnimationTransitionConditionAsset {
                    parameter: "value".into(),
                    operator,
                    value: expected,
                }],
            }],
            layers: Vec::new(),
        };
        let mut parameters = AnimationParameterMap::new();
        parameters.insert("value".into(), current);
        manager.evaluate_state_machine(&machine, Some("idle"), &parameters)
    };
    for (operator, numeric) in [
        (Operator::Greater, -1.0),
        (Operator::GreaterEqual, 0.0),
        (Operator::Less, 1.0),
        (Operator::LessEqual, 0.0),
    ] {
        for invalid in [Value::Bool(false), Value::Bool(true), Value::Trigger] {
            let result = evaluate(invalid.clone(), operator, Some(Value::Scalar(numeric)));
            assert_eq!(result.active_state.as_deref(), Some("idle"));
            assert!(!result.transitioned);
            assert!(result.transition.is_none());
            let result = evaluate(Value::Scalar(-numeric), operator, Some(invalid));
            assert_eq!(result.active_state.as_deref(), Some("idle"));
            assert!(!result.transitioned);
            assert!(result.transition.is_none());
        }
    }
    for (current, operator, expected) in [
        (
            Value::Integer(2),
            Operator::Greater,
            Some(Value::Scalar(1.0)),
        ),
        (
            Value::Scalar(2.0),
            Operator::GreaterEqual,
            Some(Value::Integer(2)),
        ),
        (Value::Integer(1), Operator::Less, Some(Value::Scalar(2.0))),
        (
            Value::Scalar(2.0),
            Operator::LessEqual,
            Some(Value::Integer(2)),
        ),
        (Value::Bool(true), Operator::Equal, Some(Value::Bool(true))),
        (
            Value::Bool(true),
            Operator::NotEqual,
            Some(Value::Bool(false)),
        ),
        (Value::Trigger, Operator::Triggered, None),
    ] {
        let result = evaluate(current, operator, expected);
        assert_eq!(result.active_state.as_deref(), Some("target"));
        assert!(result.transitioned);
        assert!(result.transition.is_none());
    }
}
