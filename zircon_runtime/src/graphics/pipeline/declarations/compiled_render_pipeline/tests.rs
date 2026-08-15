use std::sync::Arc;

use crate::graphics::RenderFeatureCapabilityRequirement;
use crate::render_graph::{QueueLane, RenderGraphBuilder};

use super::runtime_metadata::CompiledRenderPipelineRuntimeMetadata;

#[test]
fn render01_compiled_pipeline_runtime_metadata_freezes_descriptor_capability_flags() {
    let graph = RenderGraphBuilder::new("runtime-metadata-capability-flags")
        .compile()
        .unwrap();
    let metadata = CompiledRenderPipelineRuntimeMetadata::from_compiled_inputs(
        &[],
        &[RenderFeatureCapabilityRequirement::ScreenSpaceAntiAlias],
        &graph,
    );

    assert!(
        metadata
            .runtime_feature_flags()
            .screen_space_anti_alias_capability_enabled
    );
}

#[test]
fn render01_compiled_pipeline_runtime_metadata_builds_resource_write_index_once_for_scaled_graphs()
{
    for pass_count in [10_usize, 100, 500] {
        let mut builder = RenderGraphBuilder::new(format!("runtime-metadata-{pass_count}"));
        let mut written_resources = Vec::with_capacity(pass_count);
        for index in 0..pass_count {
            let resource_name = format!("runtime-metadata-output-{index}");
            let pass = builder.add_pass_with_executor(
                format!("runtime-metadata-pass-{index}"),
                QueueLane::Graphics,
                Some("runtime-metadata.executor"),
            );
            let resource = builder.import_external_resource(resource_name.clone());
            builder.write_external(pass, resource).unwrap();
            written_resources.push(resource_name);
        }
        let graph = builder.compile().unwrap();
        let metadata =
            CompiledRenderPipelineRuntimeMetadata::from_compiled_inputs(&[], &[], &graph);
        let equivalent =
            CompiledRenderPipelineRuntimeMetadata::from_compiled_inputs(&[], &[], &graph);
        let build_stats = metadata.build_stats();
        let storage_before = metadata.resource_write_storage_snapshot();

        assert_ne!(
            metadata.validation_generation(),
            equivalent.validation_generation()
        );
        assert_eq!(
            metadata, equivalent,
            "validation identity must not change public compiled-pipeline equality"
        );
        assert_eq!(build_stats.0, pass_count);
        assert_eq!(build_stats.1, pass_count);
        for _ in 0..128 {
            for resource_name in &written_resources {
                assert!(metadata.writes_resource(resource_name));
            }
            assert!(!metadata.writes_resource("runtime-metadata-missing"));
        }

        assert_eq!(metadata.build_stats(), build_stats);
        assert_eq!(metadata.resource_write_storage_snapshot(), storage_before);
    }
}

#[test]
fn render01_compiled_pipeline_runtime_metadata_lazily_shares_graph_dump_per_generation() {
    let mut builder = RenderGraphBuilder::new("runtime-metadata-graph-dump");
    let pass = builder.add_pass_with_executor(
        "runtime-metadata-graph-dump-pass",
        QueueLane::Graphics,
        Some("runtime-metadata.executor"),
    );
    let output = builder.import_external_resource("runtime-metadata-graph-dump-output");
    builder.write_external(pass, output).unwrap();
    let graph = builder.compile().unwrap();
    let metadata = CompiledRenderPipelineRuntimeMetadata::from_compiled_inputs(&[], &[], &graph);

    let first = metadata.graph_dump_text(&graph);
    let second = metadata.graph_dump_text(&graph);

    assert!(Arc::ptr_eq(&first, &second));
    assert!(first.contains("runtime-metadata-graph-dump-pass"));
}
