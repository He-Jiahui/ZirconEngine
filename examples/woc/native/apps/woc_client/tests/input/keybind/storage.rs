use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};

use serde_json::Value;
use woc_client::{
    KeyModifiers, KeybindCaptureOutcome, KeybindOptionsModel, PreferenceStorage, StoredKeybinds,
    LEGACY_KEYBIND_STORAGE_KEY,
};

#[derive(Clone, Default)]
struct MemoryStorage {
    values: Rc<RefCell<BTreeMap<String, String>>>,
    fail_reads: Rc<Cell<bool>>,
    fail_writes: Rc<Cell<bool>>,
}

impl MemoryStorage {
    fn insert(&self, key: &str, value: &str) {
        self.values
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
    }

    fn get(&self, key: &str) -> Option<String> {
        self.values.borrow().get(key).cloned()
    }
}

impl PreferenceStorage for MemoryStorage {
    type Error = ();

    fn read(&self, key: &str) -> Result<Option<String>, Self::Error> {
        if self.fail_reads.get() {
            Err(())
        } else {
            Ok(self.get(key))
        }
    }

    fn write(&self, key: &str, value: &str) -> Result<(), Self::Error> {
        if self.fail_writes.get() {
            Err(())
        } else {
            self.insert(key, value);
            Ok(())
        }
    }
}

#[test]
fn valid_scoped_profile_wins_over_the_legacy_seed() {
    let storage = MemoryStorage::default();
    storage.insert(LEGACY_KEYBIND_STORAGE_KEY, r#"{"jump":["KeyJ",null]}"#);
    storage.insert("woc_keybinds:char:alice", r#"{"jump":["KeyK",null]}"#);

    let bindings = StoredKeybinds::new("char:alice", storage);
    assert_eq!(bindings.action_for_combo("KeyK"), Some("jump"));
    assert_eq!(
        bindings.action_for_combo("KeyJ"),
        Some("targetFriendlyNext")
    );
}

#[test]
fn missing_corrupt_or_non_object_scoped_values_seed_from_legacy() {
    let storage = MemoryStorage::default();
    storage.insert(LEGACY_KEYBIND_STORAGE_KEY, r#"{"jump":["KeyZ",null]}"#);
    for (scope, raw) in [
        ("char:missing", None),
        ("char:corrupt", Some("{not valid json")),
        ("char:array", Some(r#"["garbage"]"#)),
        ("char:scalar", Some("42")),
        ("char:null", Some("null")),
    ] {
        if let Some(raw) = raw {
            storage.insert(&format!("woc_keybinds:{scope}"), raw);
        }
        let bindings = StoredKeybinds::new(scope, storage.clone());
        assert_eq!(bindings.action_for_combo("KeyZ"), Some("jump"), "{scope}");
    }
}

#[test]
fn valid_empty_scoped_object_blocks_legacy_and_keeps_defaults() {
    let storage = MemoryStorage::default();
    storage.insert(LEGACY_KEYBIND_STORAGE_KEY, r#"{"jump":["KeyJ",null]}"#);
    storage.insert("woc_keybinds:char:alice", "{}");

    let bindings = StoredKeybinds::new("char:alice", storage);
    assert_eq!(bindings.code_at("jump", 0), Some("Space"));
    assert_eq!(
        bindings.action_for_combo("KeyJ"),
        Some("targetFriendlyNext")
    );
}

#[test]
fn json_arrays_preserve_explicit_unbinds_and_malformed_entries_keep_defaults() {
    let storage = MemoryStorage::default();
    storage.insert(
        "woc_keybinds:char:alice",
        r#"{
            "jump": [],
            "slot0": ["Escape", 42, "KeyR"],
            "slot1": "not-an-array"
        }"#,
    );

    let bindings = StoredKeybinds::new("char:alice", storage);
    assert_eq!(bindings.code_at("jump", 0), None);
    assert_eq!(bindings.code_at("slot0", 0), None);
    assert_eq!(bindings.code_at("slot0", 1), None);
    assert_eq!(bindings.code_at("slot1", 0), Some("Digit2"));
    assert_eq!(bindings.action_for_combo("KeyR"), Some("autorun"));
}

#[test]
fn successful_mutations_save_the_complete_scoped_profile_without_touching_legacy() {
    let storage = MemoryStorage::default();
    let legacy = r#"{"jump":["KeyJ",null]}"#;
    storage.insert(LEGACY_KEYBIND_STORAGE_KEY, legacy);
    let mut bindings = StoredKeybinds::new("char:alice", storage.clone());

    assert!(bindings.bind("jump", 0, "KeyK"));
    let raw = storage
        .get("woc_keybinds:char:alice")
        .expect("scoped profile written");
    let encoded: Value = serde_json::from_str(&raw).expect("stored JSON object");
    assert_eq!(encoded.as_object().map(|object| object.len()), Some(61));
    assert_eq!(encoded["jump"], serde_json::json!(["KeyK", null]));
    assert_eq!(
        storage.get(LEGACY_KEYBIND_STORAGE_KEY).as_deref(),
        Some(legacy)
    );

    let reloaded = StoredKeybinds::new("char:alice", storage);
    assert_eq!(reloaded.action_for_combo("KeyK"), Some("jump"));
    assert_eq!(reloaded.action_for_combo("KeyJ"), None);
}

#[test]
fn clear_and_reset_persist_to_the_selected_scope() {
    let storage = MemoryStorage::default();
    let mut bindings = StoredKeybinds::new("offline:warrior:Aldric", storage.clone());

    bindings.clear("jump", 0);
    assert_eq!(
        StoredKeybinds::new("offline:warrior:Aldric", storage.clone()).code_at("jump", 0),
        None
    );

    bindings.reset();
    let reloaded = StoredKeybinds::new("offline:warrior:Aldric", storage);
    assert_eq!(reloaded.code_at("jump", 0), Some("Space"));
    assert_eq!(reloaded.action_for_combo("KeyZ"), Some("sheathe"));
}

#[test]
fn empty_scope_reads_and_writes_the_legacy_global_key() {
    let storage = MemoryStorage::default();
    let mut bindings = StoredKeybinds::new("", storage.clone());
    assert!(bindings.bind("jump", 0, "F1"));

    assert!(storage.get(LEGACY_KEYBIND_STORAGE_KEY).is_some());
    assert_eq!(
        StoredKeybinds::new("", storage).action_for_combo("F1"),
        Some("jump")
    );
}

#[test]
fn character_scopes_remain_independent() {
    let storage = MemoryStorage::default();
    let mut alice = StoredKeybinds::new("char:alice", storage.clone());
    let mut bob = StoredKeybinds::new("char:bob", storage.clone());
    assert!(alice.bind("jump", 0, "Semicolon"));
    assert!(bob.bind("jump", 0, "F1"));

    assert_eq!(
        StoredKeybinds::new("char:alice", storage.clone()).action_for_combo("Semicolon"),
        Some("jump")
    );
    assert_eq!(
        StoredKeybinds::new("char:bob", storage).action_for_combo("F1"),
        Some("jump")
    );
}

#[test]
fn unavailable_storage_degrades_to_defaults_and_keeps_mutations_in_memory() {
    let storage = MemoryStorage::default();
    storage.fail_reads.set(true);
    storage.fail_writes.set(true);
    let mut bindings = StoredKeybinds::new("char:alice", storage.clone());

    assert_eq!(bindings.code_at("jump", 0), Some("Space"));
    assert!(bindings.bind("jump", 0, "F1"));
    assert_eq!(bindings.action_for_combo("F1"), Some("jump"));
    assert!(storage.get("woc_keybinds:char:alice").is_none());
}

#[test]
fn options_capture_persists_through_the_stored_keybind_contract() {
    let storage = MemoryStorage::default();
    let mut bindings = StoredKeybinds::new("char:alice", storage.clone());
    let mut options = KeybindOptionsModel::default();
    assert!(options.begin_capture("jump", 0));

    assert_eq!(
        options.handle_key_down(&mut bindings, "F2", KeyModifiers::default(), false),
        KeybindCaptureOutcome::Bound {
            action_id: "jump",
            slot: 0,
            stored_combo: "F2".to_string(),
        }
    );
    assert_eq!(
        StoredKeybinds::new("char:alice", storage).action_for_combo("F2"),
        Some("jump")
    );
}
