use crate::core::framework::input::InputButton;

use super::script_input_button;

#[test]
fn gameplay_key_query_compiles_codes_and_names_to_typed_buttons() {
    assert_eq!(script_input_button("KeyCode:87"), InputButton::KeyCode(87));
    assert_eq!(
        script_input_button("Jump"),
        InputButton::Key("Jump".to_string())
    );
}

#[test]
fn gameplay_key_query_uses_direct_manager_lookup_without_snapshot_clone() {
    let source = include_str!("../input.rs");
    let key_pressed = source
        .split("pub(super) fn key_pressed")
        .nth(1)
        .and_then(|source| source.split("fn script_input_button").next())
        .expect("key_pressed source section");

    assert!(key_pressed.contains("input.button_pressed"));
    assert!(key_pressed.contains("script_input_button(key)"));
    assert!(!key_pressed.contains("input.snapshot()"));
}
