use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::scene::scene_renderer::core::scene_renderer_core::{
    HZB_INDIRECT_ARGS_NEUTRAL_BACKING, HzbNeutralBuffers, LightGridNeutralBuffers,
    SceneRendererNeutralGraphBuffers,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderPassMeshCommandLists,
};
use crate::graphics::scene::scene_renderer::hzb::{
    HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE, HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE,
    HZB_OCCLUSION_DRAW_COUNT_RESOURCE, HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE,
    HZB_OCCLUSION_STATS_RESOURCE, HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
    HzbOcclusionCuller,
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
        let mut neutral_buffers = SceneRendererNeutralGraphBuffers::default();

        bind_hzb_occlusion_external_buffers(
            &graph,
            &mut resources,
            None,
            None,
            neutral_buffers.hzb(&backend.device),
        );

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
                && alias.backing_name == HZB_INDIRECT_ARGS_NEUTRAL_BACKING
        }));
    }

    #[test]
    fn execution_owned_resource_binding_selects_the_first_hzb_execution_without_collecting() {
        let source = include_str!("bind_execution_owned_graph_resources.rs");
        let product = source
            .split_once("#[cfg(test)]")
            .expect("execution-owned binder should keep tests below product code")
            .0;
        assert!(!product.contains(".collect::<Vec<_>>()"));
        assert!(product.contains("let first_hzb_execution ="));
        assert!(product.contains(".flatten()"));
        assert!(product.contains(".next();"));
        assert!(product.contains("neutral_buffers.light_grid(device)"));
        assert!(product.contains("neutral_buffers.hzb(device)"));
        assert!(!product.contains("create_buffer"));
        assert!(!product.contains("create_buffer_init"));
        assert!(!product.contains("format!("));
    }

    #[test]
    fn light_grid_external_fallback_buffers_satisfy_materialization_report() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = light_grid_external_graph();
        let mut resources = RenderGraphExecutionResources::new();
        let mut neutral_buffers = SceneRendererNeutralGraphBuffers::default();

        bind_light_grid_external_buffers(
            &graph,
            &mut resources,
            neutral_buffers.light_grid(&backend.device),
        );

        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("light-grid fallback buffers should bind declared externals");
        assert_eq!(report.required_external_count, 3);
        assert_eq!(report.bound_required_external_count, 3);
        assert_eq!(report.missing_required_external_count, 0);
        assert_eq!(report.report_only_external_count, 0);
        assert_eq!(report.bound_external_count(), 3);
        assert_eq!(report.missing_external_count(), 0);
        assert!(report.is_complete());
        let aliases = resources.resource_alias_report();
        assert_eq!(aliases.buffer_aliases.len(), 3);
        for logical_name in LIGHT_GRID_EXTERNAL_BUFFER_NAMES {
            assert!(aliases.buffer_aliases.iter().any(|alias| {
                alias.logical_name == *logical_name && alias.backing_name.ends_with(":neutral")
            }));
        }
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

    fn light_grid_external_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("light-grid-external-materialization");
        let params = required_external_buffer(
            &mut builder,
            PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
        );
        let zbins =
            required_external_buffer(&mut builder, PostProcessGraphResourceNames::LIGHT_ZBINS);
        let tile_masks = required_external_buffer(
            &mut builder,
            PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
        );
        let pass = builder.add_pass("mesh-shading", QueueLane::Graphics);
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    allow_culling: true,
                    has_side_effects: true,
                },
            )
            .unwrap();
        builder.read_external(pass, params).unwrap();
        builder.read_external(pass, zbins).unwrap();
        builder.read_external(pass, tile_masks).unwrap();
        builder.compile().unwrap()
    }

    fn required_external_buffer(
        builder: &mut RenderGraphBuilder,
        name: &'static str,
    ) -> crate::render_graph::ExternalResource {
        builder.import_present_external_resource_with_binding(
            name,
            RenderGraphExternalResourceBinding::required_buffer(),
        )
    }
}
