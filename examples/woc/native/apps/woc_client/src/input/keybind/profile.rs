use std::collections::{BTreeMap, HashSet};

use super::{
    bindings::{default_slots, Keybinds, BINDING_SLOTS},
    combo::is_reserved_combo,
    registry::KEYBIND_ACTIONS,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoredBindingSlot {
    Combo(String),
    Empty,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoredKeybindValue {
    Slots(Vec<StoredBindingSlot>),
    Malformed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StoredKeybindProfile {
    entries: BTreeMap<String, StoredKeybindValue>,
}

impl StoredKeybindProfile {
    pub fn from_entries<I, K>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, StoredKeybindValue)>,
        K: Into<String>,
    {
        Self {
            entries: entries
                .into_iter()
                .map(|(id, value)| (id.into(), value))
                .collect(),
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    pub fn get(&self, id: &str) -> Option<&StoredKeybindValue> {
        self.entries.get(id)
    }

    fn slots(&self, id: &str) -> Option<&[StoredBindingSlot]> {
        match self.entries.get(id) {
            Some(StoredKeybindValue::Slots(slots)) => Some(slots),
            Some(StoredKeybindValue::Malformed) | None => None,
        }
    }

    fn remove(&mut self, id: &str) {
        self.entries.remove(id);
    }
}

pub fn repair_stored_bindings(profile: &mut StoredKeybindProfile) -> &mut StoredKeybindProfile {
    let strafe_signature = entry_primary(profile, "slot10") == Some("KeyQ")
        && entry_primary(profile, "slot11") == Some("KeyE")
        && is_empty_entry(profile, "strafeLeft")
        && is_empty_entry(profile, "strafeRight");
    if strafe_signature {
        for id in ["slot10", "slot11", "strafeLeft", "strafeRight"] {
            profile.remove(id);
        }
    }

    let friendly_target_is_key_h = !profile.contains("targetFriendly")
        || entry_primary(profile, "targetFriendly") == Some("KeyH");
    if is_empty_entry(profile, "meters") && friendly_target_is_key_h {
        profile.remove("meters");
    }
    profile
}

impl Keybinds {
    pub fn from_stored_profile(mut profile: StoredKeybindProfile) -> Self {
        repair_stored_bindings(&mut profile);

        let mut slots = default_slots();
        let mut claimed = HashSet::new();

        for (index, action) in KEYBIND_ACTIONS.iter().enumerate() {
            let Some(stored_slots) = profile.slots(action.id) else {
                continue;
            };
            let mut loaded = std::array::from_fn(|_| None);
            for (slot_index, stored) in stored_slots.iter().take(BINDING_SLOTS).enumerate() {
                let StoredBindingSlot::Combo(combo) = stored else {
                    continue;
                };
                if is_reserved_combo(combo) {
                    continue;
                }
                if !action.allow_shared && !claimed.insert(combo.clone()) {
                    continue;
                }
                loaded[slot_index] = Some(combo.clone());
            }
            slots[index] = loaded;
        }

        for (index, action) in KEYBIND_ACTIONS.iter().enumerate() {
            if profile.slots(action.id).is_some() || action.allow_shared {
                continue;
            }
            for stored in &mut slots[index] {
                let Some(combo) = stored else {
                    continue;
                };
                if !claimed.insert(combo.clone()) {
                    *stored = None;
                }
            }
        }

        Self::from_slots(slots)
    }
}

fn entry_primary<'a>(profile: &'a StoredKeybindProfile, id: &str) -> Option<&'a str> {
    match profile.slots(id)?.first() {
        Some(StoredBindingSlot::Combo(combo)) => Some(combo),
        Some(StoredBindingSlot::Empty | StoredBindingSlot::Invalid) | None => None,
    }
}

fn is_empty_entry(profile: &StoredKeybindProfile, id: &str) -> bool {
    profile.slots(id).is_some_and(|slots| {
        slots
            .iter()
            .all(|slot| matches!(slot, StoredBindingSlot::Empty))
    })
}
