use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const PROJECTIONS_PER_SAMPLE: usize = 64;
const LOADED_PLUGINS_PER_PROJECTION: usize = 4_096;

#[test]
fn optimization_batch_20260826ge_runtime226_map_capacity_covers_loaded_plugins() {
    let mut diagnostics =
        HashMap::<usize, [usize; 4]>::with_capacity(LOADED_PLUGINS_PER_PROJECTION);
    let mut loaded = HashMap::<usize, bool>::with_capacity(LOADED_PLUGINS_PER_PROJECTION);
    for plugin in 0..LOADED_PLUGINS_PER_PROJECTION {
        diagnostics.insert(plugin, [plugin; 4]);
        loaded.insert(plugin, plugin % 2 == 0);
    }

    assert_eq!(diagnostics.len(), LOADED_PLUGINS_PER_PROJECTION);
    assert_eq!(loaded.len(), LOADED_PLUGINS_PER_PROJECTION);
    assert!(diagnostics.capacity() >= LOADED_PLUGINS_PER_PROJECTION);
    assert!(loaded.capacity() >= LOADED_PLUGINS_PER_PROJECTION);
}

#[test]
fn optimization_batch_20260826ge_runtime226_projection_reserves_loaded_maps_only() {
    let source = include_str!("../projection.rs");

    assert_eq!(
        source
            .matches("HashMap::<String, PluginDiagnostics>::with_capacity(report.loaded.len())")
            .count(),
        1
    );
    assert_eq!(
        source
            .matches("HashMap::with_capacity(report.loaded.len())")
            .count(),
        1
    );
    assert!(source.contains("let mut descriptor_diagnostics = Vec::new();"));
    assert!(source.contains("let mut entry_diagnostics = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ge_runtime226_native_diagnostics_map_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME226_NATIVE_DIAGNOSTICS_MAP_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
projections_per_sample={PROJECTIONS_PER_SAMPLE} \
loaded_plugins_per_projection={LOADED_PLUGINS_PER_PROJECTION} \
legacy_preallocated_maps_per_projection=0 optimized_preallocated_maps_per_projection=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for projection in 0..PROJECTIONS_PER_SAMPLE {
        let mut diagnostics = if reserve {
            HashMap::<usize, [usize; 4]>::with_capacity(LOADED_PLUGINS_PER_PROJECTION)
        } else {
            HashMap::new()
        };
        let mut loaded = if reserve {
            HashMap::<usize, bool>::with_capacity(LOADED_PLUGINS_PER_PROJECTION)
        } else {
            HashMap::new()
        };
        for plugin in 0..LOADED_PLUGINS_PER_PROJECTION {
            let key = black_box(projection ^ plugin);
            diagnostics.insert(key, [key; 4]);
            loaded.insert(key, key % 2 == 0);
        }
        checksum ^= black_box(diagnostics.capacity() ^ loaded.capacity());
        black_box((&diagnostics, &loaded));
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
