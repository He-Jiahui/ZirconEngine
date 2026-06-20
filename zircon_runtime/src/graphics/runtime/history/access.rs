use crate::core::framework::render::FrameHistoryHandle;

use crate::graphics::visibility::VisibilityStaticIndex;
use crate::graphics::VisibilityHistorySnapshot;

use super::viewport_frame_history::ViewportFrameHistory;

impl ViewportFrameHistory {
    pub(crate) fn handle(&self) -> FrameHistoryHandle {
        self.handle
    }

    pub(crate) fn visibility(&self) -> &VisibilityHistorySnapshot {
        &self.visibility
    }

    pub(crate) fn static_index(&self) -> &VisibilityStaticIndex {
        &self.static_index
    }
}
