use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn temporal_frame_index(&self) -> u64 {
        self.temporal_frame_index
    }

    pub(in crate::graphics::runtime::render_framework) fn advance_temporal_frame_index(&mut self) {
        self.temporal_frame_index = self.temporal_frame_index.wrapping_add(1);
    }
}
