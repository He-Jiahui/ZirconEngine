use woc_client::{
    repair_stored_bindings, Keybinds, StoredBindingSlot, StoredKeybindProfile, StoredKeybindValue,
};

fn combo(value: &str) -> StoredBindingSlot {
    StoredBindingSlot::Combo(value.to_string())
}

fn slots(values: &[Option<&str>]) -> StoredKeybindValue {
    StoredKeybindValue::Slots(
        values
            .iter()
            .map(|value| value.map_or(StoredBindingSlot::Empty, combo))
            .collect(),
    )
}

fn profile(entries: &[(&str, StoredKeybindValue)]) -> StoredKeybindProfile {
    StoredKeybindProfile::from_entries(
        entries
            .iter()
            .map(|(id, value)| ((*id).to_string(), value.clone())),
    )
}

#[test]
fn repair_drops_only_the_complete_qe_strafe_signature() {
    let mut stored = profile(&[
        ("strafeLeft", slots(&[None, None])),
        ("strafeRight", slots(&[None, None])),
        ("slot10", slots(&[Some("KeyQ"), Some("Minus")])),
        ("slot11", slots(&[Some("KeyE"), Some("Equal")])),
        ("jump", slots(&[Some("KeyZ"), None])),
    ]);

    repair_stored_bindings(&mut stored);

    for id in ["strafeLeft", "strafeRight", "slot10", "slot11"] {
        assert!(!stored.contains(id), "{id}");
    }
    assert_eq!(stored.get("jump"), Some(&slots(&[Some("KeyZ"), None])));
}

#[test]
fn repair_keeps_deliberate_swaps_and_partial_strafe_signatures() {
    let mut swapped = profile(&[
        ("slot0", slots(&[Some("Digit2")])),
        ("slot1", slots(&[Some("Digit1")])),
    ]);
    let before = swapped.clone();
    repair_stored_bindings(&mut swapped);
    assert_eq!(swapped, before);

    let mut partial = profile(&[
        ("slot10", slots(&[Some("KeyQ"), Some("Minus")])),
        ("slot11", slots(&[Some("KeyE"), Some("Equal")])),
        ("strafeLeft", slots(&[Some("KeyQ")])),
        ("strafeRight", slots(&[None, None])),
    ]);
    let before = partial.clone();
    repair_stored_bindings(&mut partial);
    assert_eq!(partial, before);
}

#[test]
fn repair_drops_empty_meters_when_friendly_target_resolves_to_key_h() {
    let mut absent_target = profile(&[("meters", slots(&[None, None]))]);
    repair_stored_bindings(&mut absent_target);
    assert!(!absent_target.contains("meters"));

    let mut explicit_target = profile(&[
        ("meters", slots(&[None, None])),
        ("targetFriendly", slots(&[Some("KeyH"), None])),
    ]);
    repair_stored_bindings(&mut explicit_target);
    assert!(!explicit_target.contains("meters"));
}

#[test]
fn repair_keeps_empty_meters_when_friendly_target_moved_off_key_h() {
    let mut stored = profile(&[
        ("meters", slots(&[None, None])),
        ("targetFriendly", slots(&[Some("KeyG"), None])),
    ]);
    repair_stored_bindings(&mut stored);
    assert_eq!(stored.get("meters"), Some(&slots(&[None, None])));
}

#[test]
fn repair_keeps_deliberately_bound_meters() {
    let mut stored = profile(&[("meters", slots(&[Some("KeyH"), None]))]);
    let before = stored.clone();
    repair_stored_bindings(&mut stored);
    assert_eq!(stored, before);
}

#[test]
fn load_applies_explicit_entries_and_keeps_missing_action_defaults() {
    let bindings = Keybinds::from_stored_profile(profile(&[
        ("slot0", slots(&[Some("KeyR"), None])),
        ("jump", slots(&[Some("KeyJ"), None])),
        ("unknownAction", slots(&[Some("F12")])),
    ]));

    assert_eq!(bindings.action_for_combo("KeyR"), Some("slot0"));
    assert_eq!(bindings.action_for_combo("KeyJ"), Some("jump"));
    assert_eq!(bindings.action_for_combo("KeyW"), Some("forward"));
    assert_eq!(bindings.action_for_combo("Tab"), Some("target"));
    assert_eq!(bindings.action_for_combo("KeyZ"), Some("sheathe"));
    assert_eq!(bindings.action_for_combo("F12"), None);
}

#[test]
fn explicit_binding_claims_remove_colliding_retained_defaults() {
    let friendly_claim =
        Keybinds::from_stored_profile(profile(&[("jump", slots(&[Some("KeyH"), None]))]));
    assert_eq!(friendly_claim.action_for_combo("KeyH"), Some("jump"));
    assert_eq!(friendly_claim.code_at("targetFriendly", 0), None);

    let jump_claim =
        Keybinds::from_stored_profile(profile(&[("slot1", slots(&[Some("Space"), None]))]));
    assert_eq!(jump_claim.action_for_combo("Space"), Some("slot1"));
    assert_eq!(jump_claim.code_at("jump", 0), None);
}

#[test]
fn duplicate_explicit_codes_are_won_in_registry_order() {
    let bindings = Keybinds::from_stored_profile(profile(&[
        ("slot1", slots(&[Some("KeyR"), None])),
        ("slot0", slots(&[Some("KeyR"), Some("KeyR")])),
    ]));

    assert_eq!(bindings.action_for_combo("KeyR"), Some("slot0"));
    assert_eq!(bindings.code_at("slot0", 0), Some("KeyR"));
    assert_eq!(bindings.code_at("slot0", 1), None);
    assert_eq!(bindings.code_at("slot1", 0), None);
}

#[test]
fn explicit_empty_or_invalid_arrays_unbind_without_restoring_defaults() {
    let bindings = Keybinds::from_stored_profile(profile(&[
        ("jump", StoredKeybindValue::Slots(Vec::new())),
        (
            "slot0",
            StoredKeybindValue::Slots(vec![
                combo("Escape"),
                StoredBindingSlot::Invalid,
                combo("KeyR"),
            ]),
        ),
    ]));

    assert_eq!(bindings.code_at("jump", 0), None);
    assert_eq!(bindings.action_for_combo("Space"), None);
    assert_eq!(bindings.code_at("slot0", 0), None);
    assert_eq!(bindings.code_at("slot0", 1), None);
    assert_eq!(bindings.action_for_combo("KeyR"), Some("autorun"));
}

#[test]
fn malformed_non_array_entries_keep_defaults() {
    let bindings = Keybinds::from_stored_profile(profile(&[
        ("jump", StoredKeybindValue::Malformed),
        ("slot0", StoredKeybindValue::Malformed),
    ]));

    assert_eq!(bindings.code_at("jump", 0), Some("Space"));
    assert_eq!(bindings.code_at("slot0", 0), Some("Digit1"));
}

#[test]
fn shared_explicit_bindings_survive_claimed_codes_and_never_claim_them() {
    let bindings = Keybinds::from_stored_profile(profile(&[
        ("turnLeft", slots(&[Some("KeyA"), None])),
        ("attackMove", slots(&[Some("KeyA"), None])),
        ("bags", slots(&[Some("KeyA"), None])),
    ]));

    assert_eq!(bindings.code_at("turnLeft", 0), Some("KeyA"));
    assert_eq!(bindings.code_at("attackMove", 0), Some("KeyA"));
    assert_eq!(bindings.code_at("bags", 0), None);
}

#[test]
fn load_repairs_known_signatures_before_applying_stored_entries() {
    let bindings = Keybinds::from_stored_profile(profile(&[
        ("strafeLeft", slots(&[None, None])),
        ("strafeRight", slots(&[None, None])),
        ("slot10", slots(&[Some("KeyQ"), Some("Minus")])),
        ("slot11", slots(&[Some("KeyE"), Some("Equal")])),
        ("meters", slots(&[None, None])),
    ]));

    assert_eq!(bindings.code_at("strafeLeft", 0), Some("KeyQ"));
    assert_eq!(bindings.code_at("strafeRight", 0), Some("KeyE"));
    assert_eq!(bindings.code_at("slot10", 0), Some("Minus"));
    assert_eq!(bindings.code_at("slot11", 0), Some("Equal"));
    assert_eq!(bindings.code_at("meters", 0), Some("Shift+KeyH"));
}
