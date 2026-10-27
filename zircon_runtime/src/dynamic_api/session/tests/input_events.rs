use super::keyboard_logical_key;

#[test]
fn keyboard_button_names_cover_every_letter_and_digit_without_text() {
    for code in b'A'..=b'Z' {
        let expected = char::from(code).to_string();
        assert_eq!(keyboard_logical_key(u32::from(code), None), Some(expected));
    }
    for code in b'0'..=b'9' {
        let expected = char::from(code).to_string();
        assert_eq!(keyboard_logical_key(u32::from(code), None), Some(expected));
    }
}

#[test]
fn keyboard_button_names_cover_gameplay_and_navigation_keys() {
    for (code, name) in [
        (16, "Shift"),
        (17, "Control"),
        (18, "Alt"),
        (19, "Pause"),
        (20, "CapsLock"),
        (27, "Escape"),
        (32, "Space"),
        (33, "PageUp"),
        (34, "PageDown"),
        (44, "PrintScreen"),
        (45, "Insert"),
        (91, "Meta"),
        (92, "Meta"),
        (93, "ContextMenu"),
        (144, "NumLock"),
        (145, "ScrollLock"),
    ] {
        assert_eq!(keyboard_logical_key(code, None).as_deref(), Some(name));
        assert_eq!(
            keyboard_logical_key(code, Some("unrelated text")).as_deref(),
            Some(name),
            "text must not change the button identity between press and release"
        );
    }
    for number in 1..=24 {
        let name = format!("F{number}");
        assert_eq!(keyboard_logical_key(111 + number, None), Some(name));
    }
}

#[test]
fn keyboard_button_names_cover_punctuation_and_numpad_keys() {
    for (code, name) in [
        (106, "*"),
        (107, "+"),
        (109, "-"),
        (110, "."),
        (111, "/"),
        (186, ";"),
        (187, "="),
        (188, ","),
        (189, "-"),
        (190, "."),
        (191, "/"),
        (192, "`"),
        (219, "["),
        (220, "\\"),
        (221, "]"),
        (222, "'"),
    ] {
        assert_eq!(keyboard_logical_key(code, None).as_deref(), Some(name));
    }
    for digit in 0..=9 {
        assert_eq!(
            keyboard_logical_key(96 + digit, None),
            Some(digit.to_string())
        );
    }
}

#[test]
fn keyboard_button_names_accept_existing_standalone_physical_key_transport() {
    // The app converter sends its legacy subset directly and hashes other
    // KeyCode debug names. Keep that transport usable without changing the ABI.
    fn physical_code(name: &str) -> u32 {
        name.bytes()
            .fold(2_166_136_261_u32, |hash, byte| {
                (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
            })
            .max(1)
    }

    for letter in b'A'..=b'Z' {
        if b"ADSWCVXYZ".contains(&letter) {
            continue;
        }
        let name = char::from(letter).to_string();
        assert_eq!(
            keyboard_logical_key(physical_code(&format!("Key{name}")), None),
            Some(name)
        );
    }
    for digit in 0..=9 {
        for prefix in ["Digit", "Numpad"] {
            assert_eq!(
                keyboard_logical_key(physical_code(&format!("{prefix}{digit}")), None),
                Some(digit.to_string())
            );
        }
    }
    for number in 1..=24 {
        let name = format!("F{number}");
        assert_eq!(keyboard_logical_key(physical_code(&name), None), Some(name));
    }
    for (physical, logical) in [
        ("NumpadEnter", "Enter"),
        ("Pause", "Pause"),
        ("CapsLock", "CapsLock"),
        ("Escape", "Escape"),
        ("Space", "Space"),
        ("PageUp", "PageUp"),
        ("PageDown", "PageDown"),
        ("PrintScreen", "PrintScreen"),
        ("Insert", "Insert"),
        ("SuperLeft", "Meta"),
        ("SuperRight", "Meta"),
        ("ContextMenu", "ContextMenu"),
        ("NumpadMultiply", "*"),
        ("NumpadAdd", "+"),
        ("NumpadSubtract", "-"),
        ("NumpadDecimal", "."),
        ("NumpadDivide", "/"),
        ("NumLock", "NumLock"),
        ("ScrollLock", "ScrollLock"),
        ("Semicolon", ";"),
        ("Equal", "="),
        ("Comma", ","),
        ("Minus", "-"),
        ("Period", "."),
        ("Slash", "/"),
        ("Backquote", "`"),
        ("BracketLeft", "["),
        ("Backslash", "\\"),
        ("BracketRight", "]"),
        ("Quote", "'"),
    ] {
        assert_eq!(
            keyboard_logical_key(physical_code(physical), None).as_deref(),
            Some(logical),
            "standalone physical key {physical}"
        );
    }
    assert_eq!(keyboard_logical_key(0, Some("Escape")), None);
    assert_eq!(keyboard_logical_key(u32::MAX, Some("W")), None);
}

#[test]
fn keyboard_button_escape_ingress_preserves_gameplay_press_and_release() {
    use super::super::profile::RuntimeDynamicSessionProfile;
    use super::super::RuntimeDynamicSession;
    use crate::core::framework::input::{InputButton, InputEvent};
    use zircon_runtime_interface::{
        ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZR_RUNTIME_KEY_ACTION_PRESSED_V1, ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
    };

    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless runtime session should construct");
    // Once the editor's effective Stop binding lets Escape through, both the
    // embedded DOM code and standalone app code must reach the named query.
    for code in [27, 3_082_514_982] {
        for (action, pressed) in [
            (ZR_RUNTIME_KEY_ACTION_PRESSED_V1, true),
            (ZR_RUNTIME_KEY_ACTION_RELEASED_V1, false),
        ] {
            assert!(session
                .handle_event(ZrRuntimeEventV1::keyboard(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    ZrRuntimeViewportHandle::new(1),
                    action,
                    code,
                    0,
                    ZrByteSlice::empty(),
                ))
                .is_ok());
            let input = session.resolve_input_manager().expect("runtime input");
            assert_eq!(
                input.button_pressed(&InputButton::Key("Escape".to_string())),
                pressed
            );
            assert_eq!(input.button_pressed(&InputButton::KeyCode(code)), pressed);
            assert!(input.drain_events().iter().any(|event| matches!(
                event,
                InputEvent::KeyboardInput {
                    key_code,
                    logical_key: Some(name),
                    pressed: is_pressed,
                    ..
                } if *key_code == code && name == "Escape" && *is_pressed == pressed
            )));
        }
    }
}

#[test]
fn keyboard_button_gameplay_host_queries_follow_forwarded_runtime_key_names() {
    use super::super::profile::RuntimeDynamicSessionProfile;
    use super::super::RuntimeDynamicSession;
    use crate::core::framework::script::ScriptHostValue;
    use crate::script::{
        register_gameplay_host_module, with_script_runtime_call_context, CapabilitySet,
        HostExportRegistry, HostRegistry, ScriptRuntimeCallContext,
    };
    use zircon_runtime_interface::{
        ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZR_RUNTIME_KEY_ACTION_PRESSED_V1, ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
    };

    fn query(session: &RuntimeDynamicSession, exports: &HostExportRegistry, key: &str) -> bool {
        let value = with_script_runtime_call_context(
            ScriptRuntimeCallContext {
                core: session.runtime.weak(),
                level: session.level.clone(),
                entity: session.level.with_world(|world| world.active_camera()),
                delta_seconds: 0.016,
            },
            || {
                exports.call_with_capabilities(
                    "zr.zircon.gameplay",
                    "key_pressed",
                    vec![ScriptHostValue::String(key.to_string())],
                    &CapabilitySet::default().with("gameplay.input"),
                )
            },
        )
        .expect("registered gameplay key_pressed should query the session input manager");
        match value {
            ScriptHostValue::Bool(pressed) => pressed,
            value => panic!("key_pressed returned {value:?}"),
        }
    }

    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless runtime session should construct");
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).expect("gameplay host registration");

    for (code, name) in [
        (49, "1"),
        (27, "Escape"),
        (66, "B"),
        (112, "F1"),
        (750_950_213, "1"),
        (3_082_514_982, "Escape"),
        (791_340_490, "B"),
        (332_577_688, "F1"),
    ] {
        for (action, pressed) in [
            (ZR_RUNTIME_KEY_ACTION_PRESSED_V1, true),
            (ZR_RUNTIME_KEY_ACTION_RELEASED_V1, false),
        ] {
            assert!(session
                .handle_event(ZrRuntimeEventV1::keyboard(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    ZrRuntimeViewportHandle::new(1),
                    action,
                    code,
                    0,
                    ZrByteSlice::empty(),
                ))
                .is_ok());
            assert_eq!(query(&session, &exports, name), pressed, "{name} ({code})");
            assert_eq!(
                query(&session, &exports, &format!("KeyCode:{code}")),
                pressed,
                "explicit code queries remain available"
            );
        }
    }
}

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
