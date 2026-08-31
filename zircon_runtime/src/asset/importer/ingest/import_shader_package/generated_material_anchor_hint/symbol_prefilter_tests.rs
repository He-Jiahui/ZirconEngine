use std::hint::black_box;
use std::time::Instant;

use super::{
    wgsl_needs_generated_material_anchor_hint, wgsl_source_without_comments,
    wgsl_uses_generated_material_symbol,
};
use crate::core::framework::render::{GENERATED_MATERIAL_MODULE_IMPORT_PATH, wgsl_include_paths};

const SAMPLE_PAIRS: usize = 21;
const CHECKS_PER_SAMPLE: usize = 512;

#[test]
fn optimization_batch_20260828ip_runtime288_prefilter_preserves_anchor_semantics() {
    assert!(!wgsl_needs_generated_material_anchor_hint(
        "fn surface() { let color = vec4<f32>(1.0); }"
    ));
    assert!(!wgsl_needs_generated_material_anchor_hint(
        "// zr_mat_base_color()\nfn surface() {}"
    ));
    assert!(wgsl_needs_generated_material_anchor_hint(
        "fn surface() { let color = zr_mat_base_color(); }"
    ));
    assert!(!wgsl_needs_generated_material_anchor_hint(
        "#include <self::material>\nfn surface() { zr_mat_base_color(); }"
    ));
}

#[test]
fn optimization_batch_20260828ip_runtime288_checks_symbol_before_comment_copy() {
    let source = include_str!("../generated_material_anchor_hint.rs");
    let helper_start = source
        .find("fn wgsl_needs_generated_material_anchor_hint")
        .expect("prefilter helper");
    let helper = &source[helper_start..];

    assert!(helper.contains("if !wgsl_source.contains(\"zr_mat_\")"));
    assert!(
        helper.find("contains(\"zr_mat_\")").unwrap()
            < helper
                .find("wgsl_source_without_comments(wgsl_source)")
                .unwrap()
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828ip_runtime288_wgsl_generated_symbol_prefilter_bench() {
    let source = "fn shade_surface() { let color = vec4<f32>(1.0); }\n".repeat(1_024);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&source, false));
            optimized_samples.push(measure(&source, true));
        } else {
            optimized_samples.push(measure(&source, true));
            legacy_samples.push(measure(&source, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME288_WGSL_GENERATED_SYMBOL_PREFILTER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} source_bytes={} \
legacy_source_copies_per_sample={CHECKS_PER_SAMPLE} optimized_source_copies_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        source.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_needs_generated_material_anchor_hint(wgsl_source: &str) -> bool {
    let authored_source = wgsl_source_without_comments(wgsl_source);
    !wgsl_include_paths(&authored_source)
        .iter()
        .any(|path| path == GENERATED_MATERIAL_MODULE_IMPORT_PATH)
        && wgsl_uses_generated_material_symbol(&authored_source)
}

fn measure(source: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = false;
    for _ in 0..CHECKS_PER_SAMPLE {
        checksum ^= if optimized {
            black_box(wgsl_needs_generated_material_anchor_hint(black_box(source)))
        } else {
            black_box(legacy_needs_generated_material_anchor_hint(black_box(
                source,
            )))
        };
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
