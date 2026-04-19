use crate::VisibilityVirtualGeometryFeedback;

use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn refresh_hot_resident_pages(
        &mut self,
        feedback: &VisibilityVirtualGeometryFeedback,
    ) {
        self.recent_hot_resident_pages = self.current_hot_resident_pages.clone();
        self.recent_hot_resident_pages
            .retain(|page_id| self.resident_slots.contains_key(page_id));
        self.current_hot_resident_pages = feedback
            .hot_resident_pages
            .iter()
            .copied()
            .filter(|page_id| self.resident_slots.contains_key(page_id))
            .collect();
    }

    pub(crate) fn consume_feedback(&mut self, feedback: &VisibilityVirtualGeometryFeedback) {
        self.refresh_hot_resident_pages(feedback);
        self.complete_pending_pages(
            feedback.requested_pages.iter().copied(),
            &feedback.evictable_pages,
        );
        self.current_hot_resident_pages
            .retain(|page_id| self.resident_slots.contains_key(page_id));
        self.recent_hot_resident_pages
            .retain(|page_id| self.resident_slots.contains_key(page_id));
    }
}
