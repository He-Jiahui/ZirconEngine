use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::parse_selected_folder;

const PERF_MARKER: &str = "EDITOR81_PICKER_BORROWED_UTF8_SELECTION_BENCH_V1";

#[test]
fn optimization_batch_20260826cr_editor_picker_selection_preserves_text_behavior() {
    assert_eq!(
        parse_selected_folder(b"  Builds/zircon/desktop_windows\r\n"),
        Some(PathBuf::from("Builds/zircon/desktop_windows"))
    );
    assert_eq!(parse_selected_folder(b" \r\n\t"), None);
    assert_eq!(
        parse_selected_folder(b"Builds/invalid-\xff\n"),
        Some(PathBuf::from("Builds/invalid-\u{fffd}"))
    );
}

#[test]
fn optimization_batch_20260826cr_editor_picker_selection_source_contract() {
    let source = include_str!("../selection.rs");

    assert!(source.contains("let selected = String::from_utf8_lossy(stdout);"));
    assert!(source.contains("let selected = selected.trim();"));
    assert!(!source.contains("String::from_utf8_lossy(stdout).trim().to_string()"));
    assert_eq!(
        PERF_MARKER,
        "EDITOR81_PICKER_BORROWED_UTF8_SELECTION_BENCH_V1"
    );
}

#[test]
#[ignore = "release-only paired P95 performance evidence"]
fn optimization_batch_20260826cr_editor_picker_selection_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const PARSES_PER_SAMPLE: usize = 90_000;
    let stdout = b"  C:/projects/zircon/Builds/desktop_windows/development/content/shaders/cache/output/artifacts  \r\n";

    black_box(measure_legacy(stdout, PARSES_PER_SAMPLE / 10));
    black_box(measure_optimized(stdout, PARSES_PER_SAMPLE / 10));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy(stdout, PARSES_PER_SAMPLE));
            optimized_ns.push(measure_optimized(stdout, PARSES_PER_SAMPLE));
        } else {
            optimized_ns.push(measure_optimized(stdout, PARSES_PER_SAMPLE));
            legacy_ns.push(measure_legacy(stdout, PARSES_PER_SAMPLE));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    let reduction = 100.0 * (legacy_p95_ns.saturating_sub(optimized_p95_ns)) as f64
        / legacy_p95_ns.max(1) as f64;

    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} parses_per_sample={PARSES_PER_SAMPLE} input_bytes={} order=alternating_legacy_first_even legacy_allocations_per_parse=2 optimized_allocations_per_parse=1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} p95_reduction_percent={reduction:.2}",
        stdout.len()
    );
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "borrowed picker UTF-8 parsing must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_parse_selected_folder(stdout: &[u8]) -> Option<PathBuf> {
    let selected = String::from_utf8_lossy(stdout).trim().to_string();
    (!selected.is_empty()).then(|| PathBuf::from(selected))
}

fn measure_legacy(stdout: &[u8], parses: usize) -> u128 {
    measure(stdout, parses, legacy_parse_selected_folder)
}

fn measure_optimized(stdout: &[u8], parses: usize) -> u128 {
    measure(stdout, parses, parse_selected_folder)
}

fn measure(stdout: &[u8], parses: usize, parse: fn(&[u8]) -> Option<PathBuf>) -> u128 {
    let mut checksum = 0usize;
    let started = Instant::now();
    for _ in 0..parses {
        let selected = parse(black_box(stdout)).expect("benchmark selection");
        checksum = checksum.wrapping_add(selected.as_os_str().len());
        black_box(selected);
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
