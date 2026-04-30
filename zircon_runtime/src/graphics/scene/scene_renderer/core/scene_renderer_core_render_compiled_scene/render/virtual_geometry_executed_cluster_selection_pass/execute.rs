use std::collections::HashSet;

use crate::core::framework::render::{
    RenderVirtualGeometryExtract, RenderVirtualGeometrySelectedClusterSource,
};
use crate::graphics::scene::scene_renderer::mesh::MeshDraw;
use crate::graphics::types::VirtualGeometryClusterSelection;

use super::super::virtual_geometry_node_and_cluster_cull_pass::VirtualGeometryNodeAndClusterCullPassOutput;
use super::buffer::create_selected_cluster_buffer;
use super::output::VirtualGeometryExecutedClusterSelectionPassOutput;
use super::seed_backed_execution_selection::collect_execution_cluster_selection_collection_from_root_seeds;
use super::selection_collection::ExecutedClusterSelectionCollection;
use super::selection_filter::collect_execution_cluster_selections_from_submission_keys;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render) fn execute_virtual_geometry_executed_cluster_selection_pass(
    device: &wgpu::Device,
    selected_cluster_pass_enabled: bool,
    cluster_selections: Option<&[VirtualGeometryClusterSelection]>,
    indirect_execution_draws: &[&MeshDraw],
    extract: Option<&RenderVirtualGeometryExtract>,
    node_and_cluster_cull_pass: &VirtualGeometryNodeAndClusterCullPassOutput,
) -> VirtualGeometryExecutedClusterSelectionPassOutput {
    if !selected_cluster_pass_enabled {
        return VirtualGeometryExecutedClusterSelectionPassOutput::new(
            Vec::new(),
            Vec::new(),
            RenderVirtualGeometrySelectedClusterSource::Unavailable,
            0,
            None,
        );
    }

    let executed_submission_keys = indirect_execution_draws
        .iter()
        .filter_map(|draw| draw.virtual_geometry_execution_selection_key())
        .collect::<HashSet<_>>();
    let selections = collect_execution_cluster_selections_from_submission_keys(
        cluster_selections,
        &executed_submission_keys,
    );
    let selection_collection = if selections.is_empty() && cluster_selections.is_none() {
        collect_execution_cluster_selection_collection_from_root_seeds(
            extract,
            node_and_cluster_cull_pass,
        )
    } else {
        ExecutedClusterSelectionCollection::from_selections(selections)
    };
    let selected_cluster_count =
        u32::try_from(selection_collection.selected_clusters().len()).unwrap_or(u32::MAX);
    let selected_cluster_buffer =
        create_selected_cluster_buffer(device, selection_collection.selected_clusters());
    let (selections, selected_clusters) = selection_collection.into_parts();
    VirtualGeometryExecutedClusterSelectionPassOutput::new(
        selections,
        selected_clusters,
        if selected_cluster_count == 0 {
            RenderVirtualGeometrySelectedClusterSource::RenderPathClearOnly
        } else {
            RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections
        },
        selected_cluster_count,
        selected_cluster_buffer,
    )
}
