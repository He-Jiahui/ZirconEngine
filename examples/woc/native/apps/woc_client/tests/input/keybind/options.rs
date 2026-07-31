use woc_client::{
    KeyModifiers, KeybindCaptureOutcome, KeybindOptionsModel, KeybindOptionsNote, Keybinds,
};

#[test]
fn options_rows_follow_registry_category_and_action_order() {
    let model = KeybindOptionsModel::default();
    let bindings = Keybinds::default();
    let categories = model.categories(&bindings);

    assert_eq!(
        categories
            .iter()
            .map(|category| category.id)
            .collect::<Vec<_>>(),
        ["Movement", "Targeting", "Interface", "Pet", "Action Bar"]
    );
    assert_eq!(
        categories
            .iter()
            .map(|category| category.rows.len())
            .collect::<Vec<_>>(),
        [8, 4, 20, 5, 23]
    );
    assert_eq!(categories[0].rows[0].action_id, "forward");
    assert_eq!(categories[4].rows[0].action_id, "slot0");
    assert_eq!(categories[4].rows[22].action_id, "slot22");
}

#[test]
fn attack_move_row_is_visible_only_while_its_mode_is_enabled() {
    let mut model = KeybindOptionsModel::default();
    let bindings = Keybinds::default();
    assert!(model
        .categories(&bindings)
        .iter()
        .flat_map(|category| &category.rows)
        .all(|row| row.action_id != "attackMove"));
    assert!(!model.begin_capture("attackMove", 0));

    model.set_attack_move_enabled(true);
    let targeting = model
        .categories(&bindings)
        .into_iter()
        .find(|category| category.id == "Targeting")
        .expect("targeting category");
    assert_eq!(
        targeting
            .rows
            .iter()
            .map(|row| row.action_id)
            .collect::<Vec<_>>(),
        [
            "target",
            "targetFriendly",
            "targetFriendlyNext",
            "interact",
            "attackMove"
        ]
    );
    assert!(model.begin_capture("attackMove", 0));
}

#[test]
fn rows_expose_primary_hints_slot_labels_and_capture_state() {
    let mut model = KeybindOptionsModel::default();
    let mut bindings = Keybinds::default();
    bindings.clear("forward", 1);
    assert!(model.begin_capture("forward", 1));

    let categories = model.categories(&bindings);
    let movement = &categories[0];
    let forward = &movement.rows[0];
    assert_eq!(forward.label, "Move Forward");
    assert_eq!(forward.primary_hint.as_deref(), Some("W"));
    assert_eq!(forward.slots[0].label.as_deref(), Some("W"));
    assert_eq!(forward.slots[1].label, None);
    assert!(!forward.slots[0].capturing);
    assert!(forward.slots[1].capturing);
    assert_eq!(
        model.note(),
        &KeybindOptionsNote::Capturing {
            action_id: "forward"
        }
    );
}

#[test]
fn invalid_actions_or_slots_do_not_replace_an_active_capture() {
    let mut model = KeybindOptionsModel::default();
    assert!(model.begin_capture("jump", 0));
    assert!(!model.begin_capture("missing", 0));
    assert!(!model.begin_capture("jump", 2));
    assert_eq!(model.capture_target(), Some(("jump", 0)));
}

#[test]
fn repeat_and_bare_modifier_events_leave_capture_armed() {
    let mut model = KeybindOptionsModel::default();
    let mut bindings = Keybinds::default();
    assert!(model.begin_capture("slot0", 0));

    assert_eq!(
        model.handle_key_down(&mut bindings, "Digit1", KeyModifiers::default(), true),
        KeybindCaptureOutcome::RepeatIgnored
    );
    assert_eq!(
        model.handle_key_down(
            &mut bindings,
            "ShiftLeft",
            KeyModifiers {
                shift: true,
                ..KeyModifiers::default()
            },
            false
        ),
        KeybindCaptureOutcome::ModifierIgnored
    );
    assert_eq!(model.capture_target(), Some(("slot0", 0)));
}

#[test]
fn escape_cancels_capture_without_changing_the_binding() {
    let mut model = KeybindOptionsModel::default();
    let mut bindings = Keybinds::default();
    assert!(model.begin_capture("jump", 0));

    assert_eq!(
        model.handle_key_down(
            &mut bindings,
            "Escape",
            KeyModifiers {
                ctrl: true,
                ..KeyModifiers::default()
            },
            false
        ),
        KeybindCaptureOutcome::Cancelled
    );
    assert_eq!(model.capture_target(), None);
    assert_eq!(model.note(), &KeybindOptionsNote::Cancelled);
    assert_eq!(bindings.code_at("jump", 0), Some("Space"));
}

#[test]
fn captured_edge_chords_use_canonical_modifier_order() {
    let mut model = KeybindOptionsModel::default();
    let mut bindings = Keybinds::default();
    assert!(model.begin_capture("slot0", 1));

    assert_eq!(
        model.handle_key_down(
            &mut bindings,
            "Digit1",
            KeyModifiers {
                ctrl: true,
                alt: true,
                shift: true,
                meta: true,
            },
            false
        ),
        KeybindCaptureOutcome::Bound {
            action_id: "slot0",
            slot: 1,
            stored_combo: "Ctrl+Alt+Shift+Meta+Digit1".to_string(),
        }
    );
    assert_eq!(
        bindings.code_at("slot0", 1),
        Some("Ctrl+Alt+Shift+Meta+Digit1")
    );
    assert_eq!(
        model.note(),
        &KeybindOptionsNote::Bound {
            action_id: "slot0",
            key_label: "Ctrl+Alt+Shift+Meta+1".to_string(),
        }
    );
}

#[test]
fn captured_held_chords_confirm_the_normalized_stored_key() {
    let mut model = KeybindOptionsModel::default();
    let mut bindings = Keybinds::default();
    assert!(model.begin_capture("forward", 0));

    assert_eq!(
        model.handle_key_down(
            &mut bindings,
            "KeyW",
            KeyModifiers {
                shift: true,
                ..KeyModifiers::default()
            },
            false
        ),
        KeybindCaptureOutcome::Bound {
            action_id: "forward",
            slot: 0,
            stored_combo: "KeyW".to_string(),
        }
    );
    assert_eq!(bindings.code_at("forward", 0), Some("KeyW"));
    assert_eq!(
        model.note(),
        &KeybindOptionsNote::Bound {
            action_id: "forward",
            key_label: "W".to_string(),
        }
    );
}

#[test]
fn reset_and_leave_clear_capture_and_restore_panel_notes() {
    let mut model = KeybindOptionsModel::default();
    let mut bindings = Keybinds::default();
    assert!(bindings.bind("jump", 0, "F1"));
    assert!(model.begin_capture("slot0", 0));

    model.reset(&mut bindings);
    assert_eq!(bindings.code_at("jump", 0), Some("Space"));
    assert_eq!(model.capture_target(), None);
    assert_eq!(model.note(), &KeybindOptionsNote::Reset);

    assert!(model.begin_capture("jump", 0));
    model.leave_panel();
    assert_eq!(model.capture_target(), None);
    assert_eq!(model.note(), &KeybindOptionsNote::Help);
}
