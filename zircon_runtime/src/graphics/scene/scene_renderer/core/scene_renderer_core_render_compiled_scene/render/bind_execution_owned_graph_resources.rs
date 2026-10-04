use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::scene::scene_renderer::core::scene_renderer_core::{
    HzbNeutralBuffers, LightGridNeutralBuffers, SceneRendererNeutralGraphBuffers,
    HZB_INDIRECT_ARGS_NEUTRAL_BACKING,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderPassMeshCommandLists,
};
use crate::graphics::scene::scene_renderer::hzb::{
    HzbOcclusionCuller, HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE,
    HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE, HZB_OCCLUSION_DRAW_COUNT_RESOURCE,
    HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE, HZB_OCCLUSION_STATS_RESOURCE,
    HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshIndirectDrawExecution;
use crate::render_graph::CompiledRenderGraph;

const HZB_INDIRECT_ARGS_EXECUTION_BACKING: &str = "hzb-occlusion-indirect-args:phase0";
const HZB_METADATA_EXECUTION_BACKING: &str = "hzb-occlusion-compaction-metadata:phase0";
const HZB_COMPACTED_ARGS_EXECUTION_BACKING: &str = "hzb-occlusion-compacted-indirect-args:phase0";
const HZB_VISIBLE_INDEX_EXECUTION_BACKING: &str = "hzb-occlusion-visible-instance-index:phase0";
const HZB_DRAW_COUNT_EXECUTION_BACKING: &str = "hzb-occlusion-draw-count:phase0";
const HZB_STATS_EXECUTION_BACKING: &str = "hzb-occlusion-stats:shared";

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn bind_execution_owned_graph_resources(
    device: &wgpu::Device,
    neutral_buffers: &mut SceneRendererNeutralGraphBuffers,
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    mesh_draw_lists: RenderPassMeshCommandLists<'_>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
) {
    let first_hzb_execution = mesh_draw_lists
        .hzb_occlusion_indirect_executions()
        .into_iter()
        .flatten()
        .next();
    if graph_declares_any_light_grid_external(graph) {
        bind_light_grid_external_buffers(graph, resources, neutral_buffers.light_grid(device));
    }
    if graph_declares_any_hzb_occlusion_external(graph) {
        bind_hzb_occlusion_external_buffers(
            graph,
            resources,
            first_hzb_execution,
            hzb_occlusion_culler,
            neutral_buffers.hzb(device),
        );
    }
}

fn bind_light_grid_external_buffers(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    neutral: &LightGridNeutralBuffers,
) {
    for logical_name in LIGHT_GRID_EXTERNAL_BUFFER_NAMES {
        bind_light_grid_execution_buffer(graph, resources, logical_name, neutral);
    }
}

fn graph_declares_any_light_grid_external(graph: &CompiledRenderGraph) -> bool {
    LIGHT_GRID_EXTERNAL_BUFFER_NAMES
        .iter()
        .any(|resource_name| graph.resource_lifetime_by_name(resource_name).is_some())
}

fn bind_light_grid_execution_buffer(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    neutral: &LightGridNeutralBuffers,
) {
    if graph.resource_lifetime_by_name(logical_name).is_none() || resources.has_buffer(logical_name)
    {
        return;
    }

    if let Some((buffer, backing_name)) = neutral.buffer(logical_name) {
        resources.bind_execution_owned_buffer(logical_name, backing_name, buffer);
    }
}

fn bind_hzb_occlusion_external_buffers(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    first_execution: Option<&MeshIndirectDrawExecution>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
    neutral: &HzbNeutralBuffers,
) {
    bind_hzb_execution_buffer(
        graph,
        resources,
        HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE,
        first_execution.map(MeshIndirectDrawExecution::args_buffer),
        HZB_INDIRECT_ARGS_EXECUTION_BACKING,
        neutral,
    );
    bind_hzb_execution_buffer(
        graph,
        resources,
        HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE,
        first_execution.map(|execution| execution.compaction_resources().metadata_buffer()),
        HZB_METADATA_EXECUTION_BACKING,
        neutral,
    );
    bind_hzb_execution_buffer(
        graph,
        resources,
        HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE,
        first_execution.map(|execution| {
            execution
                .compaction_resources()
                .compacted_indirect_args_buffer()
        }),
        HZB_COMPACTED_ARGS_EXECUTION_BACKING,
        neutral,
    );
    bind_hzb_execution_buffer(
        graph,
        resources,
        HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
        first_execution.map(|execution| {
            execution
                .compaction_resources()
                .visible_instance_index_buffer()
        }),
        HZB_VISIBLE_INDEX_EXECUTION_BACKING,
        neutral,
    );
    bind_hzb_execution_buffer(
        graph,
        resources,
        HZB_OCCLUSION_DRAW_COUNT_RESOURCE,
        first_execution.map(|execution| execution.compaction_resources().draw_count_buffer()),
        HZB_DRAW_COUNT_EXECUTION_BACKING,
        neutral,
    );
    bind_hzb_execution_buffer(
        graph,
        resources,
        HZB_OCCLUSION_STATS_RESOURCE,
        hzb_occlusion_culler.map(HzbOcclusionCuller::stats_buffer),
        HZB_STATS_EXECUTION_BACKING,
        neutral,
    );
}

fn graph_declares_any_hzb_occlusion_external(graph: &CompiledRenderGraph) -> bool {
    HZB_OCCLUSION_EXTERNAL_BUFFER_NAMES
        .iter()
        .any(|resource_name| graph.resource_lifetime_by_name(resource_name).is_some())
}

fn bind_hzb_execution_buffer(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    buffer: Option<&wgpu::Buffer>,
    execution_backing_name: &'static str,
    neutral: &HzbNeutralBuffers,
) {
    if graph.resource_lifetime_by_name(logical_name).is_none() {
        return;
    }
    if let Some(buffer) = buffer {
        resources.bind_execution_owned_buffer(logical_name, execution_backing_name, buffer);
        return;
    }

    if let Some((buffer, backing_name)) = neutral.buffer(logical_name) {
        resources.bind_execution_owned_buffer(logical_name, backing_name, buffer);
    }
}

const HZB_OCCLUSION_EXTERNAL_BUFFER_NAMES: &[&str] = &[
    HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE,
    HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE,
    HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE,
    HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
    HZB_OCCLUSION_DRAW_COUNT_RESOURCE,
    HZB_OCCLUSION_STATS_RESOURCE,
];
const LIGHT_GRID_EXTERNAL_BUFFER_NAMES: &[&str] = &[
    PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
    PostProcessGraphResourceNames::LIGHT_ZBINS,
    PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
];

#[cfg(test)]
#[path = "tests/bind_execution_owned_graph_resources.rs"]
mod tests;
