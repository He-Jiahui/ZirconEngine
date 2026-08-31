use std::hint::black_box;
use std::time::Instant;

use super::*;

const DESCRIPTOR_STRING_COUNT: usize = 512;
const OPERATIONS_PER_SAMPLE: usize = 512;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED: RenderFeatureCapabilityRequirement =
    RenderFeatureCapabilityRequirement::PipelineStatisticsQuery;

#[test]
fn optimization_batch_20260826he_runtime251_preserves_feature_capability_sources() {
    let mut descriptor = RenderFeatureDescriptor::new("plugin", Vec::new(), Vec::new(), Vec::new());
    descriptor
        .capability_requirements
        .push(RenderFeatureCapabilityRequirement::AsyncCompute);
    let descriptor_asset = RendererFeatureAsset::plugin(descriptor);
    assert!(descriptor_asset.requires_capability(RenderFeatureCapabilityRequirement::AsyncCompute));
    assert!(!descriptor_asset.requires_capability(REQUIRED));

    let local_asset = descriptor_asset
        .clone()
        .with_capability_requirement(REQUIRED);
    assert!(local_asset.requires_capability(REQUIRED));
}

#[test]
fn optimization_batch_20260826he_runtime251_borrows_descriptor_override() {
    let source = include_str!("../renderer_feature_asset.rs");
    let start = source
        .find("pub fn requires_capability(")
        .expect("requires_capability function");
    let end = source[start..]
        .find("\n    pub fn with_enabled")
        .map(|offset| start + offset)
        .expect("next function boundary");
    let body = &source[start..end];

    assert!(body.contains("self.descriptor_override.as_ref()"));
    assert!(body.contains("descriptor.capability_requirements.contains"));
    assert!(!body.contains("self.descriptor()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826he_runtime251_renderer_feature_borrowed_capability_release_benchmark()
{
    let mut descriptor = RenderFeatureDescriptor::new(
        "plugin",
        (0..DESCRIPTOR_STRING_COUNT)
            .map(|index| format!("extract-section-{index:04}-with-retained-metadata"))
            .collect(),
        Vec::new(),
        Vec::new(),
    );
    descriptor.capability_requirements.push(REQUIRED);
    let asset = RendererFeatureAsset::plugin(descriptor);
    assert_eq!(
        asset.requires_capability(REQUIRED),
        legacy_requires_capability(&asset, REQUIRED)
    );

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_requires_capability(black_box(&asset), REQUIRED));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(asset.requires_capability(black_box(REQUIRED)));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME251_RENDERER_FEATURE_BORROWED_CAPABILITY_BENCH_V1 \
         descriptor_strings={DESCRIPTOR_STRING_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_requires_capability(
    asset: &RendererFeatureAsset,
    requirement: RenderFeatureCapabilityRequirement,
) -> bool {
    asset.capability_requirements.contains(&requirement)
        || asset
            .descriptor()
            .capability_requirements
            .contains(&requirement)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
