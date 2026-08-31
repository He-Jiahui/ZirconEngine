use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use super::CompiledRenderPipelineResourceWriteIndex;
use crate::render_graph::{
    CompiledRenderGraph, QueueLane, RenderGraphBuilder, RenderGraphResourceAccessKind,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 512;
const RESOURCE_COUNT: usize = 256;

#[test]
fn optimization_batch_20260826ek_runtime180_direct_write_bits_preserve_access_semantics() {
    let mut builder = RenderGraphBuilder::new("runtime180-access-semantics");
    let written = builder.import_present_external_resource("runtime180-written");
    let read_only = builder.import_present_external_resource("runtime180-read-only");
    let pass = builder.add_pass_with_executor(
        "runtime180-pass",
        QueueLane::Graphics,
        Some("runtime180.executor"),
    );
    builder
        .read_external(pass, read_only)
        .expect("fixture should read the external resource");
    builder
        .write_external(pass, written)
        .expect("fixture should write the external resource");
    let graph = builder.compile().expect("fixture graph should compile");

    let index = CompiledRenderPipelineResourceWriteIndex::from_graph(&graph);

    assert!(index.contains("runtime180-written"));
    assert!(!index.contains("runtime180-read-only"));
    assert!(!index.contains("runtime180-missing"));
    assert_eq!(index.build_stats(), (1, 2));
}

#[test]
fn optimization_batch_20260826ek_runtime180_builds_write_bits_in_the_access_pass() {
    let source = include_str!("../resource_write_index.rs");
    let builder_start = source.find("pub(super) fn from_graph").unwrap();
    let builder_end = source[builder_start..]
        .find("pub(super) fn contains")
        .map(|offset| builder_start + offset)
        .unwrap();
    let builder_source = &source[builder_start..builder_end];

    assert!(builder_source.contains("HashMap::with_capacity(resource_capacity)"));
    assert!(builder_source.contains("Vec::with_capacity(resource_capacity.div_ceil(64))"));
    assert!(builder_source.contains("write_bits.push(0)"));
    assert!(builder_source.contains("write_bits[index / 64] |="));
    assert!(!builder_source.contains("resource_write_flags"));
    assert!(!builder_source.contains("for (index, written)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ek_runtime180_resource_write_index_direct_bits_bench() {
    let graph = graph_fixture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&graph));
            optimized_samples.push(measure_optimized(&graph));
        } else {
            optimized_samples.push(measure_optimized(&graph));
            legacy_samples.push(measure_legacy(&graph));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME180_RESOURCE_WRITE_INDEX_DIRECT_BITS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} resources_per_build={RESOURCE_COUNT} \
legacy_intermediate_flags_per_build={RESOURCE_COUNT} optimized_intermediate_flags_per_build=0 \
legacy_index_passes=2 optimized_index_passes=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct write-bit construction P95 {optimized_p95_ns}ns must be at most 70% of intermediate-flag construction P95 {legacy_p95_ns}ns"
    );
}

fn graph_fixture() -> CompiledRenderGraph {
    let mut builder = RenderGraphBuilder::new("runtime180-benchmark");
    let pass = builder.add_pass_with_executor(
        "runtime180-benchmark-pass",
        QueueLane::Graphics,
        Some("runtime180.benchmark"),
    );
    for index in 0..RESOURCE_COUNT {
        let resource =
            builder.import_present_external_resource(format!("runtime180-resource-{index}"));
        builder
            .write_external(pass, resource)
            .expect("fixture should write every external resource");
    }
    builder.compile().expect("benchmark graph should compile")
}

fn legacy_from_graph(graph: &CompiledRenderGraph) -> CompiledRenderPipelineResourceWriteIndex {
    let mut resource_indices = HashMap::new();
    let mut resource_write_flags = Vec::new();
    let mut executable_pass_count = 0;
    let mut resource_access_count = 0;
    for pass in graph.passes().iter().filter(|pass| !pass.culled) {
        executable_pass_count += 1;
        for access in &pass.resources {
            resource_access_count += 1;
            let index = match resource_indices.get(access.name.as_str()).copied() {
                Some(index) => index,
                None => {
                    let index = resource_indices.len();
                    resource_indices.insert(access.name.clone(), index);
                    resource_write_flags.push(false);
                    index
                }
            };
            if access.access == RenderGraphResourceAccessKind::Write {
                resource_write_flags[index] = true;
            }
        }
    }
    let mut write_bits = vec![0_u64; resource_indices.len().div_ceil(64)];
    for (index, written) in resource_write_flags.into_iter().enumerate() {
        if written {
            write_bits[index / 64] |= 1_u64 << (index % 64);
        }
    }
    CompiledRenderPipelineResourceWriteIndex {
        resource_indices,
        write_bits: write_bits.into_boxed_slice(),
        executable_pass_count,
        resource_access_count,
    }
}

fn measure_legacy(graph: &CompiledRenderGraph) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let index = black_box(legacy_from_graph(black_box(graph)));
        checksum ^= index.resource_indices.len() ^ index.write_bits.len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(graph: &CompiledRenderGraph) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let index = black_box(CompiledRenderPipelineResourceWriteIndex::from_graph(
            black_box(graph),
        ));
        checksum ^= index.resource_indices.len() ^ index.write_bits.len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
