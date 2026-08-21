use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::graphics::RenderFeatureCapabilityRequirement;
use crate::render_graph::{QueueLane, RenderGraphBuilder};

use super::runtime_metadata::CompiledRenderPipelineRuntimeMetadata;

const RUNTIME89_LOOKUP_PASS_COUNT: usize = 2_048;
const RUNTIME89_LOOKUP_SAMPLE_PAIRS: usize = 21;

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
#[ignore = "release performance gate"]
fn runtime89_compiled_pass_identity_lookup_beats_name_scan_p95() {
    let mut builder = RenderGraphBuilder::new("runtime89-pass-identity-lookup");
    let mut pass_stages = Vec::with_capacity(RUNTIME89_LOOKUP_PASS_COUNT);
    let mut final_pass = None;
    for index in 0..RUNTIME89_LOOKUP_PASS_COUNT {
        let pass_name = format!("runtime89-pass-{index:04}");
        let pass_id = builder.add_pass(pass_name.clone(), QueueLane::Graphics);
        pass_stages.push(super::CompiledRenderPipelinePassStage::new(
            pass_id,
            pass_name,
            super::RenderPassStage::PostProcess,
        ));
        final_pass = Some(pass_id);
    }
    builder
        .set_pass_flags(
            final_pass.expect("benchmark pass"),
            crate::render_graph::PassFlags {
                has_side_effects: true,
                ..crate::render_graph::PassFlags::default()
            },
        )
        .expect("benchmark cull root");
    let graph = builder.compile().expect("benchmark graph should compile");

    let _ = legacy_name_lookup_sample(&graph, &pass_stages);
    let _ = direct_identity_lookup_sample(&graph, &pass_stages);
    let mut legacy_samples = Vec::with_capacity(RUNTIME89_LOOKUP_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(RUNTIME89_LOOKUP_SAMPLE_PAIRS);
    let mut legacy_comparisons = 0_usize;
    let mut optimized_lookups = 0_usize;
    for pair in 0..RUNTIME89_LOOKUP_SAMPLE_PAIRS {
        if pair % 2 == 0 {
            let (elapsed, work) = legacy_name_lookup_sample(&graph, &pass_stages);
            legacy_samples.push(elapsed);
            legacy_comparisons = work;
            let (elapsed, work) = direct_identity_lookup_sample(&graph, &pass_stages);
            optimized_samples.push(elapsed);
            optimized_lookups = work;
        } else {
            let (elapsed, work) = direct_identity_lookup_sample(&graph, &pass_stages);
            optimized_samples.push(elapsed);
            optimized_lookups = work;
            let (elapsed, work) = legacy_name_lookup_sample(&graph, &pass_stages);
            legacy_samples.push(elapsed);
            legacy_comparisons = work;
        }
    }

    let legacy_p95_ns = nearest_rank_duration(&legacy_samples, 95).as_nanos();
    let optimized_p95_ns = nearest_rank_duration(&optimized_samples, 95).as_nanos();
    let ratio_pct = optimized_p95_ns.saturating_mul(100) / legacy_p95_ns.max(1);
    println!(
        "RUNTIME89_COMPILED_PASS_IDENTITY_TIME workload=compiled_pass_identity_lookup pass_count={} sample_pairs={} alternation=legacy_first_even_pair legacy_first_pairs=11 optimized_first_pairs=10 legacy_p95_ns={} optimized_p95_ns={} legacy_name_comparisons={} optimized_id_lookups={} ratio_pct={} legacy_ns={} optimized_ns={}",
        RUNTIME89_LOOKUP_PASS_COUNT,
        RUNTIME89_LOOKUP_SAMPLE_PAIRS,
        legacy_p95_ns,
        optimized_p95_ns,
        legacy_comparisons,
        optimized_lookups,
        ratio_pct,
        join_duration_ns(&legacy_samples),
        join_duration_ns(&optimized_samples)
    );

    assert_eq!(
        legacy_comparisons,
        RUNTIME89_LOOKUP_PASS_COUNT * (RUNTIME89_LOOKUP_PASS_COUNT + 1) / 2
    );
    assert_eq!(optimized_lookups, RUNTIME89_LOOKUP_PASS_COUNT);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(25),
        "direct pass identity lookup P95 must be at least 75% faster: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_name_lookup_sample(
    graph: &crate::render_graph::CompiledRenderGraph,
    pass_stages: &[super::CompiledRenderPipelinePassStage],
) -> (Duration, usize) {
    let started = Instant::now();
    let mut comparisons = 0_usize;
    let mut checksum = 0_usize;
    for stage_entry in black_box(pass_stages) {
        for pass in black_box(graph.passes()) {
            comparisons += 1;
            if pass.name == stage_entry.pass_name {
                checksum ^= pass.id.index();
                break;
            }
        }
    }
    black_box(checksum);
    (started.elapsed(), comparisons)
}

fn direct_identity_lookup_sample(
    graph: &crate::render_graph::CompiledRenderGraph,
    pass_stages: &[super::CompiledRenderPipelinePassStage],
) -> (Duration, usize) {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for stage_entry in black_box(pass_stages) {
        let (_, pass) = black_box(graph)
            .indexed_pass(stage_entry.pass_id)
            .expect("compiled pass identity");
        checksum ^= pass.id.index();
    }
    black_box(checksum);
    (started.elapsed(), pass_stages.len())
}

fn nearest_rank_duration(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1).min(sorted.len().saturating_sub(1))]
}

fn join_duration_ns(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
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
