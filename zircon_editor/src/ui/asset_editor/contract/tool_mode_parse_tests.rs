use std::hint::black_box;
use std::time::Instant;

use super::UiDesignerToolMode;

const PERF_MARKER: &str = "EDITOR82_DESIGNER_TOOL_MODE_BORROWED_PARSE_BENCH_V1";

#[test]
fn optimization_batch_20260826cs_editor_designer_tool_mode_parse_preserves_aliases() {
    for value in ["select", " SELECT "] {
        assert_eq!(
            UiDesignerToolMode::parse(value),
            Some(UiDesignerToolMode::Select)
        );
    }
    for value in ["resize_slot", "RESIZE-SLOT", " resize slot "] {
        assert_eq!(
            UiDesignerToolMode::parse(value),
            Some(UiDesignerToolMode::ResizeSlot)
        );
    }
    for value in ["preview_interact", "PREVIEW-INTERACT", " preview interact "] {
        assert_eq!(
            UiDesignerToolMode::parse(value),
            Some(UiDesignerToolMode::PreviewInteract)
        );
    }
    assert_eq!(UiDesignerToolMode::parse("preview-interact\u{00e9}"), None);
}

#[test]
fn optimization_batch_20260826cs_editor_designer_tool_mode_parse_source_contract() {
    let source = include_str!("../contract.rs");
    let parser = source
        .split("pub fn parse(value: &str)")
        .nth(1)
        .expect("designer tool parser")
        .split("#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]")
        .next()
        .expect("bounded designer tool parser");

    assert!(parser.contains("match value.len()"));
    assert!(parser.contains("eq_ignore_ascii_case"));
    assert!(!parser.contains("to_ascii_lowercase"));
    assert_eq!(
        PERF_MARKER,
        "EDITOR82_DESIGNER_TOOL_MODE_BORROWED_PARSE_BENCH_V1"
    );
}

#[test]
#[ignore = "release-only paired P95 performance evidence"]
fn optimization_batch_20260826cs_editor_designer_tool_mode_parse_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const PARSES_PER_SAMPLE: usize = 240_000;
    let aliases = [
        " SELECT ",
        "resize_slot",
        "RESIZE-SLOT",
        " resize slot ",
        "preview_interact",
        "PREVIEW-INTERACT",
        " preview interact ",
    ];

    black_box(measure_legacy(&aliases, PARSES_PER_SAMPLE / 10));
    black_box(measure_optimized(&aliases, PARSES_PER_SAMPLE / 10));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy(&aliases, PARSES_PER_SAMPLE));
            optimized_ns.push(measure_optimized(&aliases, PARSES_PER_SAMPLE));
        } else {
            optimized_ns.push(measure_optimized(&aliases, PARSES_PER_SAMPLE));
            legacy_ns.push(measure_legacy(&aliases, PARSES_PER_SAMPLE));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    let reduction = 100.0 * (legacy_p95_ns.saturating_sub(optimized_p95_ns)) as f64
        / legacy_p95_ns.max(1) as f64;

    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} parses_per_sample={PARSES_PER_SAMPLE} aliases=7 order=alternating_legacy_first_even legacy_normalized_string_allocations_per_sample={PARSES_PER_SAMPLE} optimized_normalized_string_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} p95_reduction_percent={reduction:.2}"
    );
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "borrowed designer tool parsing must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_parse(value: &str) -> Option<UiDesignerToolMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "select" => Some(UiDesignerToolMode::Select),
        "resize_slot" | "resize-slot" | "resize slot" => Some(UiDesignerToolMode::ResizeSlot),
        "preview_interact" | "preview-interact" | "preview interact" => {
            Some(UiDesignerToolMode::PreviewInteract)
        }
        _ => None,
    }
}

fn measure_legacy(aliases: &[&str], parses: usize) -> u128 {
    measure(aliases, parses, legacy_parse)
}

fn measure_optimized(aliases: &[&str], parses: usize) -> u128 {
    measure(aliases, parses, UiDesignerToolMode::parse)
}

fn measure(aliases: &[&str], parses: usize, parse: fn(&str) -> Option<UiDesignerToolMode>) -> u128 {
    let mut checksum = 0usize;
    let started = Instant::now();
    for index in 0..parses {
        let mode = parse(black_box(aliases[index % aliases.len()])).expect("benchmark alias");
        checksum = checksum.wrapping_add(mode as usize);
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
