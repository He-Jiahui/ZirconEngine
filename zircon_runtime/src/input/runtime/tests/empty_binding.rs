use super::*;

use crate::core::framework::input::{InputAction, InputBinding};

#[test]
fn evaluator_ignores_directly_constructed_empty_bindings() {
    let action_map = InputActionMap {
        actions: vec![InputAction::new("gameplay.jump")],
        bindings: vec![InputBinding {
            action: "gameplay.jump".to_owned(),
            buttons: Vec::new(),
            axes: Vec::new(),
        }],
        ..InputActionMap::default()
    };
    let evaluator = InputActionEvaluator::new(action_map);

    let state = evaluator.evaluate(&InputFrameSnapshot::default());

    assert!(!state.pressed("gameplay.jump"));
    assert!(!state.just_activated("gameplay.jump"));
}
