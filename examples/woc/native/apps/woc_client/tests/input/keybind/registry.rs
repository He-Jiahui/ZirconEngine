use woc_client::{
    action_allows_shared, action_kind, keybind_action, KeyBindingKind, ACTION_BAR_SLOTS,
    KEYBIND_ACTIONS, KEYBIND_CATEGORIES,
};

#[test]
fn registry_has_all_target_categories_and_sixty_one_actions() {
    assert_eq!(
        KEYBIND_CATEGORIES,
        ["Movement", "Targeting", "Interface", "Pet", "Action Bar"]
    );
    assert_eq!(KEYBIND_ACTIONS.len(), 61);
    assert_eq!(
        KEYBIND_ACTIONS
            .iter()
            .filter(|action| action.category == "Action Bar")
            .count(),
        ACTION_BAR_SLOTS
    );
}

#[test]
fn registry_classifies_held_and_edge_actions() {
    assert_eq!(action_kind("forward"), Some(KeyBindingKind::Held));
    assert_eq!(action_kind("jump"), Some(KeyBindingKind::Held));
    assert_eq!(action_kind("emoteWheel"), Some(KeyBindingKind::Held));
    assert_eq!(action_kind("autorun"), Some(KeyBindingKind::Edge));
    assert_eq!(action_kind("target"), Some(KeyBindingKind::Edge));
    assert_eq!(action_kind("slot0"), Some(KeyBindingKind::Edge));
    assert_eq!(action_kind("nope"), None);
}

#[test]
fn registry_pins_high_risk_interface_and_action_bar_defaults() {
    for (id, category, kind, primary) in [
        ("discord", "Interface", KeyBindingKind::Edge, "KeyU"),
        ("valecup", "Interface", KeyBindingKind::Edge, "KeyY"),
        ("deeds", "Interface", KeyBindingKind::Edge, "Shift+KeyZ"),
        ("sheathe", "Interface", KeyBindingKind::Edge, "KeyZ"),
        ("slot12", "Action Bar", KeyBindingKind::Edge, "Numpad1"),
        (
            "slot22",
            "Action Bar",
            KeyBindingKind::Edge,
            "NumpadDecimal",
        ),
    ] {
        let action = keybind_action(id).expect("target action");
        assert_eq!(action.category, category, "{id}");
        assert_eq!(action.kind, kind, "{id}");
        assert_eq!(action.defaults[0], Some(primary), "{id}");
    }
}

#[test]
fn only_attack_move_allows_a_shared_binding() {
    assert!(action_allows_shared("attackMove"));
    for action in KEYBIND_ACTIONS {
        assert_eq!(
            action.allow_shared,
            action.id == "attackMove",
            "{}",
            action.id
        );
    }
}

#[test]
fn action_slots_zero_through_twenty_two_are_contiguous_and_unique() {
    for slot in 0..ACTION_BAR_SLOTS {
        let id = format!("slot{slot}");
        assert_eq!(
            KEYBIND_ACTIONS
                .iter()
                .filter(|action| action.id == id)
                .count(),
            1,
            "{id}"
        );
    }
}
