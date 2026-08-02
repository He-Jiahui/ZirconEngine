use std::sync::Arc;

use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::surface::UiTextRange;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextRouteIdentity {
    tree_id: Arc<str>,
    node_id: UiNodeId,
    source_range: Option<(usize, usize)>,
}

impl ScreenSpaceUiTextRouteIdentity {
    pub(in crate::graphics::scene::scene_renderer::ui) fn new(
        tree_id: impl Into<Arc<str>>,
        node_id: UiNodeId,
        source_range: Option<UiTextRange>,
    ) -> Self {
        Self {
            tree_id: tree_id.into(),
            node_id,
            source_range: source_range.map(|range| (range.start, range.end)),
        }
    }
}
