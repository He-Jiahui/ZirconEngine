use crate::core::framework::render::{
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
};

use super::super::virtual_geometry_executed_cluster_selection_pass::VirtualGeometryExecutedClusterSelectionPassOutput;
use super::buffer::create_visbuffer64_buffer;
use super::entries::{
    collect_execution_visbuffer64_entries_from_pass, pack_execution_visbuffer64_entries,
};
use super::output::VirtualGeometryVisBuffer64PassOutput;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render) fn execute_virtual_geometry_visbuffer64_pass(
    device: &wgpu::Device,
    pass_enabled: bool,
    executed_cluster_selection_pass: &VirtualGeometryExecutedClusterSelectionPassOutput,
) -> VirtualGeometryVisBuffer64PassOutput {
    let entries = collect_execution_visbuffer64_entries_from_pass(executed_cluster_selection_pass);
    let packed_words = pack_execution_visbuffer64_entries(&entries);
    VirtualGeometryVisBuffer64PassOutput::new(
        RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        entries,
        if !pass_enabled {
            RenderVirtualGeometryVisBuffer64Source::Unavailable
        } else if packed_words.is_empty() {
            RenderVirtualGeometryVisBuffer64Source::RenderPathClearOnly
        } else {
            RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections
        },
        u32::try_from(packed_words.len()).unwrap_or(u32::MAX),
        create_visbuffer64_buffer(device, &packed_words),
    )
}
