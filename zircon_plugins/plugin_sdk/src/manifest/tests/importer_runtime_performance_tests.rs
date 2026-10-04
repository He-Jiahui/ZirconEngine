use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::asset::AssetKind;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const MANIFEST_BUILDS_PER_SAMPLE: usize = 256;
const CAPABILITIES_PER_BUILDER: usize = 32;
const IMPORTERS_PER_BUILDER: usize = 8;

#[test]
fn importer_runtime_manifest_builder_move_preserves_complete_manifest() {
    let descriptor = benchmark_descriptor();
    let builder = benchmark_builder();

    let legacy = builder.clone().legacy_build_package_manifest(&descriptor);
    let optimized = builder.build_package_manifest(&descriptor);

    assert_eq!(optimized, legacy);
}

#[test]
#[ignore = "release-only performance contract"]
fn benchmark_importer_runtime_manifest_builder_move_projection() {
    let descriptor = benchmark_descriptor();
    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_raw.push(measure_manifest_builds(
                ImporterRuntimeManifestBuilder::legacy_build_package_manifest,
                &descriptor,
            ));
            optimized_raw.push(measure_manifest_builds(
                ImporterRuntimeManifestBuilder::build_package_manifest,
                &descriptor,
            ));
        } else {
            optimized_raw.push(measure_manifest_builds(
                ImporterRuntimeManifestBuilder::build_package_manifest,
                &descriptor,
            ));
            legacy_raw.push(measure_manifest_builds(
                ImporterRuntimeManifestBuilder::legacy_build_package_manifest,
                &descriptor,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
        "move-backed importer manifest assembly must improve P95 by at least 20%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "PERF_RESULT task=plugins07_move_importer_manifest_builder sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank manifest_builds_per_sample={MANIFEST_BUILDS_PER_SAMPLE} capabilities_per_builder={CAPABILITIES_PER_BUILDER} importers_per_builder={IMPORTERS_PER_BUILDER} legacy_builder_field_clone_allocations_per_build=37 optimized_builder_field_clone_allocations_per_build=1 legacy_capability_clones_per_build=32 optimized_capability_clones_per_build=0 threshold_percent=20 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_raw),
        raw_samples(&optimized_raw)
    );
}

fn measure_manifest_builds(
    build: fn(ImporterRuntimeManifestBuilder, &RuntimePluginDescriptor) -> PluginPackageManifest,
    descriptor: &RuntimePluginDescriptor,
) -> u128 {
    let builders = (0..MANIFEST_BUILDS_PER_SAMPLE)
        .map(|_| benchmark_builder())
        .collect::<Vec<_>>();
    let started = Instant::now();
    for builder in builders {
        black_box(build(builder, black_box(descriptor)));
    }
    started.elapsed().as_nanos()
}

fn benchmark_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        "benchmark.runtime",
        "zircon_plugin_benchmark_runtime",
        "benchmark.dist",
        "zircon_plugin_benchmark_dist",
        "zircon_plugin_benchmark_runtime_entry_v3",
    )
    .with_capabilities(
        (0..CAPABILITIES_PER_BUILDER)
            .map(|index| format!("runtime.asset.importer.benchmark.capability_{index}")),
    )
    .with_asset_importers((0..IMPORTERS_PER_BUILDER).map(|index| {
        AssetImporterDescriptor::new(
            format!("benchmark.importer_{index}"),
            "benchmark_importer",
            AssetKind::Data,
            1,
        )
        .with_source_extensions([format!("benchmark_{index}")])
    }))
}

fn benchmark_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        "benchmark_importer",
        "Benchmark Importer",
        zircon_runtime::builtin::RuntimePluginId::GltfImporter,
        "zircon_plugin_benchmark_runtime",
    )
    .build()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
