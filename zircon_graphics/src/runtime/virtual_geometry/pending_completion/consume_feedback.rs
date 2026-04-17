use crate::VisibilityVirtualGeometryFeedback;

use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn consume_feedback(&mut self, feedback: &VisibilityVirtualGeometryFeedback) {
        self.complete_pending_pages(
            feedback.requested_pages.iter().copied(),
            &feedback.evictable_pages,
        );
    }
}
