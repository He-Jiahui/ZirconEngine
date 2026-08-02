use std::ops::Deref;

use serde_json::{Map, Value};

use super::{
    bindings::{Keybinds, BINDING_SLOTS},
    combo::{keybind_storage_key, LEGACY_KEYBIND_STORAGE_KEY},
    profile::{StoredBindingSlot, StoredKeybindProfile, StoredKeybindValue},
    registry::{keybind_action, KEYBIND_ACTIONS},
};
use crate::preferences::{read_preference_text, submit_preference_text, PreferenceRead};
use zircon_runtime::core::framework::platform::{PreferenceMutationSubmission, PreferenceStorage};

pub struct StoredKeybinds<S> {
    storage: S,
    storage_key: String,
    use_legacy_fallback: bool,
    bindings: Keybinds,
    last_persistence_submission: Option<PreferenceMutationSubmission>,
}

impl<S> StoredKeybinds<S>
where
    S: AsRef<dyn PreferenceStorage>,
{
    pub fn new(scope: &str, storage: S) -> Self {
        let storage_key = keybind_storage_key(scope);
        let use_legacy_fallback = !scope.is_empty();
        let bindings = read_bindings(&storage, &storage_key, use_legacy_fallback)
            .into_ready()
            .unwrap_or_default();
        Self {
            storage,
            storage_key,
            use_legacy_fallback,
            bindings,
            last_persistence_submission: None,
        }
    }

    pub fn storage_key(&self) -> &str {
        &self.storage_key
    }

    pub fn refresh_from_storage(&mut self) -> bool {
        let Some(bindings) =
            read_bindings(&self.storage, &self.storage_key, self.use_legacy_fallback).into_ready()
        else {
            return false;
        };
        self.bindings = bindings;
        true
    }

    pub fn take_persistence_submission(&mut self) -> Option<PreferenceMutationSubmission> {
        self.last_persistence_submission.take()
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

    fn save(&mut self) {
        let encoded = encode_stored_keybinds(&self.bindings);
        self.last_persistence_submission =
            submit_preference_text(self.storage.as_ref(), &self.storage_key, &encoded);
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

fn read_bindings<S>(
    storage: &S,
    storage_key: &str,
    use_legacy_fallback: bool,
) -> PreferenceRead<Keybinds>
where
    S: AsRef<dyn PreferenceStorage>,
{
    match read_profile(storage, storage_key) {
        PreferenceRead::Pending => PreferenceRead::Pending,
        PreferenceRead::Ready(Some(profile)) => {
            PreferenceRead::Ready(Keybinds::from_stored_profile(profile))
        }
        PreferenceRead::Ready(None) if use_legacy_fallback => {
            read_profile(storage, LEGACY_KEYBIND_STORAGE_KEY).map(|profile| {
                profile.map_or_else(Keybinds::default, Keybinds::from_stored_profile)
            })
        }
        PreferenceRead::Ready(None) => PreferenceRead::Ready(Keybinds::default()),
    }
}

fn read_profile<S>(storage: &S, key: &str) -> PreferenceRead<Option<StoredKeybindProfile>>
where
    S: AsRef<dyn PreferenceStorage>,
{
    read_preference_text(storage.as_ref(), key)
        .map(|raw| raw.as_deref().and_then(decode_stored_keybind_profile))
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
