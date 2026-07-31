use woc_client::{
    combo_code, combo_modifiers, is_modifier_code, is_reserved_combo, key_cap_label, key_label,
    keybind_storage_key, make_key_combo, normalize_key_combo, KeyBindingKind, KeyModifiers,
    LEGACY_KEYBIND_STORAGE_KEY,
};

#[test]
fn combo_builder_uses_the_target_modifier_order() {
    assert_eq!(
        make_key_combo(
            "KeyA",
            KeyModifiers {
                ctrl: true,
                alt: true,
                shift: true,
                meta: true,
            }
        ),
        "Ctrl+Alt+Shift+Meta+KeyA"
    );
    assert_eq!(make_key_combo("Digit1", KeyModifiers::default()), "Digit1");
}

#[test]
fn combo_parser_round_trips_code_and_modifiers() {
    let combo = "Ctrl+Alt+Shift+Meta+KeyA";
    assert_eq!(combo_code(combo), "KeyA");
    assert_eq!(
        combo_modifiers(combo),
        KeyModifiers {
            ctrl: true,
            alt: true,
            shift: true,
            meta: true,
        }
    );
    assert_eq!(combo_code("Digit1"), "Digit1");
    assert_eq!(combo_modifiers("Digit1"), KeyModifiers::default());
}

#[test]
fn only_physical_modifier_keys_are_modifier_codes() {
    for code in [
        "ShiftLeft",
        "ShiftRight",
        "ControlLeft",
        "ControlRight",
        "AltLeft",
        "AltRight",
        "MetaLeft",
        "MetaRight",
    ] {
        assert!(is_modifier_code(code), "{code}");
    }
    assert!(!is_modifier_code("KeyA"));
    assert!(!is_modifier_code("Escape"));
}

#[test]
fn escape_is_reserved_under_every_modifier_layer() {
    assert!(is_reserved_combo("Escape"));
    assert!(is_reserved_combo("Shift+Escape"));
    assert!(is_reserved_combo("Ctrl+Alt+Escape"));
    assert!(!is_reserved_combo("Shift+KeyA"));
}

#[test]
fn edge_bindings_keep_chords_while_held_bindings_keep_the_physical_key() {
    assert_eq!(
        normalize_key_combo(KeyBindingKind::Edge, "Shift+Digit1"),
        "Shift+Digit1"
    );
    assert_eq!(
        normalize_key_combo(KeyBindingKind::Held, "Shift+KeyW"),
        "KeyW"
    );
}

#[test]
fn key_labels_match_full_and_compact_target_keycaps() {
    for (combo, full, compact) in [
        ("Digit1", "1", "1"),
        ("KeyA", "A", "a"),
        ("Shift+KeyZ", "Shift+Z", "s-z"),
        ("Ctrl+Alt+Digit1", "Ctrl+Alt+1", "c-a-1"),
        ("ArrowUp", "↑", "↑"),
        ("NumpadDecimal", "Num.", "num."),
    ] {
        let label = key_label(Some(combo));
        assert_eq!(label, full);
        assert_eq!(key_cap_label(&label), compact);
    }
    assert_eq!(key_label(None), "");
}

#[test]
fn keybind_storage_scopes_match_legacy_online_and_offline_profiles() {
    assert_eq!(keybind_storage_key(""), LEGACY_KEYBIND_STORAGE_KEY);
    assert_eq!(keybind_storage_key("char:42"), "woc_keybinds:char:42");
    assert_eq!(
        keybind_storage_key("offline:shaman:Storm Caller"),
        "woc_keybinds:offline:shaman:Storm Caller"
    );
}
