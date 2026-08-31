use std::hint::black_box;
use std::time::Instant;

use super::{skeleton_frame_variant, skeleton_radius_variant};

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const VARIANT_BYTES: usize = 4096;

fn legacy_frame_variant(value: &str) -> u8 {
    if value
        .split(|c: char| c.is_ascii_whitespace() || matches!(c, ',' | '/' | '|' | ':' | ';'))
        .any(|p| p.eq_ignore_ascii_case("circular"))
    {
        1
    } else if value
        .split(|c: char| c.is_ascii_whitespace() || matches!(c, ',' | '/' | '|' | ':' | ';'))
        .any(|p| p.eq_ignore_ascii_case("text"))
    {
        2
    } else {
        0
    }
}
fn legacy_radius_variant(value: &str) -> u8 {
    if value
        .split(|c: char| c.is_ascii_whitespace() || matches!(c, ',' | '/' | '|' | ':' | ';'))
        .any(|p| p.eq_ignore_ascii_case("rectangular"))
    {
        1
    } else if value
        .split(|c: char| c.is_ascii_whitespace() || matches!(c, ',' | '/' | '|' | ':' | ';'))
        .any(|p| p.eq_ignore_ascii_case("circular"))
    {
        2
    } else {
        0
    }
}
fn measure(value: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut sum = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        sum += if optimized {
            skeleton_frame_variant(black_box(value))
        } else {
            legacy_frame_variant(black_box(value))
        };
        sum += if optimized {
            skeleton_radius_variant(black_box(value))
        } else {
            legacy_radius_variant(black_box(value))
        };
    }
    black_box(sum);
    started.elapsed().as_nanos().max(1)
}
fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut s = samples.to_vec();
    s.sort_unstable();
    let r = (s.len() * p).div_ceil(100);
    s[r.saturating_sub(1)]
}
fn csv(s: &[u128]) -> String {
    s.iter().map(u128::to_string).collect::<Vec<_>>().join(",")
}
#[test]
fn optimization_batch_20260829bm_editor285_skeleton_geometry_preserves_results() {
    for v in [
        "circular",
        "text",
        "rectangular",
        "filled circular",
        "",
        "\u{4f8b}",
    ] {
        assert_eq!(skeleton_frame_variant(v), legacy_frame_variant(v));
        assert_eq!(skeleton_radius_variant(v), legacy_radius_variant(v));
    }
}
#[test]
fn optimization_batch_20260829bm_editor285_skeleton_geometry_uses_one_scan() {
    let s = include_str!("../geometry.rs");
    let p = s.split_once("#[cfg(test)]").unwrap().0;
    assert!(p.contains("skeleton_frame_variant(&node.component_variant)"));
    assert!(p.contains("skeleton_radius_variant(&node.component_variant)"));
}
#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bm_editor285_single_scan_skeleton_geometry_bench() {
    let v = "x".repeat(VARIANT_BYTES);
    let mut b = Vec::with_capacity(SAMPLE_PAIRS);
    let mut c = Vec::with_capacity(SAMPLE_PAIRS);
    for i in 0..SAMPLE_PAIRS {
        if i % 2 == 0 {
            b.push(measure(&v, false));
            c.push(measure(&v, true));
        } else {
            c.push(measure(&v, true));
            b.push(measure(&v, false));
        }
    }
    let bp = percentile(&b, 95);
    let cp = percentile(&c, 95);
    println!("EDITOR285_SINGLE_SCAN_SKELETON_GEOMETRY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} variant_bytes={VARIANT_BYTES} baseline_variant_scans=4 candidate_variant_scans=2 baseline_p95_ns={bp} candidate_p95_ns={cp} baseline_raw_ns={} candidate_raw_ns={}",csv(&b),csv(&c));
    assert!(cp.saturating_mul(100) <= bp.saturating_mul(70));
}
