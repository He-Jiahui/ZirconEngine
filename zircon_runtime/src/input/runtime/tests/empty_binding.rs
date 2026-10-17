use super::*;

use crate::core::framework::input::{InputAction, InputBinding};
use crate::input::InputButton;

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

#[test]
fn deserialization_drops_empty_bindings_and_preserves_valid_bindings() {
    let action_map: InputActionMap = serde_json::from_str(
        r#"{
            "actions": [{"id": "gameplay.jump", "context": null, "display_name": null}],
            "bindings": [
                {"action": "gameplay.jump", "buttons": []},
                {"action": "gameplay.jump", "buttons": ["MouseLeft"]}
            ]
        }"#,
    )
    .expect("valid map with an empty and a button binding should deserialize");

    assert_eq!(action_map.bindings.len(), 1);
    assert_eq!(action_map.bindings[0].buttons, vec![InputButton::MouseLeft]);

    let evaluator = InputActionEvaluator::new(action_map);
    let mut frame = InputFrameSnapshot::default();
    frame.buttons.press(InputButton::MouseLeft);

    let state = evaluator.evaluate(&frame);

    assert!(state.pressed("gameplay.jump"));
    assert!(state.just_activated("gameplay.jump"));
}
