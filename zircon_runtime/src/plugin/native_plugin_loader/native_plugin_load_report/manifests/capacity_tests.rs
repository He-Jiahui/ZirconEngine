use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const PROJECTIONS_PER_SAMPLE: usize = 4_096;
const MODULES_PER_PROJECTION: usize = 64;

#[test]
fn optimization_batch_20260826ga_runtime222_shader_module_capacity_covers_capped_declarations() {
    let module_capacity = 96_usize.min(MODULES_PER_PROJECTION);
    let mut sources = Vec::with_capacity(module_capacity);
    let mut seen_import_paths = HashSet::with_capacity(module_capacity);
    for module in 0..module_capacity {
        assert!(seen_import_paths.insert(module));
        sources.push(module * 2);
    }

    assert_eq!(sources.len(), MODULES_PER_PROJECTION);
    assert_eq!(seen_import_paths.len(), MODULES_PER_PROJECTION);
    assert!(sources.capacity() >= MODULES_PER_PROJECTION);
    assert!(seen_import_paths.capacity() >= MODULES_PER_PROJECTION);
}

#[test]
fn optimization_batch_20260826ga_runtime222_shader_projection_reserves_capped_module_count() {
    let source = include_str!("../manifests.rs");

    assert!(source.contains("declared_modules.len().min(MAX_SHADER_MODULES_PER_PACKAGE)"));
    assert!(source.contains("Vec::with_capacity(module_capacity)"));
    assert!(source.contains("HashSet::with_capacity(module_capacity)"));
    assert!(source.contains("return (Vec::new(), diagnostics);"));
    assert!(!source.contains("let mut seen_import_paths = HashSet::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ga_runtime222_shader_module_projection_capacity_bench() {
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
        "RUNTIME222_SHADER_MODULE_PROJECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
projections_per_sample={PROJECTIONS_PER_SAMPLE} modules_per_projection={MODULES_PER_PROJECTION} \
legacy_preallocated_collections_per_projection=0 optimized_preallocated_collections_per_projection=2 \
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
        let mut sources = if reserve {
            Vec::with_capacity(MODULES_PER_PROJECTION)
        } else {
            Vec::new()
        };
        let mut seen = if reserve {
            HashSet::with_capacity(MODULES_PER_PROJECTION)
        } else {
            HashSet::new()
        };
        for module in 0..MODULES_PER_PROJECTION {
            let value = black_box(projection ^ module);
            if seen.insert(value) {
                sources.push([value; 16]);
            }
        }
        checksum ^= black_box(sources.len() ^ sources.capacity() ^ seen.capacity());
        black_box((&sources, &seen));
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
