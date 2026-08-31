use woc_client::{KeyBindingKind, Keybinds};

#[test]
fn defaults_resolve_movement_interface_and_action_bar_bindings() {
    let bindings = Keybinds::default();
    for (combo, action) in [
        ("KeyW", "forward"),
        ("ArrowUp", "forward"),
        ("KeyD", "turnRight"),
        ("Space", "jump"),
        ("Tab", "target"),
        ("KeyB", "bags"),
        ("KeyX", "emoteWheel"),
        ("Digit1", "slot0"),
        ("Equal", "slot11"),
        ("KeyH", "targetFriendly"),
        ("KeyJ", "targetFriendlyNext"),
        ("KeyU", "discord"),
        ("KeyT", "crafting"),
        ("KeyY", "valecup"),
        ("KeyZ", "sheathe"),
        ("Shift+KeyZ", "deeds"),
    ] {
        assert_eq!(bindings.action_for_combo(combo), Some(action), "{combo}");
    }
}

#[test]
fn defaults_expose_both_slots_and_target_keycap_labels() {
    let bindings = Keybinds::default();
    assert_eq!(bindings.code_at("forward", 0), Some("KeyW"));
    assert_eq!(bindings.code_at("forward", 1), Some("ArrowUp"));
    assert_eq!(
        bindings.codes_for_action("forward"),
        vec!["KeyW", "ArrowUp"]
    );
    assert_eq!(bindings.primary_label("slot0"), "1");
    assert_eq!(bindings.label_at("forward", 1), "↑");
}

#[test]
fn rebind_moves_a_combo_across_categories_and_preserves_other_slots() {
    let mut bindings = Keybinds::default();
    assert!(bindings.bind("bags", 0, "KeyW"));
    assert_eq!(bindings.action_for_combo("KeyW"), Some("bags"));
    assert_eq!(bindings.code_at("forward", 0), None);
    assert_eq!(bindings.action_for_combo("ArrowUp"), Some("forward"));

    assert!(bindings.bind("slot1", 1, "Semicolon"));
    assert_eq!(bindings.code_at("slot1", 0), Some("Digit2"));
    assert_eq!(bindings.code_at("slot1", 1), Some("Semicolon"));
}

#[test]
fn rebind_allows_the_same_combo_in_both_slots_of_one_action() {
    let mut bindings = Keybinds::default();
    assert!(bindings.bind("slot1", 1, "Digit2"));
    assert_eq!(bindings.code_at("slot1", 0), Some("Digit2"));
    assert_eq!(bindings.code_at("slot1", 1), Some("Digit2"));
}

#[test]
fn reserved_or_unknown_bind_requests_are_atomic() {
    let mut bindings = Keybinds::default();
    assert!(!bindings.bind("jump", 0, "Escape"));
    assert!(!bindings.bind("missing", 0, "KeyQ"));
    assert!(!bindings.bind("jump", 2, "KeyQ"));
    assert_eq!(bindings.code_at("jump", 0), Some("Space"));
    assert_eq!(bindings.action_for_combo("KeyQ"), Some("strafeLeft"));
}

#[test]
fn held_actions_strip_modifiers_but_edge_actions_keep_chords() {
    let mut bindings = Keybinds::default();
    assert!(bindings.bind("forward", 0, "Shift+KeyW"));
    assert_eq!(bindings.code_at("forward", 0), Some("KeyW"));
    assert_eq!(bindings.held_action_for_code("KeyW"), Some("forward"));

    assert!(bindings.bind("slot0", 0, "Shift+Digit1"));
    assert_eq!(bindings.code_at("slot0", 0), Some("Shift+Digit1"));
    assert_eq!(
        bindings.edge_action_for_combo("Shift+Digit1"),
        Some("slot0")
    );
    assert_eq!(bindings.edge_action_for_combo("Digit1"), None);
    assert_eq!(bindings.kind("forward"), Some(KeyBindingKind::Held));
}

#[test]
fn attack_move_shares_a_without_stealing_or_being_stolen() {
    let mut bindings = Keybinds::default();
    assert_eq!(bindings.code_at("attackMove", 0), Some("KeyA"));
    assert_eq!(bindings.code_at("turnLeft", 0), Some("KeyA"));
    assert_eq!(bindings.action_for_combo("KeyA"), Some("turnLeft"));

    assert!(bindings.bind("attackMove", 0, "KeyA"));
    assert_eq!(bindings.code_at("turnLeft", 0), Some("KeyA"));
    assert!(bindings.bind("bags", 0, "KeyA"));
    assert_eq!(bindings.code_at("attackMove", 0), Some("KeyA"));
    assert_eq!(bindings.code_at("turnLeft", 0), None);
    assert_eq!(bindings.action_for_combo("KeyA"), Some("attackMove"));
}

#[test]
fn lookup_kinds_preserve_shared_precedence_after_mutations() {
    let mut bindings = Keybinds::default();
    assert_eq!(bindings.action_for_combo("KeyA"), Some("turnLeft"));
    assert_eq!(bindings.held_action_for_code("KeyA"), Some("turnLeft"));
    assert_eq!(bindings.edge_action_for_combo("KeyA"), Some("attackMove"));

    assert!(bindings.bind("bags", 0, "KeyA"));
    assert_eq!(bindings.action_for_combo("KeyA"), Some("attackMove"));
    assert_eq!(bindings.held_action_for_code("KeyA"), None);
    assert_eq!(bindings.edge_action_for_combo("KeyA"), Some("attackMove"));

    bindings.clear("attackMove", 0);
    assert_eq!(bindings.action_for_combo("KeyA"), Some("bags"));
    assert_eq!(bindings.edge_action_for_combo("KeyA"), Some("bags"));

    bindings.reset();
    assert_eq!(bindings.action_for_combo("KeyA"), Some("turnLeft"));
    assert_eq!(bindings.held_action_for_code("KeyA"), Some("turnLeft"));
    assert_eq!(bindings.edge_action_for_combo("KeyA"), Some("attackMove"));
}

#[test]
fn clear_and_reset_restore_exact_defaults() {
    let mut bindings = Keybinds::default();
    bindings.clear("forward", 1);
    assert_eq!(bindings.codes_for_action("forward"), vec!["KeyW"]);
    assert_eq!(bindings.action_for_combo("ArrowUp"), None);

    assert!(bindings.bind("slot0", 0, "KeyR"));
    bindings.clear("jump", 0);
    bindings.reset();
    assert_eq!(bindings.action_for_combo("Digit1"), Some("slot0"));
    assert_eq!(bindings.action_for_combo("Space"), Some("jump"));
}
