use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::component::UiValue;

/// Carries host projection updates while keeping attributes separate from retained state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiComponentProjectionPatch {
    pub control_id: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, UiValue>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub state_values: BTreeMap<String, UiValue>,
}

impl UiComponentProjectionPatch {
    pub fn new(control_id: impl Into<String>) -> Self {
        Self {
            control_id: control_id.into(),
            attributes: BTreeMap::new(),
            state_values: BTreeMap::new(),
        }
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: UiValue) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    pub fn with_state_value(mut self, key: impl Into<String>, value: UiValue) -> Self {
        self.state_values.insert(key.into(), value);
        self
    }
}
