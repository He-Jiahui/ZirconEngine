use crate::graphics::types::GraphicsError;

use super::super::decode::{completed_page_assignments, page_table_entries};
use super::super::readback::VirtualGeometryGpuReadback;
use super::VirtualGeometryGpuPendingReadback;

impl VirtualGeometryGpuPendingReadback {
    pub(in crate::graphics::scene::scene_renderer) fn collect(
        self,
        device: &wgpu::Device,
    ) -> Result<VirtualGeometryGpuReadback, GraphicsError> {
        let (completed_page_assignments, completed_page_ids, completed_page_replacements) =
            completed_page_assignments(device, &self.completed_buffer, self.completed_word_count)?;
        let page_table_entries = page_table_entries(
            device,
            &self.page_table_buffer,
            self.page_table_word_count,
            self.resident_entry_count,
            self.resident_slots,
            &completed_page_assignments,
        )?;

        Ok(VirtualGeometryGpuReadback::new(
            page_table_entries,
            completed_page_ids,
            completed_page_assignments,
            completed_page_replacements,
        ))
    }
}
