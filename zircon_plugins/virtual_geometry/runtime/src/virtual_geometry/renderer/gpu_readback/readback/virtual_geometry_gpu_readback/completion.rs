use super::super::VirtualGeometryGpuReadbackCompletionParts;
use super::VirtualGeometryGpuReadback;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};

impl VirtualGeometryGpuReadback {
    pub(in crate::virtual_geometry::renderer) fn new(
        page_table_entries: Vec<(u32, u32)>,
        completed_page_ids: Vec<u32>,
        completed_page_assignments: Vec<(u32, u32)>,
        completed_page_replacements: Vec<(u32, u32)>,
    ) -> Self {
        Self {
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
        }
    }

    pub(in crate::virtual_geometry::renderer) fn into_completion_parts(
        self,
    ) -> VirtualGeometryGpuReadbackCompletionParts {
        VirtualGeometryGpuReadbackCompletionParts::new(
            self.page_table_entries,
            self.completed_page_assignments,
            self.completed_page_replacements,
        )
    }
}
