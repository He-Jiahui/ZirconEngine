use crate::VisibilityVirtualGeometryFeedback;

use super::super::{VirtualGeometryRuntimeState, HOT_FRONTIER_COOLING_FRAME_COUNT};

impl VirtualGeometryRuntimeState {
    pub(crate) fn refresh_hot_resident_pages(
        &mut self,
        feedback: &VisibilityVirtualGeometryFeedback,
    ) {
        self.recent_hot_resident_pages
            .retain(|page_id, frames_remaining| {
                if !self.resident_slots.contains_key(page_id) {
                    return false;
                }
                if *frames_remaining <= 1 {
                    return false;
                }
                *frames_remaining -= 1;
                true
            });
        self.recent_hot_resident_pages.extend(
            self.current_hot_resident_pages
                .iter()
                .copied()
                .filter(|page_id| self.resident_slots.contains_key(page_id))
                .map(|page_id| (page_id, HOT_FRONTIER_COOLING_FRAME_COUNT)),
        );
        self.recent_hot_resident_pages
            .retain(|page_id, _| self.resident_slots.contains_key(page_id));
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
            .retain(|page_id, _| self.resident_slots.contains_key(page_id));
    }
}
