use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{UiDragSourceMetadata, UiValidationState, UiValue};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiComponentFlags {
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub dragging: bool,
    pub drop_hovered: bool,
    pub active_drag_target: bool,
    pub popup_open: bool,
    pub expanded: bool,
    pub selected: bool,
    pub checked: bool,
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentState {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub values: BTreeMap<String, UiValue>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reference_sources: BTreeMap<String, UiDragSourceMetadata>,
    #[serde(default)]
    pub validation: UiValidationState,
    #[serde(default)]
    pub flags: UiComponentFlags,
}

impl Default for UiComponentState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiComponentState {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
            reference_sources: BTreeMap::new(),
            validation: UiValidationState::normal(),
            flags: UiComponentFlags::default(),
        }
    }

    pub fn with_value(mut self, property: impl Into<String>, value: UiValue) -> Self {
        self.values.insert(property.into(), value);
        self
    }

    pub fn value(&self, property: &str) -> Option<&UiValue> {
        self.values.get(property)
    }

    pub fn reference_source(&self, property: &str) -> Option<&UiDragSourceMetadata> {
        self.reference_sources.get(property)
    }
}
