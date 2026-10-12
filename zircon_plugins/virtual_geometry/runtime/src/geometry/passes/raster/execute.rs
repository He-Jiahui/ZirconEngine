use zircon_runtime::core::framework::render::RenderVirtualGeometryHardwareRasterizationSource;

use super::super::virtual_geometry_executed_cluster_selection_pass::VirtualGeometryExecutedClusterSelectionPassOutput;
use super::buffer::create_hardware_rasterization_buffer;
use super::output::VirtualGeometryHardwareRasterizationPassOutput;
use super::records::{
    collect_execution_hardware_rasterization_records_from_pass, pack_hardware_rasterization_records,
};

pub(in crate::virtual_geometry::renderer::root_render_passes) fn execute_virtual_geometry_hardware_rasterization_pass(
    device: &wgpu::Device,
    pass_enabled: bool,
    executed_cluster_selection_pass: &VirtualGeometryExecutedClusterSelectionPassOutput,
) -> VirtualGeometryHardwareRasterizationPassOutput {
    let records =
        collect_execution_hardware_rasterization_records_from_pass(executed_cluster_selection_pass);
    let packed_words = pack_hardware_rasterization_records(&records);
    VirtualGeometryHardwareRasterizationPassOutput::new(
        if !pass_enabled {
            RenderVirtualGeometryHardwareRasterizationSource::Unavailable
        } else if records.is_empty() {
            RenderVirtualGeometryHardwareRasterizationSource::RenderPathClearOnly
        } else {
            RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections
        },
        u32::try_from(records.len()).unwrap_or(u32::MAX),
        create_hardware_rasterization_buffer(device, &packed_words),
        records,
    )
}
