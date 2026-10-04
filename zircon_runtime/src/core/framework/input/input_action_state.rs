use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// 一次动作求值的按住、边沿与模拟值结果；构造器把值约束到负一到一，并从值表去除零与非有限值。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputActionState {
    pressed: BTreeSet<String>,
    just_activated: BTreeSet<String>,
    just_deactivated: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    values: BTreeMap<String, f32>,
}

impl InputActionState {
    /// 兼容仅有布尔动作状态的调用方：按住动作按满幅值解释。
    pub fn from_sets(
        pressed: BTreeSet<String>,
        just_activated: BTreeSet<String>,
        just_deactivated: BTreeSet<String>,
    ) -> Self {
        let values = pressed
            .iter()
            .map(|action| (action.clone(), 1.0))
            .collect::<BTreeMap<_, _>>();
        Self {
            pressed,
            just_activated,
            just_deactivated,
            values,
        }
    }

    pub fn from_sets_and_values(
        pressed: BTreeSet<String>,
        just_activated: BTreeSet<String>,
        just_deactivated: BTreeSet<String>,
        values: BTreeMap<String, f32>,
    ) -> Self {
        Self {
            pressed,
            just_activated,
            just_deactivated,
            values: normalized_action_values(values),
        }
    }

    pub fn pressed(&self, action: impl AsRef<str>) -> bool {
        self.pressed.contains(action.as_ref())
    }

    pub fn just_activated(&self, action: impl AsRef<str>) -> bool {
        self.just_activated.contains(action.as_ref())
    }

    pub fn just_deactivated(&self, action: impl AsRef<str>) -> bool {
        self.just_deactivated.contains(action.as_ref())
    }

    pub fn value(&self, action: impl AsRef<str>) -> f32 {
        self.values.get(action.as_ref()).copied().unwrap_or(0.0)
    }

    pub fn pressed_actions(&self) -> Vec<String> {
        self.pressed.iter().cloned().collect()
    }

    pub fn just_activated_actions(&self) -> Vec<String> {
        self.just_activated.iter().cloned().collect()
    }

    pub fn just_deactivated_actions(&self) -> Vec<String> {
        self.just_deactivated.iter().cloned().collect()
    }

    pub fn valued_actions(&self) -> Vec<(String, f32)> {
        self.values
            .iter()
            .map(|(action, value)| (action.clone(), *value))
            .collect()
    }
}

fn normalized_action_value(value: f32) -> Option<f32> {
    if !value.is_finite() || value == 0.0 {
        None
    } else {
        Some(value.clamp(-1.0, 1.0))
    }
}

fn normalized_action_values(mut values: BTreeMap<String, f32>) -> BTreeMap<String, f32> {
    values.retain(|_, value| {
        let Some(normalized) = normalized_action_value(*value) else {
            return false;
        };
        *value = normalized;
        true
    });
    values
}

#[cfg(test)]
#[path = "tests/input_action_state.rs"]
mod tests;
