use super::*;

#[test]
fn key_states_map_to_runtime_constants() {
    assert_eq!(
        key_action(ElementState::Pressed),
        Some(ZR_RUNTIME_KEY_ACTION_PRESSED_V1)
    );
    assert_eq!(
        key_action(ElementState::Released),
        Some(ZR_RUNTIME_KEY_ACTION_RELEASED_V1)
    );
}

#[test]
fn physical_keys_map_to_runtime_values() {
    assert_eq!(
        physical_key_code(&PhysicalKey::Code(KeyCode::ShiftLeft)),
        16
    );
    assert_eq!(
        physical_key_code(&PhysicalKey::Code(KeyCode::ControlRight)),
        17
    );
    assert_eq!(physical_key_code(&PhysicalKey::Code(KeyCode::AltLeft)), 18);
    assert_eq!(physical_key_code(&PhysicalKey::Code(KeyCode::KeyW)), 87);
    assert_eq!(physical_key_code(&PhysicalKey::Code(KeyCode::KeyA)), 65);
    assert_eq!(physical_key_code(&PhysicalKey::Code(KeyCode::KeyS)), 83);
    assert_eq!(physical_key_code(&PhysicalKey::Code(KeyCode::KeyD)), 68);
    assert_eq!(
        physical_key_code(&PhysicalKey::Unidentified(NativeKeyCode::Xkb(77))),
        77
    );
}

#[test]
fn fallback_key_codes_keep_the_previous_debug_fnv_values() {
    for (code, expected) in [
        (KeyCode::Escape, 3_082_514_982),
        (KeyCode::F12, 3_736_956_062),
        (KeyCode::Numpad8, 2_044_486_356),
        (KeyCode::Numpad9, 2_061_263_975),
    ] {
        assert_eq!(physical_key_code(&PhysicalKey::Code(code)), expected);
    }
}

#[test]
fn production_key_fallback_formats_into_the_hash_without_allocating() {
    let production = include_str!("../keyboard.rs")
        .split_once("\n#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("keyboard production source precedes its test module");

    assert!(!production.contains("format!("));
    assert!(!production.contains("to_string("));
    assert!(production.contains("impl fmt::Write for StableKeyCodeHasher"));
    assert!(production.contains("fmt::write(&mut hasher, format_args!(\"{code:?}\"))"));
}

#[test]
fn editing_command_keys_use_runtime_legacy_codes() {
    for (key, expected) in [
        (KeyCode::Backspace, 8),
        (KeyCode::Tab, 9),
        (KeyCode::Enter, 13),
        (KeyCode::Home, 36),
        (KeyCode::End, 35),
        (KeyCode::ArrowLeft, 37),
        (KeyCode::ArrowUp, 38),
        (KeyCode::ArrowRight, 39),
        (KeyCode::ArrowDown, 40),
        (KeyCode::Delete, 46),
        (KeyCode::KeyC, 67),
        (KeyCode::KeyV, 86),
        (KeyCode::KeyX, 88),
        (KeyCode::KeyY, 89),
        (KeyCode::KeyZ, 90),
    ] {
        assert_eq!(
            physical_key_code(&PhysicalKey::Code(key)),
            expected,
            "{key:?}"
        );
    }
}
