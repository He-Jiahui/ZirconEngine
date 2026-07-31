use std::{cmp::Ordering, ops::Deref};

use serde_json::{Map, Value};

use super::{
    bindings::{is_bindable_gamepad_button, GamepadBindingEntry, GamepadBindings},
    layout::BINDABLE_GAMEPAD_BUTTONS,
};
use crate::preferences::PreferenceStorage;

pub const GAMEPAD_STORAGE_KEY: &str = "woc_gamepad";

pub struct StoredGamepadBindings<S> {
    storage: S,
    bindings: GamepadBindings,
}

impl<S> StoredGamepadBindings<S>
where
    S: PreferenceStorage,
{
    pub fn new(storage: S) -> Self {
        let bindings = read_stored_entries(&storage)
            .map(GamepadBindings::from_stored)
            .unwrap_or_default();
        Self { storage, bindings }
    }

    pub fn bind(&mut self, button: usize, action: impl Into<String>) {
        if !is_bindable_gamepad_button(button) {
            return;
        }
        self.bindings.bind(button, action);
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
        let encoded = encode_stored_gamepad_bindings(&self.bindings);
        let _ = self.storage.write(GAMEPAD_STORAGE_KEY, &encoded);
    }
}

impl<S> Deref for StoredGamepadBindings<S> {
    type Target = GamepadBindings;

    fn deref(&self) -> &Self::Target {
        &self.bindings
    }
}

pub fn decode_stored_gamepad_bindings(raw: &str) -> Option<Vec<GamepadBindingEntry>> {
    let value: Value = serde_json::from_str(raw).ok()?;
    let entries = match value {
        Value::Object(entries) => ordered_object_entries(entries),
        Value::Array(entries) => entries
            .into_iter()
            .enumerate()
            .map(|(index, value)| (index.to_string(), value))
            .collect(),
        _ => return None,
    };

    Some(
        entries
            .into_iter()
            .filter_map(|(key, value)| {
                let button = javascript_number_button(&key)?;
                let Value::String(action) = value else {
                    return None;
                };
                Some(GamepadBindingEntry { button, action })
            })
            .collect(),
    )
}

pub fn encode_stored_gamepad_bindings(bindings: &GamepadBindings) -> String {
    let mut entries = Map::new();
    for (button, action) in bindings.stored_entries() {
        entries.insert(button.to_string(), Value::String(action.to_string()));
    }
    Value::Object(entries).to_string()
}

fn read_stored_entries<S>(storage: &S) -> Option<Vec<GamepadBindingEntry>>
where
    S: PreferenceStorage,
{
    let raw = storage.read(GAMEPAD_STORAGE_KEY).ok()??;
    decode_stored_gamepad_bindings(&raw)
}

fn ordered_object_entries(entries: Map<String, Value>) -> Vec<(String, Value)> {
    let mut entries = entries.into_iter().collect::<Vec<_>>();
    entries.sort_by(|(left, _), (right, _)| {
        match (javascript_array_index(left), javascript_array_index(right)) {
            (Some(left), Some(right)) => left.cmp(&right),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    });
    entries
}

fn javascript_array_index(key: &str) -> Option<u32> {
    let value = key.parse::<u32>().ok()?;
    (value != u32::MAX && value.to_string() == key).then_some(value)
}

fn javascript_number_button(key: &str) -> Option<usize> {
    let key = key.trim();
    let value = if key.is_empty() {
        0.0
    } else if let Some(hex) = key.strip_prefix("0x").or_else(|| key.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()? as f64
    } else if let Some(binary) = key.strip_prefix("0b").or_else(|| key.strip_prefix("0B")) {
        u64::from_str_radix(binary, 2).ok()? as f64
    } else if let Some(octal) = key.strip_prefix("0o").or_else(|| key.strip_prefix("0O")) {
        u64::from_str_radix(octal, 8).ok()? as f64
    } else {
        key.parse::<f64>().ok()?
    };
    value
        .is_finite()
        .then(|| {
            BINDABLE_GAMEPAD_BUTTONS
                .into_iter()
                .find(|button| value == *button as f64)
        })
        .flatten()
}
