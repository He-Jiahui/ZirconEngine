use zircon_runtime::graphics::VisibilityVirtualGeometryFeedback;

use super::super::{VirtualGeometryRuntimeState, HOT_FRONTIER_COOLING_FRAME_COUNT};

impl VirtualGeometryRuntimeState {
    pub(crate) fn refresh_hot_resident_pages(
        &mut self,
        feedback: &VisibilityVirtualGeometryFeedback,
    ) {
        let resident_page_ids = self.resident_page_id_index();
        self.retain_recent_hot_resident_pages(|page_id, frames_remaining| {
            if !resident_page_ids.contains(page_id) {
                return false;
            }
            if *frames_remaining <= 1 {
                return false;
            }
            *frames_remaining -= 1;
            true
        });
        let cooling_entries = self
            .current_hot_resident_page_ids()
            .filter(|page_id| resident_page_ids.contains(page_id))
            .map(|page_id| (page_id, HOT_FRONTIER_COOLING_FRAME_COUNT))
            .collect::<Vec<_>>();
        self.extend_recent_hot_resident_pages(cooling_entries);
        self.retain_recent_hot_resident_pages(|page_id, _| resident_page_ids.contains(page_id));
        let current_hot_resident_pages = feedback
            .hot_resident_pages
            .iter()
            .copied()
            .filter(|page_id| resident_page_ids.contains(page_id))
            .collect();
        self.replace_current_hot_resident_pages(current_hot_resident_pages);
    }

    pub(crate) fn consume_feedback(&mut self, feedback: &VisibilityVirtualGeometryFeedback) {
        // BUG: [CR-VIRTUAL-GEOMETRY-0002] provider 在无 GPU 完成时已刷新热点，此处再次刷新使同一帧冷却计数递减两次，页提前失去热点保护；见 provider.rs:98-110。
        self.refresh_hot_resident_pages(feedback);
        self.complete_pending_pages(
            feedback.requested_pages.iter().copied(),
            &feedback.evictable_pages,
        );
        let resident_page_ids = self.resident_page_id_index();
        self.retain_current_hot_resident_pages(|page_id| resident_page_ids.contains(page_id));
        self.retain_recent_hot_resident_pages(|page_id, _| resident_page_ids.contains(page_id));
    }
}
