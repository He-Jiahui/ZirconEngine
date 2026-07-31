use woc_client::{
    gamepad_action_options, gamepad_button, GamepadBindings, GamepadControllerModel, GamepadKind,
    GAMEPAD_NONE_ACTION,
};

#[test]
fn action_options_match_the_target_controller_dropdown_catalog() {
    let options = gamepad_action_options();
    assert_eq!(options.len(), 55);
    assert_eq!(options[0].action_id, GAMEPAD_NONE_ACTION);
    assert_eq!(options[1].action_id, "escape");
    assert_eq!(options[2].action_id, "jump");
    assert_eq!(options[3].action_id, "autorun");
    for excluded in [
        "forward",
        "back",
        "turnLeft",
        "turnRight",
        "strafeLeft",
        "strafeRight",
        "emoteWheel",
        "attackMove",
    ] {
        assert!(
            options.iter().all(|option| option.action_id != excluded),
            "{excluded}"
        );
    }
    for included in ["target", "chat", "petAttack", "slot0", "slot22"] {
        assert!(
            options.iter().any(|option| option.action_id == included),
            "{included}"
        );
    }
}

#[test]
fn controller_rows_follow_w3c_order_and_physical_brand_labels() {
    let model = GamepadControllerModel::new(GamepadKind::Nintendo);
    let rows = model.rows(&GamepadBindings::default());
    assert_eq!(rows.len(), 16);
    assert_eq!((rows[0].button, rows[0].button_label.as_str()), (0, "B"));
    assert_eq!((rows[1].button, rows[1].button_label.as_str()), (1, "A"));
    assert_eq!(rows[0].action, "jump");
    assert_eq!(rows[9].button, gamepad_button::START);
    assert_eq!(rows[9].action, "escape");
    assert_eq!(rows[15].button, gamepad_button::DPAD_RIGHT);
}

#[test]
fn controller_model_allows_duplicate_actions_and_explicit_unbind() {
    let model = GamepadControllerModel::new(GamepadKind::Xbox);
    let mut bindings = GamepadBindings::default();
    model.bind(&mut bindings, gamepad_button::A, "slot1");
    model.bind(&mut bindings, gamepad_button::B, "slot1");
    assert_eq!(bindings.action_for(gamepad_button::A), "slot1");
    assert_eq!(bindings.action_for(gamepad_button::B), "slot1");

    model.bind(&mut bindings, gamepad_button::A, GAMEPAD_NONE_ACTION);
    assert_eq!(bindings.action_for(gamepad_button::A), GAMEPAD_NONE_ACTION);
}

#[test]
fn controller_reset_restores_rows_without_changing_detected_kind() {
    let model = GamepadControllerModel::new(GamepadKind::PlayStation);
    let mut bindings = GamepadBindings::default();
    model.bind(&mut bindings, gamepad_button::A, "slot8");
    model.reset(&mut bindings);

    assert_eq!(model.kind(), GamepadKind::PlayStation);
    let rows = model.rows(&bindings);
    assert_eq!(rows[0].button_label, "Cross");
    assert_eq!(rows[0].action, "jump");
}
