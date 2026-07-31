use std::collections::BTreeMap;

use super::registry::{bool_setting, numeric_setting, BOOL_SETTINGS, NUMERIC_SETTINGS};

#[derive(Clone, Debug, PartialEq)]
pub struct ClientSettings {
    numeric: BTreeMap<&'static str, f64>,
    boolean: BTreeMap<&'static str, bool>,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            numeric: NUMERIC_SETTINGS
                .into_iter()
                .map(|setting| (setting.id, setting.default))
                .collect(),
            boolean: BOOL_SETTINGS
                .into_iter()
                .map(|setting| (setting.id, setting.default))
                .collect(),
        }
    }
}

impl ClientSettings {
    pub fn numeric(&self, id: &str) -> Option<f64> {
        self.numeric.get(id).copied()
    }

    pub fn boolean(&self, id: &str) -> Option<bool> {
        self.boolean.get(id).copied()
    }

    pub fn set_numeric(&mut self, id: &str, value: f64) -> Option<f64> {
        let setting = numeric_setting(id)?;
        let value = if value.is_finite() {
            value.clamp(setting.min, setting.max)
        } else {
            setting.default
        };
        self.numeric.insert(setting.id, value);
        Some(value)
    }

    pub fn set_boolean(&mut self, id: &str, value: bool) -> Option<bool> {
        let setting = bool_setting(id)?;
        self.boolean.insert(setting.id, value);
        Some(value)
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn all(&self) -> Self {
        self.clone()
    }

    pub fn numeric_len(&self) -> usize {
        self.numeric.len()
    }

    pub fn boolean_len(&self) -> usize {
        self.boolean.len()
    }
}

pub fn normalize_click_move_button(value: f64) -> u8 {
    if value >= 1.0 {
        2
    } else {
        0
    }
}

pub fn click_move_button_label(value: f64) -> &'static str {
    if normalize_click_move_button(value) == 2 {
        "Right Click"
    } else {
        "Left Click"
    }
}
