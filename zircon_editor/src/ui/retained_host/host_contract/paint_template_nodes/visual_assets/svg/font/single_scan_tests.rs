use std::hint::black_box;
use std::time::Instant;

use super::svg_may_need_fonts;

const SVG_BYTES: usize = 64 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 256;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826he_editor197_preserves_svg_font_markers() {
    assert!(svg_may_need_fonts(b"<svg><text>hello</text></svg>"));
    assert!(svg_may_need_fonts(b"<svg><tspan>hello</tspan></svg>"));
    assert!(svg_may_need_fonts(
        b"<svg><path style=\"font-family: Inter\"/></svg>"
    ));
    assert!(!svg_may_need_fonts(b"<svg><path d=\"M0 0L1 1\"/></svg>"));
    assert!(!svg_may_need_fonts(
        b"<svg><TEXT>case sensitive</TEXT></svg>"
    ));
    assert!(!svg_may_need_fonts(&[0xff, 0xfe, 0xfd]));
}

#[test]
fn optimization_batch_20260826he_editor197_scans_svg_bytes_once() {
    let source = include_str!("../font.rs");
    let start = source
        .find("fn svg_may_need_fonts(")
        .expect("svg_may_need_fonts function");
    let end = source[start..]
        .find("\n#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("test module boundary");
    let body = &source[start..end];

    assert!(body.contains("while index < svg.len()"));
    assert!(body.contains("starts_with(b\"font-family\")"));
    assert!(!body.contains("svg.contains("));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826he_editor197_svg_font_single_scan_release_benchmark() {
    let mut svg = Vec::with_capacity(SVG_BYTES);
    svg.extend_from_slice(b"<svg><path d=\"");
    svg.resize(SVG_BYTES - "\"/></svg>".len(), b'M');
    svg.extend_from_slice(b"\"/></svg>");
    assert!(!legacy_svg_may_need_fonts(&svg));
    assert!(!svg_may_need_fonts(&svg));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_svg_may_need_fonts(black_box(&svg)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(svg_may_need_fonts(black_box(&svg)));
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
        "EDITOR197_SVG_FONT_SINGLE_SCAN_BENCH_V1 svg_bytes={SVG_BYTES} \
         operations_per_sample={OPERATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} \
         legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
         optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} \
         legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_svg_may_need_fonts(svg: &[u8]) -> bool {
    let Ok(svg) = std::str::from_utf8(svg) else {
        return false;
    };
    svg.contains("<text") || svg.contains("<tspan") || svg.contains("font-family")
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
