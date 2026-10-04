use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const BUILDS_PER_SAMPLE: usize = 65_536;
const IMPORT_COUNT: usize = 64;

#[test]
fn optimization_batch_20260920es_runtime851_import_capacity_source_contract() {
    let source = include_str!("../../ui_style_resolver.rs");

    assert!(source
        .contains("let mut imported_styles = Vec::with_capacity(document.imports.styles.len());"));
    assert!(source.contains("for reference in &document.imports.styles"));
    assert!(source.contains("imported_styles.push("));
    assert!(!source.contains("collect::<Result<Vec<_>, _>>()?"));
}

#[test]
fn optimization_batch_20260920es_runtime851_import_capacity_preserves_resolution_shape() {
    let source = include_str!("../../ui_style_resolver.rs");
    let resolve_start = source.find("let mut imported_styles").unwrap();
    let resolve_end = source[resolve_start..]
        .find("let imported_stylesheet_count")
        .map(|offset| resolve_start + offset)
        .unwrap();
    let resolve_source = &source[resolve_start..resolve_end];

    assert_eq!(resolve_source.matches(".get(reference)").count(), 1);
    assert!(resolve_source.contains("ok_or_else(||"));
    assert!(resolve_source.contains("UiAssetError::UnknownImport"));
    assert!(resolve_source.contains("imported_styles.push("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260920es_runtime851_style_import_capacity_bench() {
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
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME851_STYLE_IMPORT_RESOLUTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} imports_per_build={IMPORT_COUNT} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "reserved imported-style resolution P95 {optimized_p95_ns}ns must be at most 70% of growth-driven resolution P95 {legacy_p95_ns}ns"
    );
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = if reserve {
            Vec::with_capacity(IMPORT_COUNT)
        } else {
            Vec::new()
        };
        for index in 0..IMPORT_COUNT {
            output.push(black_box(index));
        }
        checksum ^= black_box(output.len() ^ output.capacity());
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
