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
    /// Advances whenever the retained node's text-layout inputs change.
    #[serde(default)]
    pub text_layout_revision: u64,
}

impl UiLayoutCache {
    pub fn advance_text_layout_revision(&mut self) {
        self.text_layout_revision = self.text_layout_revision.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::UiLayoutCache;

    #[test]
    fn text_layout_revision_advances_from_the_default_cache() {
        let mut cache = UiLayoutCache::default();

        cache.advance_text_layout_revision();

        assert_eq!(cache.text_layout_revision, 1);
    }
}
