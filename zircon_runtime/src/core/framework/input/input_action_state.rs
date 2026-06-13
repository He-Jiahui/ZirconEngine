use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputActionState {
    pressed: BTreeSet<String>,
    just_activated: BTreeSet<String>,
    just_deactivated: BTreeSet<String>,
}

impl InputActionState {
    pub fn from_sets(
        pressed: BTreeSet<String>,
        just_activated: BTreeSet<String>,
        just_deactivated: BTreeSet<String>,
    ) -> Self {
        Self {
            pressed,
            just_activated,
            just_deactivated,
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

    pub fn pressed_actions(&self) -> Vec<String> {
        self.pressed.iter().cloned().collect()
    }

    pub fn just_activated_actions(&self) -> Vec<String> {
        self.just_activated.iter().cloned().collect()
    }

    pub fn just_deactivated_actions(&self) -> Vec<String> {
        self.just_deactivated.iter().cloned().collect()
    }
}
