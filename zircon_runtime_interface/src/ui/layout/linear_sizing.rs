use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiLinearSlotSizeRule {
    Auto,
    #[default]
    Stretch,
    StretchContent,
}

/// Main-axis sizing policy for a child owned by a Linear parent. This mirrors
/// Slate BoxPanel slot sizing without moving parent-specific policy onto the child node.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiLinearSlotSizing {
    #[serde(default)]
    pub rule: UiLinearSlotSizeRule,
    #[serde(default = "default_linear_slot_value")]
    pub value: f32,
    #[serde(default = "default_linear_slot_value")]
    pub shrink_value: f32,
    #[serde(default)]
    pub min: f32,
    #[serde(default = "default_linear_slot_max")]
    pub max: f32,
}

impl Default for UiLinearSlotSizing {
    fn default() -> Self {
        Self::new(UiLinearSlotSizeRule::Stretch)
    }
}

impl UiLinearSlotSizing {
    pub fn new(rule: UiLinearSlotSizeRule) -> Self {
        Self {
            rule,
            value: default_linear_slot_value(),
            shrink_value: default_linear_slot_value(),
            min: 0.0,
            max: default_linear_slot_max(),
        }
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn with_shrink_value(mut self, shrink_value: f32) -> Self {
        self.shrink_value = shrink_value;
        self
    }

    pub fn with_min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    pub fn with_max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }
}

const fn default_linear_slot_value() -> f32 {
    1.0
}

const fn default_linear_slot_max() -> f32 {
    -1.0
}
