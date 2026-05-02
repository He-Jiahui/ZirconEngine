use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiHostCapability {
    Editor,
    Runtime,
    TextInput,
    PointerInput,
    KeyboardNavigation,
    GamepadNavigation,
    ImageRender,
    CanvasRender,
    VirtualizedLayout,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiHostCapabilitySet {
    #[serde(default)]
    pub capabilities: BTreeSet<UiHostCapability>,
}

impl UiHostCapabilitySet {
    pub fn new(capabilities: impl IntoIterator<Item = UiHostCapability>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }

    /// Authoring hosts must support the full native widget palette and previews.
    pub fn editor_authoring() -> Self {
        Self::new([
            UiHostCapability::Editor,
            UiHostCapability::Runtime,
            UiHostCapability::TextInput,
            UiHostCapability::PointerInput,
            UiHostCapability::KeyboardNavigation,
            UiHostCapability::ImageRender,
            UiHostCapability::CanvasRender,
            UiHostCapability::VirtualizedLayout,
        ])
    }

    pub fn runtime_basic() -> Self {
        Self::new([
            UiHostCapability::Runtime,
            UiHostCapability::PointerInput,
            UiHostCapability::KeyboardNavigation,
            UiHostCapability::ImageRender,
            UiHostCapability::CanvasRender,
        ])
    }

    pub fn contains_all(&self, required: &BTreeSet<UiHostCapability>) -> bool {
        required
            .iter()
            .all(|capability| self.capabilities.contains(capability))
    }

    pub fn missing(&self, required: &BTreeSet<UiHostCapability>) -> BTreeSet<UiHostCapability> {
        required
            .iter()
            .copied()
            .filter(|capability| !self.capabilities.contains(capability))
            .collect()
    }
}
