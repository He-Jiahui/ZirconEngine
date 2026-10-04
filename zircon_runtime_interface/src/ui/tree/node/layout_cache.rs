use serde::{Deserialize, Serialize};

use crate::ui::{
    layout::DesiredSize, layout::UiFrame, layout::UiSize, layout::UiVirtualListWindow,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiLayoutCache {
    pub desired_size: DesiredSize,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub content_size: UiSize,
    pub virtual_window: Option<UiVirtualListWindow>,
    /// True when desired/content size was measured for the node's current layout inputs.
    /// This is deliberately independent from `frame`: a valid layout may be zero-sized.
    #[serde(default)]
    pub measure_valid: bool,
    /// Advances whenever the retained node's text-layout inputs change.
    /// `u64::MAX` is a serialized exhaustion sentinel and is never a publishable cache revision.
    #[serde(default)]
    pub text_layout_revision: u64,
}

impl UiLayoutCache {
    pub fn invalidate_measure(&mut self) {
        self.measure_valid = false;
    }

    pub fn complete_measure(&mut self) {
        self.measure_valid = true;
    }

    pub fn advance_text_layout_revision(&mut self) {
        self.text_layout_revision = self.text_layout_revision.checked_add(1).unwrap_or(u64::MAX);
    }

    /// Returns a revision only while the retained identity cannot alias an earlier cache key.
    /// Layout remains available after exhaustion, but retained reuse must stay disabled.
    pub fn retained_text_layout_revision(&self) -> Option<u64> {
        (self.text_layout_revision != u64::MAX).then_some(self.text_layout_revision)
    }
}

#[cfg(test)]
#[path = "tests/layout_cache.rs"]
mod tests;
