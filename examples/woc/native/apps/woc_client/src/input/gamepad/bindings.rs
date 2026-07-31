use std::collections::BTreeMap;

use super::layout::{BINDABLE_GAMEPAD_BUTTONS, DEFAULT_GAMEPAD_BINDINGS, GAMEPAD_NONE_ACTION};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GamepadBindingEntry {
    pub button: usize,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GamepadBindings {
    bindings: BTreeMap<usize, String>,
}

impl Default for GamepadBindings {
    fn default() -> Self {
        Self {
            bindings: default_bindings(),
        }
    }
}

impl GamepadBindings {
    pub fn from_stored(entries: impl IntoIterator<Item = GamepadBindingEntry>) -> Self {
        let mut bindings = Self::default();
        for entry in entries {
            if is_bindable_gamepad_button(entry.button) {
                bindings.bindings.insert(entry.button, entry.action);
            }
        }
        bindings
    }

    pub fn action_for(&self, button: usize) -> &str {
        self.bindings
            .get(&button)
            .map(String::as_str)
            .unwrap_or(GAMEPAD_NONE_ACTION)
    }

    pub fn bind(&mut self, button: usize, action: impl Into<String>) {
        if !is_bindable_gamepad_button(button) {
            return;
        }
        let action = action.into();
        if action == GAMEPAD_NONE_ACTION {
            self.bindings.remove(&button);
        } else {
            self.bindings.insert(button, action);
        }
    }

    pub fn reset(&mut self) {
        self.bindings = default_bindings();
    }

    pub fn entries(&self) -> Vec<GamepadBindingEntry> {
        BINDABLE_GAMEPAD_BUTTONS
            .into_iter()
            .map(|button| GamepadBindingEntry {
                button,
                action: self.action_for(button).to_string(),
            })
            .collect()
    }

    pub(super) fn stored_entries(&self) -> impl Iterator<Item = (usize, &str)> {
        self.bindings
            .iter()
            .map(|(button, action)| (*button, action.as_str()))
    }
}

fn default_bindings() -> BTreeMap<usize, String> {
    DEFAULT_GAMEPAD_BINDINGS
        .into_iter()
        .map(|(button, action)| (button, action.to_string()))
        .collect()
}

pub fn is_bindable_gamepad_button(button: usize) -> bool {
    BINDABLE_GAMEPAD_BUTTONS.binary_search(&button).is_ok()
}
