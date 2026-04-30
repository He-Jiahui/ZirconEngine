use crate::core::framework::render::FrameHistoryHandle;

use crate::VisibilityHistorySnapshot;

use super::viewport_frame_history::ViewportFrameHistory;

impl ViewportFrameHistory {
    pub(crate) fn handle(&self) -> FrameHistoryHandle {
        self.handle
    }

    pub(crate) fn visibility(&self) -> &VisibilityHistorySnapshot {
        &self.visibility
    }
}
