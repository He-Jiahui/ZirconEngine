use woc_client::{
    gamepad_button, GamepadBindingEntry, GamepadBindings, BINDABLE_GAMEPAD_BUTTONS,
    GAMEPAD_NONE_ACTION,
};

#[test]
fn defaults_cover_every_bindable_button_in_panel_order() {
    let bindings = GamepadBindings::default();
    let entries = bindings.entries();

    assert_eq!(entries.len(), BINDABLE_GAMEPAD_BUTTONS.len());
    for (entry, expected_button) in entries.iter().zip(BINDABLE_GAMEPAD_BUTTONS) {
        assert_eq!(entry.button, expected_button);
        assert_ne!(entry.action, GAMEPAD_NONE_ACTION);
    }
    assert_eq!(bindings.action_for(gamepad_button::A), "jump");
}

#[test]
fn bind_replaces_or_clears_one_button_without_action_uniqueness() {
    let mut bindings = GamepadBindings::default();
    bindings.bind(gamepad_button::A, "slot1");
    bindings.bind(gamepad_button::B, "slot1");
    assert_eq!(bindings.action_for(gamepad_button::A), "slot1");
    assert_eq!(bindings.action_for(gamepad_button::B), "slot1");

    bindings.bind(gamepad_button::A, GAMEPAD_NONE_ACTION);
    assert_eq!(bindings.action_for(gamepad_button::A), GAMEPAD_NONE_ACTION);
    assert_eq!(bindings.action_for(gamepad_button::B), "slot1");
}

#[test]
fn non_bindable_buttons_are_ignored() {
    let mut bindings = GamepadBindings::default();
    bindings.bind(gamepad_button::GUIDE, "slot5");
    bindings.bind(99, "slot6");
    assert_eq!(
        bindings.action_for(gamepad_button::GUIDE),
        GAMEPAD_NONE_ACTION
    );
    assert_eq!(bindings.action_for(99), GAMEPAD_NONE_ACTION);
    assert_eq!(bindings.entries().len(), 16);
}

#[test]
fn stored_overrides_apply_only_to_bindable_indices_and_keep_arbitrary_action_ids() {
    let bindings = GamepadBindings::from_stored([
        GamepadBindingEntry {
            button: gamepad_button::A,
            action: "customAction".to_string(),
        },
        GamepadBindingEntry {
            button: gamepad_button::GUIDE,
            action: "slot5".to_string(),
        },
        GamepadBindingEntry {
            button: 99,
            action: "slot6".to_string(),
        },
    ]);

    assert_eq!(bindings.action_for(gamepad_button::A), "customAction");
    assert_eq!(
        bindings.action_for(gamepad_button::GUIDE),
        GAMEPAD_NONE_ACTION
    );
    assert_eq!(bindings.action_for(gamepad_button::B), "interact");
}

#[test]
fn reset_restores_the_complete_target_layout() {
    let mut bindings = GamepadBindings::default();
    bindings.bind(gamepad_button::A, GAMEPAD_NONE_ACTION);
    bindings.bind(gamepad_button::START, "slot8");

    bindings.reset();
    assert_eq!(bindings.action_for(gamepad_button::A), "jump");
    assert_eq!(bindings.action_for(gamepad_button::START), "escape");
}
