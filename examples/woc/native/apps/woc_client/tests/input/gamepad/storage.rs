use serde_json::Value;
use woc_client::{
    gamepad_button, StoredGamepadBindings, BINDABLE_GAMEPAD_BUTTONS, GAMEPAD_NONE_ACTION,
    GAMEPAD_STORAGE_KEY,
};

use crate::preference_storage_support::MemoryPreferenceStorage as MemoryStorage;

#[test]
fn fresh_backend_gamepad_bindings_load_after_the_cold_read_completes() {
    let storage = MemoryStorage::default();
    storage.seed_persisted(GAMEPAD_STORAGE_KEY, r#"{"0":"slot8"}"#);
    storage.block_read(GAMEPAD_STORAGE_KEY);

    let mut bindings = StoredGamepadBindings::new(storage.clone());
    storage.wait_until_read_started(GAMEPAD_STORAGE_KEY);
    assert_eq!(bindings.action_for(gamepad_button::A), "jump");
    assert!(!bindings.refresh_from_storage());
    storage.release_read(GAMEPAD_STORAGE_KEY);
    storage.wait_until_loaded(GAMEPAD_STORAGE_KEY);

    assert!(bindings.refresh_from_storage());
    assert_eq!(bindings.action_for(gamepad_button::A), "slot8");
}

#[test]
fn stored_object_strings_override_defaults_only_for_bindable_buttons() {
    let storage = MemoryStorage::default();
    storage.insert(
        GAMEPAD_STORAGE_KEY,
        r#"{
            "0": "customAction",
            "1": "slot7",
            "2": 42,
            "16": "slot8",
            "99": "slot9"
        }"#,
    );

    let bindings = StoredGamepadBindings::new(storage);
    assert_eq!(bindings.action_for(gamepad_button::A), "customAction");
    assert_eq!(bindings.action_for(gamepad_button::B), "slot7");
    assert_eq!(bindings.action_for(gamepad_button::X), "slot0");
    assert_eq!(
        bindings.action_for(gamepad_button::GUIDE),
        GAMEPAD_NONE_ACTION
    );
}

#[test]
fn corrupt_null_or_scalar_storage_keeps_the_complete_default_layout() {
    for raw in ["{invalid", "null", "false", "42", r#""text""#] {
        let storage = MemoryStorage::default();
        storage.insert(GAMEPAD_STORAGE_KEY, raw);
        let bindings = StoredGamepadBindings::new(storage);
        assert_eq!(bindings.action_for(gamepad_button::A), "jump", "{raw}");
        assert_eq!(bindings.entries().len(), 16, "{raw}");
    }
}

#[test]
fn stored_arrays_and_javascript_numeric_property_keys_are_supported() {
    let array_storage = MemoryStorage::default();
    array_storage.insert(GAMEPAD_STORAGE_KEY, r#"["slot4","slot5"]"#);
    let array_bindings = StoredGamepadBindings::new(array_storage);
    assert_eq!(array_bindings.action_for(gamepad_button::A), "slot4");
    assert_eq!(array_bindings.action_for(gamepad_button::B), "slot5");

    let object_storage = MemoryStorage::default();
    object_storage.insert(
        GAMEPAD_STORAGE_KEY,
        r#"{" 0 ":"slot6","0x1":"slot7","1.0":"slot8","nope":"slot9"}"#,
    );
    let object_bindings = StoredGamepadBindings::new(object_storage);
    assert_eq!(object_bindings.action_for(gamepad_button::A), "slot6");
    assert_eq!(object_bindings.action_for(gamepad_button::B), "slot8");
}

#[test]
fn bind_persists_all_current_entries_and_allows_duplicate_actions() {
    let storage = MemoryStorage::default();
    let mut bindings = StoredGamepadBindings::new(storage.clone());
    bindings.bind(gamepad_button::A, "slot1");
    bindings.bind(gamepad_button::B, "slot1");

    let raw = storage.get(GAMEPAD_STORAGE_KEY).expect("gamepad JSON");
    let encoded: Value = serde_json::from_str(&raw).expect("stored JSON object");
    assert_eq!(encoded.as_object().map(|object| object.len()), Some(16));
    assert_eq!(encoded["0"], "slot1");
    assert_eq!(encoded["1"], "slot1");
    let reloaded = StoredGamepadBindings::new(storage);
    assert_eq!(reloaded.action_for(gamepad_button::A), "slot1");
    assert_eq!(reloaded.action_for(gamepad_button::B), "slot1");
}

#[test]
fn clearing_deletes_the_stored_entry_and_the_default_returns_after_reload() {
    let storage = MemoryStorage::default();
    let mut bindings = StoredGamepadBindings::new(storage.clone());
    bindings.bind(gamepad_button::A, GAMEPAD_NONE_ACTION);
    assert_eq!(bindings.action_for(gamepad_button::A), GAMEPAD_NONE_ACTION);

    let raw = storage.get(GAMEPAD_STORAGE_KEY).expect("gamepad JSON");
    let encoded: Value = serde_json::from_str(&raw).expect("stored JSON object");
    assert_eq!(encoded.as_object().map(|object| object.len()), Some(15));
    assert!(encoded.get("0").is_none());
    assert_eq!(
        StoredGamepadBindings::new(storage).action_for(gamepad_button::A),
        "jump"
    );
}

#[test]
fn reset_persists_the_exact_complete_default_layout() {
    let storage = MemoryStorage::default();
    let mut bindings = StoredGamepadBindings::new(storage.clone());
    bindings.bind(gamepad_button::A, "slot8");
    bindings.reset();

    let raw = storage.get(GAMEPAD_STORAGE_KEY).expect("gamepad JSON");
    let encoded: Value = serde_json::from_str(&raw).expect("stored JSON object");
    assert_eq!(encoded.as_object().map(|object| object.len()), Some(16));
    assert_eq!(encoded["0"], "jump");
    assert_eq!(encoded["9"], "escape");
}

#[test]
fn non_bindable_buttons_do_not_trigger_storage_writes() {
    let storage = MemoryStorage::default();
    let mut bindings = StoredGamepadBindings::new(storage.clone());
    bindings.bind(gamepad_button::GUIDE, "slot8");
    bindings.bind(99, "slot9");
    assert!(storage.get(GAMEPAD_STORAGE_KEY).is_none());
}

#[test]
fn unavailable_storage_degrades_to_defaults_and_keeps_session_mutations() {
    let storage = MemoryStorage::default();
    storage.set_fail_reads(true);
    storage.set_fail_writes(true);
    let mut bindings = StoredGamepadBindings::new(storage.clone());
    assert_eq!(bindings.entries().len(), BINDABLE_GAMEPAD_BUTTONS.len());

    bindings.bind(gamepad_button::A, "slot8");
    assert_eq!(bindings.action_for(gamepad_button::A), "slot8");
    assert!(storage.persisted(GAMEPAD_STORAGE_KEY).is_none());
}
