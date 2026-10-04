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
    let source = include_str!("../bind_execution_owned_graph_resources.rs");
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
    let zbins = required_external_buffer(&mut builder, PostProcessGraphResourceNames::LIGHT_ZBINS);
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
