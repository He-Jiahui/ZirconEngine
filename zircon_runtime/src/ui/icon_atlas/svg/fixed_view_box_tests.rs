use std::hint::black_box;
use std::time::Instant;

use super::{
    parse_ui_svg_icon_cached, parse_view_box, SvgDocumentCache, SVG_DOCUMENT_CACHE_CAPACITY,
};

const SAMPLE_PAIRS: usize = 21;
const OPERATIONS_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_20260826hc_runtime249_preserves_view_box_parse_contract() {
    assert_eq!(parse_view_box("0 1 24 48"), Some((0.0, 1.0, 24.0, 48.0)));
    assert_eq!(parse_view_box("0, 1,24,48"), Some((0.0, 1.0, 24.0, 48.0)));
    assert_eq!(parse_view_box("0 1 24"), None);
    assert_eq!(parse_view_box("0 1 24 48 96"), None);
    assert_eq!(parse_view_box("0 1 invalid 48"), None);
}

#[test]
fn optimization_batch_20260826hc_runtime249_uses_fixed_view_box_slots() {
    let source = include_str!("../svg.rs");
    let start = source
        .find("fn parse_view_box(")
        .expect("parse_view_box function");
    let end = source[start..]
        .find("\nfn parse_svg_number")
        .map(|offset| start + offset)
        .expect("next function boundary");
    let body = &source[start..end];

    assert!(body.contains("let mut values = [0.0_f32; 4]"));
    assert!(body.contains("slot_count"));
    assert!(!body.contains("collect::<Vec"));
}

#[test]
fn optimization_batch_20260830_svg_document_cache_reuses_valid_documents() {
    let source = r#"<svg viewBox="0 0 24 24"><path d="M0 0h1v1z" /></svg>"#;
    let first = parse_ui_svg_icon_cached(source).expect("valid SVG should parse");
    let second = parse_ui_svg_icon_cached(source).expect("cached SVG should parse");
    assert_eq!(first, second);
}

#[test]
fn optimization_batch_20260830_svg_document_cache_does_not_retain_failures() {
    let source = "<svg viewBox=\"0 0 24 24\"></svg>";
    assert!(parse_ui_svg_icon_cached(source).is_err());
    assert!(parse_ui_svg_icon_cached(source).is_err());
}

#[test]
fn optimization_batch_20260830_svg_document_cache_is_bounded() {
    let mut cache = SvgDocumentCache::default();
    for index in 0..SVG_DOCUMENT_CACHE_CAPACITY + 1 {
        let source = format!("<svg><path d=\"M{index} 0h1v1z\" /></svg>");
        let document = super::parse_ui_svg_icon(&source).expect("valid SVG should parse");
        cache.insert(&source, document);
    }
    assert!(cache.documents.len() <= SVG_DOCUMENT_CACHE_CAPACITY);
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hc_runtime249_svg_view_box_fixed_parse_release_benchmark() {
    const VIEW_BOX: &str = "0,0,24,24";
    assert_eq!(parse_view_box(VIEW_BOX), legacy_parse_view_box(VIEW_BOX));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_parse_view_box(black_box(VIEW_BOX)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(parse_view_box(black_box(VIEW_BOX)));
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
        "RUNTIME249_SVG_VIEW_BOX_FIXED_PARSE_BENCH_V1 \
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

fn legacy_parse_view_box(raw: &str) -> Option<(f32, f32, f32, f32)> {
    let values = raw
        .split(|character: char| character.is_ascii_whitespace() || character == ',')
        .filter(|value| !value.is_empty())
        .filter_map(|value| value.parse::<f32>().ok())
        .collect::<Vec<_>>();
    (values.len() == 4).then(|| (values[0], values[1], values[2], values[3]))
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
