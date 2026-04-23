use crate::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};
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

        Ok(VirtualGeometryGpuReadback {
            page_table_entries,
            completed_page_ids,
            completed_page_assignments,
            completed_page_replacements,
            hardware_rasterization_record_count: 0,
            hardware_rasterization_source:
                RenderVirtualGeometryHardwareRasterizationSource::Unavailable,
            selected_cluster_count: 0,
            selected_cluster_source: RenderVirtualGeometrySelectedClusterSource::Unavailable,
            selected_clusters: Vec::new(),
            visbuffer64_entry_count: 0,
            visbuffer64_source: RenderVirtualGeometryVisBuffer64Source::Unavailable,
            visbuffer64_clear_value: 0,
            visbuffer64_entries: Vec::new(),
        })
    }
}
