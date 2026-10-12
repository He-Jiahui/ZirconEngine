use std::hint::black_box;
use std::time::Instant;

use super::runtime_plugin_capability;

const SAMPLE_PAIRS: usize = 17;
const CAPABILITIES_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_fg_runtime463_plugin_capability_preserves_bytes() {
    for package_id in [
        "terrain",
        "tilemap_2d",
        "prefab_tools",
        "plugin.with.long.id",
    ] {
        assert_eq!(
            runtime_plugin_capability(package_id),
            format!("runtime.plugin.{package_id}")
        );
    }
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fg_runtime463_direct_plugin_capability_benchmark() {
    const PACKAGE_ID: &str = "plugin.world_partition.content_tools";
    for _ in 0..4 {
        black_box(measure_capabilities(
            |id| format!("runtime.plugin.{id}"),
            PACKAGE_ID,
        ));
        black_box(measure_capabilities(runtime_plugin_capability, PACKAGE_ID));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_capabilities(
                |id| format!("runtime.plugin.{id}"),
                PACKAGE_ID,
            ));
            optimized_samples.push(measure_capabilities(runtime_plugin_capability, PACKAGE_ID));
        } else {
            optimized_samples.push(measure_capabilities(runtime_plugin_capability, PACKAGE_ID));
            legacy_samples.push(measure_capabilities(
                |id| format!("runtime.plugin.{id}"),
                PACKAGE_ID,
            ));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn measure_capabilities(mut build: impl FnMut(&str) -> String, package_id: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..CAPABILITIES_PER_SAMPLE {
        let capability = black_box(build(black_box(package_id)));
        checksum = checksum.wrapping_add(capability.len());
        black_box(capability);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME463_DIRECT_PLUGIN_CAPABILITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} capabilities_per_sample={CAPABILITIES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
        "direct plugin capability construction must reduce P95 by at least 25%"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
