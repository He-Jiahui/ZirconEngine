use crate::types::GraphicsError;

use super::completed_page_assignments::completed_page_assignments;
use super::page_table_entries::page_table_entries;
use super::virtual_geometry_gpu_pending_readback::VirtualGeometryGpuPendingReadback;
use super::virtual_geometry_gpu_readback::VirtualGeometryGpuReadback;

impl VirtualGeometryGpuPendingReadback {
    pub(crate) fn collect(
        self,
        device: &wgpu::Device,
    ) -> Result<VirtualGeometryGpuReadback, GraphicsError> {
        let (completed_page_assignments, completed_page_ids) =
            completed_page_assignments(device, &self.completed_buffer, self.completed_word_count)?;
        let page_table_entries = page_table_entries(
            device,
            &self.page_table_buffer,
            self.page_table_word_count,
            self.resident_entry_count,
            self.resident_slots,
            &completed_page_assignments,
        )?;

        Ok(VirtualGeometryGpuReadback {
            page_table_entries,
            completed_page_ids,
            completed_page_assignments,
        })
    }
}
