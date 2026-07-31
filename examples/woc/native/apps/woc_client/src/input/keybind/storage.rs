use std::ops::Deref;

use serde_json::{Map, Value};

use super::{
    bindings::{Keybinds, BINDING_SLOTS},
    combo::{keybind_storage_key, LEGACY_KEYBIND_STORAGE_KEY},
    profile::{StoredBindingSlot, StoredKeybindProfile, StoredKeybindValue},
    registry::{keybind_action, KEYBIND_ACTIONS},
};
use crate::preferences::PreferenceStorage;

pub struct StoredKeybinds<S> {
    storage: S,
    storage_key: String,
    bindings: Keybinds,
}

impl<S> StoredKeybinds<S>
where
    S: PreferenceStorage,
{
    pub fn new(scope: &str, storage: S) -> Self {
        let storage_key = keybind_storage_key(scope);
        let profile = read_profile(&storage, &storage_key).or_else(|| {
            (!scope.is_empty())
                .then(|| read_profile(&storage, LEGACY_KEYBIND_STORAGE_KEY))
                .flatten()
        });
        let bindings = profile.map_or_else(Keybinds::default, Keybinds::from_stored_profile);
        Self {
            storage,
            storage_key,
            bindings,
        }
    }

    pub fn storage_key(&self) -> &str {
        &self.storage_key
    }

    pub fn bind(&mut self, id: &str, slot: usize, combo: &str) -> bool {
        if !self.bindings.bind(id, slot, combo) {
            return false;
        }
        self.save();
        true
    }

    pub fn clear(&mut self, id: &str, slot: usize) {
        if keybind_action(id).is_none() || slot >= BINDING_SLOTS {
            return;
        }
        self.bindings.clear(id, slot);
        self.save();
    }

    pub fn reset(&mut self) {
        self.bindings.reset();
        self.save();
    }

    pub fn into_storage(self) -> S {
        self.storage
    }

    fn save(&self) {
        let encoded = encode_stored_keybinds(&self.bindings);
        let _ = self.storage.write(&self.storage_key, &encoded);
    }
}

impl<S> Deref for StoredKeybinds<S> {
    type Target = Keybinds;

    fn deref(&self) -> &Self::Target {
        &self.bindings
    }
}

pub fn decode_stored_keybind_profile(raw: &str) -> Option<StoredKeybindProfile> {
    let Value::Object(entries) = serde_json::from_str(raw).ok()? else {
        return None;
    };
    Some(StoredKeybindProfile::from_entries(
        entries
            .into_iter()
            .map(|(id, value)| (id, decode_stored_value(value))),
    ))
}

pub fn encode_stored_keybinds(bindings: &Keybinds) -> String {
    let mut entries = Map::new();
    for action in KEYBIND_ACTIONS {
        let slots = (0..BINDING_SLOTS)
            .map(|slot| {
                bindings
                    .code_at(action.id, slot)
                    .map_or(Value::Null, |combo| Value::String(combo.to_string()))
            })
            .collect();
        entries.insert(action.id.to_string(), Value::Array(slots));
    }
    Value::Object(entries).to_string()
}

fn read_profile<S>(storage: &S, key: &str) -> Option<StoredKeybindProfile>
where
    S: PreferenceStorage,
{
    let raw = storage.read(key).ok()??;
    decode_stored_keybind_profile(&raw)
}

fn decode_stored_value(value: Value) -> StoredKeybindValue {
    match value {
        Value::Array(slots) => StoredKeybindValue::Slots(
            slots
                .into_iter()
                .map(|slot| match slot {
                    Value::String(combo) => StoredBindingSlot::Combo(combo),
                    Value::Null => StoredBindingSlot::Empty,
                    _ => StoredBindingSlot::Invalid,
                })
                .collect(),
        ),
        _ => StoredKeybindValue::Malformed,
    }
}
