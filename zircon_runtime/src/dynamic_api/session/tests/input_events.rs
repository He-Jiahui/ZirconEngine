use super::keyboard_logical_key;

#[test]
fn keyboard_logical_key_maps_wasd_runtime_key_codes_for_gameplay_scripts() {
    assert_eq!(keyboard_logical_key(87, None), Some("W".to_string()));
    assert_eq!(keyboard_logical_key(65, None), Some("A".to_string()));
    assert_eq!(keyboard_logical_key(83, None), Some("S".to_string()));
    assert_eq!(keyboard_logical_key(68, None), Some("D".to_string()));
}

#[test]
fn keyboard_logical_key_maps_digit_runtime_key_codes_for_choice_inputs() {
    assert_eq!(keyboard_logical_key(49, None), Some("1".to_string()));
    assert_eq!(keyboard_logical_key(50, None), Some("2".to_string()));
    assert_eq!(keyboard_logical_key(51, None), Some("3".to_string()));
}

#[test]
fn editing_command_codes_keep_their_semantic_names() {
    for (code, name) in [
        (8, "Backspace"),
        (9, "Tab"),
        (13, "Enter"),
        (36, "Home"),
        (35, "End"),
        (37, "ArrowLeft"),
        (38, "ArrowUp"),
        (39, "ArrowRight"),
        (40, "ArrowDown"),
        (46, "Delete"),
        (67, "C"),
        (86, "V"),
        (88, "X"),
        (89, "Y"),
        (90, "Z"),
    ] {
        assert_eq!(keyboard_logical_key(code, None).as_deref(), Some(name));
    }
    assert_eq!(keyboard_logical_key(u32::MAX, None), None);
}
