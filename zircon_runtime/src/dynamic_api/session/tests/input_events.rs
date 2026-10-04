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
