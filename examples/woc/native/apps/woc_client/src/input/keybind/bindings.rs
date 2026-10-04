use std::collections::HashMap;

use super::{
    action_kind, combo_code, is_reserved_combo, key_label, normalize_key_combo, KeyBindingKind,
    KEYBIND_ACTIONS,
};

pub(super) const BINDING_SLOTS: usize = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ReverseBinding {
    any: Option<usize>,
    edge: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keybinds {
    slots: Vec<[Option<String>; BINDING_SLOTS]>,
    reverse: HashMap<String, ReverseBinding>,
    held_reverse: HashMap<String, usize>,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self::from_slots(default_slots())
    }
}

impl Keybinds {
    pub(super) fn from_slots(slots: Vec<[Option<String>; BINDING_SLOTS]>) -> Self {
        let capacity = slots.len().saturating_mul(BINDING_SLOTS);
        let mut keybinds = Self {
            slots,
            reverse: HashMap::with_capacity(capacity),
            held_reverse: HashMap::with_capacity(capacity),
        };
        keybinds.rebuild_reverse_indexes();
        keybinds
    }

    pub fn kind(&self, id: &str) -> Option<KeyBindingKind> {
        action_kind(id)
    }

    pub fn action_for_combo(&self, combo: &str) -> Option<&'static str> {
        self.reverse
            .get(combo)
            .and_then(|binding| binding.any)
            .map(|index| KEYBIND_ACTIONS[index].id)
    }

    pub fn edge_action_for_combo(&self, combo: &str) -> Option<&'static str> {
        self.reverse
            .get(combo)
            .and_then(|binding| binding.edge)
            .map(|index| KEYBIND_ACTIONS[index].id)
    }

    pub fn held_action_for_code(&self, code: &str) -> Option<&'static str> {
        self.held_reverse
            .get(code)
            .map(|index| KEYBIND_ACTIONS[*index].id)
    }

    pub fn codes_for_action(&self, id: &str) -> Vec<&str> {
        action_index(id)
            .map(|index| {
                self.slots[index]
                    .iter()
                    .filter_map(Option::as_deref)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn code_at(&self, id: &str, slot: usize) -> Option<&str> {
        action_index(id)
            .and_then(|index| self.slots.get(index))
            .and_then(|slots| slots.get(slot))
            .and_then(Option::as_deref)
    }

    pub fn label_at(&self, id: &str, slot: usize) -> String {
        key_label(self.code_at(id, slot))
    }

    pub fn primary_label(&self, id: &str) -> String {
        key_label(self.code_at(id, 0).or_else(|| self.code_at(id, 1)))
    }

    pub fn bind(&mut self, id: &str, slot: usize, combo: &str) -> bool {
        let Some(index) = action_index(id) else {
            return false;
        };
        if slot >= BINDING_SLOTS {
            return false;
        }

        let action = KEYBIND_ACTIONS[index];
        let value = normalize_key_combo(action.kind, combo);
        if is_reserved_combo(&value) {
            return false;
        }

        if !action.allow_shared {
            for (other_index, other_action) in KEYBIND_ACTIONS.iter().enumerate() {
                if other_index == index || other_action.allow_shared {
                    continue;
                }
                for other_slot in 0..BINDING_SLOTS {
                    if self.slots[other_index][other_slot].as_deref() == Some(value.as_str()) {
                        self.slots[other_index][other_slot] = None;
                    }
                }
            }
        }

        self.slots[index][slot] = Some(value);
        self.rebuild_reverse_indexes();
        true
    }

    pub fn clear(&mut self, id: &str, slot: usize) {
        let Some(index) = action_index(id) else {
            return;
        };
        let Some(binding) = self.slots[index].get_mut(slot) else {
            return;
        };
        *binding = None;
        self.rebuild_reverse_indexes();
    }

    pub fn reset(&mut self) {
        self.slots = default_slots();
        self.rebuild_reverse_indexes();
    }

    fn rebuild_reverse_indexes(&mut self) {
        self.reverse.clear();
        self.held_reverse.clear();
        for (index, (action, slots)) in KEYBIND_ACTIONS.iter().zip(&self.slots).enumerate() {
            for combo in slots.iter().flatten() {
                let binding = self.reverse.entry(combo.clone()).or_default();
                binding.any.get_or_insert(index);
                if action.kind == KeyBindingKind::Edge {
                    binding.edge.get_or_insert(index);
                } else {
                    self.held_reverse
                        .entry(combo_code(combo).to_owned())
                        .or_insert(index);
                }
            }
        }
    }
}

pub(super) fn default_slots() -> Vec<[Option<String>; BINDING_SLOTS]> {
    KEYBIND_ACTIONS
        .iter()
        .map(|action| action.defaults.map(|value| value.map(str::to_string)))
        .collect()
}

fn action_index(id: &str) -> Option<usize> {
    KEYBIND_ACTIONS.iter().position(|action| action.id == id)
}

#[cfg(test)]
#[path = "tests/bindings_performance_tests.rs"]
mod performance_tests;
