use woc_client::{
    default_gamepad_action, detect_gamepad_kind, gamepad_button, gamepad_button_label, GamepadKind,
    BINDABLE_GAMEPAD_BUTTONS, DEFAULT_GAMEPAD_BINDINGS, STANDARD_GAMEPAD_BUTTON_COUNT,
};

#[test]
fn default_layout_binds_every_standard_non_guide_button() {
    assert_eq!(STANDARD_GAMEPAD_BUTTON_COUNT, 17);
    assert_eq!(BINDABLE_GAMEPAD_BUTTONS.len(), 16);
    assert_eq!(
        DEFAULT_GAMEPAD_BINDINGS.len(),
        BINDABLE_GAMEPAD_BUTTONS.len()
    );
    assert_eq!(default_gamepad_action(gamepad_button::A), Some("jump"));
    assert_eq!(
        default_gamepad_action(gamepad_button::START),
        Some("escape")
    );
    assert_eq!(default_gamepad_action(gamepad_button::GUIDE), None);

    for button in BINDABLE_GAMEPAD_BUTTONS {
        assert!(default_gamepad_action(button).is_some(), "button {button}");
    }
}

#[test]
fn default_layout_covers_action_slots_zero_through_eight_once() {
    for slot in 0..=8 {
        let expected = format!("slot{slot}");
        assert_eq!(
            DEFAULT_GAMEPAD_BINDINGS
                .iter()
                .filter(|(_, action)| *action == expected)
                .count(),
            1,
            "{expected}"
        );
    }
}

#[test]
fn brand_detection_prefers_product_names_then_reads_vendor_fields() {
    for (id, expected) in [
        ("DualSense Wireless Controller", GamepadKind::PlayStation),
        (
            "Wireless Controller (STANDARD GAMEPAD Vendor: 054c Product: 09cc)",
            GamepadKind::PlayStation,
        ),
        (
            "Xbox Wireless Controller (STANDARD GAMEPAD Vendor: 045e Product: 02fd)",
            GamepadKind::Xbox,
        ),
        ("Microsoft X-Box 360 pad", GamepadKind::Xbox),
        (
            "Xbox 360 Controller (XInput STANDARD GAMEPAD)",
            GamepadKind::Xbox,
        ),
        (
            "Pro Controller (STANDARD GAMEPAD Vendor: 057e Product: 2009)",
            GamepadKind::Nintendo,
        ),
        ("Joy-Con (L/R)", GamepadKind::Nintendo),
        ("054c-0ce6-Wireless Controller", GamepadKind::PlayStation),
        ("045e-02fd-", GamepadKind::Xbox),
    ] {
        assert_eq!(detect_gamepad_kind(id), expected, "{id}");
    }
}

#[test]
fn product_id_collision_never_overrides_name_or_vendor_field() {
    assert_eq!(
        detect_gamepad_kind("Xbox Wireless Controller (Vendor: 045e Product: 054c)"),
        GamepadKind::Xbox
    );
    assert_eq!(
        detect_gamepad_kind("Wireless Controller (Vendor: 045e Product: 054c)"),
        GamepadKind::Xbox
    );
    assert_eq!(
        detect_gamepad_kind("Wireless Controller (Vendor: 057e Product: 2009)"),
        GamepadKind::Nintendo
    );
    assert_eq!(
        detect_gamepad_kind("Some Random Pad (Vendor: 1234 Product: 5678)"),
        GamepadKind::Generic
    );
    assert_eq!(detect_gamepad_kind(""), GamepadKind::Generic);
}

#[test]
fn labels_follow_each_controllers_physical_silk_screen() {
    assert_eq!(
        gamepad_button_label(gamepad_button::A, GamepadKind::Nintendo),
        "B"
    );
    assert_eq!(
        gamepad_button_label(gamepad_button::B, GamepadKind::Nintendo),
        "A"
    );
    assert_eq!(
        gamepad_button_label(gamepad_button::X, GamepadKind::Nintendo),
        "Y"
    );
    assert_eq!(
        gamepad_button_label(gamepad_button::Y, GamepadKind::Nintendo),
        "X"
    );
    assert_eq!(
        gamepad_button_label(gamepad_button::A, GamepadKind::Xbox),
        "A"
    );
    assert_eq!(
        gamepad_button_label(gamepad_button::A, GamepadKind::PlayStation),
        "Cross"
    );
    assert_eq!(
        gamepad_button_label(gamepad_button::LT, GamepadKind::PlayStation),
        "L2"
    );
    assert_eq!(
        gamepad_button_label(gamepad_button::LT, GamepadKind::Nintendo),
        "ZL"
    );
}

#[test]
fn every_brand_labels_every_bindable_button_and_unknowns_fall_back() {
    for kind in GamepadKind::ALL {
        for button in BINDABLE_GAMEPAD_BUTTONS {
            let label = gamepad_button_label(button, kind);
            assert!(!label.is_empty(), "{kind:?} #{button}");
            assert!(!label.starts_with('#'), "{kind:?} #{button}");
        }
        assert_eq!(
            gamepad_button_label(gamepad_button::DPAD_UP, kind),
            "D-pad ↑"
        );
    }
    assert_eq!(
        gamepad_button_label(gamepad_button::A, GamepadKind::Generic),
        "A / Cross"
    );
    assert_eq!(gamepad_button_label(99, GamepadKind::Xbox), "#99");
}
