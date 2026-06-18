use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderPassMeshCommandLists,
};
use crate::graphics::scene::scene_renderer::hzb::{
    HzbOcclusionCuller, HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE,
    HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE, HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE,
    HZB_OCCLUSION_DRAW_COUNT_RESOURCE, HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE,
    HZB_OCCLUSION_STATS_RESOURCE, HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshIndirectDrawExecution, INDEXED_INDIRECT_ARGS_STRIDE_BYTES,
    INDIRECT_COMPACTION_METADATA_STRIDE_BYTES, INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES,
    INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES,
};
use crate::render_graph::CompiledRenderGraph;

const HZB_FALLBACK_STORAGE_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE
    .union(wgpu::BufferUsages::COPY_DST)
    .union(wgpu::BufferUsages::COPY_SRC);
const HZB_FALLBACK_INDIRECT_STORAGE_USAGE: wgpu::BufferUsages =
    HZB_FALLBACK_STORAGE_USAGE.union(wgpu::BufferUsages::INDIRECT);

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn bind_execution_owned_graph_resources(
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    mesh_draw_lists: RenderPassMeshCommandLists<'_>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
) {
    let hzb_executions = mesh_draw_lists
        .hzb_occlusion_indirect_executions()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    bind_hzb_occlusion_external_buffers(
        device,
        graph,
        resources,
        &hzb_executions,
        hzb_occlusion_culler,
    );
}

fn bind_hzb_occlusion_external_buffers(
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    executions: &[&MeshIndirectDrawExecution],
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
) {
    if !graph_declares_any_hzb_occlusion_external(graph) {
        return;
    }

    let first_execution = executions.first().copied();
    bind_hzb_execution_buffer(
        device,
        graph,
        resources,
        HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE,
        "hzb-occlusion-source-indirect-args",
        first_execution.map(MeshIndirectDrawExecution::args_buffer),
        INDEXED_INDIRECT_ARGS_STRIDE_BYTES,
        HZB_FALLBACK_INDIRECT_STORAGE_USAGE,
    );
    bind_hzb_execution_buffer(
        device,
        graph,
        resources,
        HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE,
        "hzb-occlusion-compaction-metadata",
        first_execution.map(|execution| execution.compaction_resources().metadata_buffer()),
        INDIRECT_COMPACTION_METADATA_STRIDE_BYTES,
        HZB_FALLBACK_STORAGE_USAGE,
    );
    bind_hzb_execution_buffer(
        device,
        graph,
        resources,
        HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE,
        "hzb-occlusion-compacted-indirect-args",
        first_execution.map(|execution| {
            execution
                .compaction_resources()
                .compacted_indirect_args_buffer()
        }),
        INDEXED_INDIRECT_ARGS_STRIDE_BYTES,
        HZB_FALLBACK_INDIRECT_STORAGE_USAGE,
    );
    bind_hzb_execution_buffer(
        device,
        graph,
        resources,
        HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
        "hzb-occlusion-visible-instance-index",
        first_execution.map(|execution| {
            execution
                .compaction_resources()
                .visible_instance_index_buffer()
        }),
        INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES,
        HZB_FALLBACK_STORAGE_USAGE,
    );
    bind_hzb_execution_buffer(
        device,
        graph,
        resources,
        HZB_OCCLUSION_DRAW_COUNT_RESOURCE,
        "hzb-occlusion-draw-count",
        first_execution.map(|execution| execution.compaction_resources().draw_count_buffer()),
        INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES,
        HZB_FALLBACK_INDIRECT_STORAGE_USAGE,
    );
    bind_hzb_execution_buffer(
        device,
        graph,
        resources,
        HZB_OCCLUSION_STATS_RESOURCE,
        "hzb-occlusion-stats",
        hzb_occlusion_culler.map(HzbOcclusionCuller::stats_buffer),
        HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE,
        HZB_FALLBACK_STORAGE_USAGE,
    );
}

fn graph_declares_any_hzb_occlusion_external(graph: &CompiledRenderGraph) -> bool {
    HZB_OCCLUSION_EXTERNAL_BUFFER_NAMES
        .iter()
        .any(|resource_name| graph.resource_lifetime_by_name(resource_name).is_some())
}

fn bind_hzb_execution_buffer(
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    fallback_label: &'static str,
    buffer: Option<&wgpu::Buffer>,
    fallback_size: wgpu::BufferAddress,
    fallback_usage: wgpu::BufferUsages,
) {
    if graph.resource_lifetime_by_name(logical_name).is_none() {
        return;
    }
    if let Some(buffer) = buffer {
        resources.bind_execution_owned_buffer(
            logical_name,
            execution_owned_backing_name(logical_name, false),
            buffer,
        );
        return;
    }

    let fallback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(fallback_label),
        size: fallback_size,
        usage: fallback_usage,
        mapped_at_creation: false,
    });
    resources.bind_execution_owned_buffer(
        logical_name,
        execution_owned_backing_name(logical_name, true),
        &fallback,
    );
}

fn execution_owned_backing_name(logical_name: &str, fallback: bool) -> String {
    if fallback {
        format!("{logical_name}:hzb-execution-fallback")
    } else {
        format!("{logical_name}:hzb-execution-phase0")
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

#[cfg(test)]
mod tests {
    use crate::graphics::backend::RenderBackend;
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphBuilder, RenderGraphExternalResourceBinding,
    };

    use super::*;

    #[test]
    fn hzb_external_fallback_buffers_satisfy_materialization_report() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = hzb_external_graph();
        let mut resources = RenderGraphExecutionResources::new();

        bind_hzb_occlusion_external_buffers(&backend.device, &graph, &mut resources, &[], None);

        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("HZB fallback buffers should bind declared externals");
        assert_eq!(report.required_external_count, 6);
        assert_eq!(report.bound_required_external_count, 6);
        assert_eq!(report.missing_required_external_count, 0);
        assert_eq!(report.report_only_external_count, 0);
        assert_eq!(report.bound_external_count(), 6);
        assert_eq!(report.missing_external_count(), 0);
        assert!(report.is_complete());
        let aliases = resources.resource_alias_report();
        assert_eq!(aliases.buffer_aliases.len(), 6);
        assert!(aliases.buffer_aliases.iter().any(|alias| {
            alias.logical_name == HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE
                && alias.backing_name.ends_with(":hzb-execution-fallback")
        }));
    }

    fn hzb_external_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("hzb-external-materialization");
        let indirect_args =
            required_external_buffer(&mut builder, HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE);
        let metadata =
            required_external_buffer(&mut builder, HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE);
        let compacted =
            required_external_buffer(&mut builder, HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE);
        let visible =
            required_external_buffer(&mut builder, HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE);
        let draw_count = required_external_buffer(&mut builder, HZB_OCCLUSION_DRAW_COUNT_RESOURCE);
        let stats = required_external_buffer(&mut builder, HZB_OCCLUSION_STATS_RESOURCE);
        let pass = builder.add_pass("hzb-occlusion-cull", QueueLane::AsyncCompute);
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    allow_culling: true,
                    has_side_effects: true,
                },
            )
            .unwrap();
        builder.read_external(pass, indirect_args).unwrap();
        builder.read_external(pass, metadata).unwrap();
        builder.write_storage_external(pass, compacted).unwrap();
        builder.write_storage_external(pass, visible).unwrap();
        builder.write_storage_external(pass, draw_count).unwrap();
        builder.write_storage_external(pass, stats).unwrap();
        builder.compile().unwrap()
    }

    fn required_external_buffer(
        builder: &mut RenderGraphBuilder,
        name: &'static str,
    ) -> crate::render_graph::ExternalResource {
        builder.import_external_resource_with_binding(
            name,
            RenderGraphExternalResourceBinding::required_buffer(),
        )
    }
}
