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
mod tests {
    use crate::ui::layout::UiFrame;

    use super::UiLayoutCache;

    #[test]
    fn text_layout_revision_advances_from_the_default_cache() {
        let mut cache = UiLayoutCache::default();

        cache.advance_text_layout_revision();

        assert_eq!(cache.text_layout_revision, 1);
        assert_eq!(cache.retained_text_layout_revision(), Some(1));
    }

    #[test]
    fn exhausted_text_layout_revision_disables_retained_identity_without_wrapping() {
        let mut cache = UiLayoutCache {
            text_layout_revision: u64::MAX - 1,
            ..UiLayoutCache::default()
        };

        cache.advance_text_layout_revision();
        assert_eq!(cache.text_layout_revision, u64::MAX);
        assert_eq!(cache.retained_text_layout_revision(), None);

        cache.advance_text_layout_revision();
        assert_eq!(cache.text_layout_revision, u64::MAX);
        assert_eq!(cache.retained_text_layout_revision(), None);
    }

    #[test]
    fn measurement_validity_is_independent_from_zero_geometry() {
        let mut cache = UiLayoutCache::default();

        cache.complete_measure();
        assert!(cache.measure_valid);

        cache.frame = UiFrame::default();
        assert!(cache.measure_valid);

        cache.invalidate_measure();
        assert!(!cache.measure_valid);
    }
}
