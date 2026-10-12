use std::hint::black_box;
use std::time::Instant;

use super::is_major_grid_line;

#[test]
fn optimization_batch_hd_editor585_grid_line_suffix_classifies_canonical_markers() {
    for id in [
        "WorkbenchViewportGridH2",
        "WorkbenchViewportGridH4",
        "WorkbenchViewportGridV2",
        "WorkbenchViewportGridV5",
    ] {
        assert!(is_major_grid_line(id), "{id} should be a major grid line");
    }
    assert!(!is_major_grid_line("WorkbenchViewportGridH3"));
    assert!(!is_major_grid_line("workbenchviewportgridh2"));
    assert!(!is_major_grid_line("WorkbenchViewportGridH2Decoration"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hd_editor585_grid_marker_suffix_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS: usize = 32_768;
    let control_id = "WorkbenchViewportGridMinorLine".repeat(16);
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false, &control_id, ITERATIONS));
            optimized.push(measure(true, &control_id, ITERATIONS));
        } else {
            optimized.push(measure(true, &control_id, ITERATIONS));
            legacy.push(measure(false, &control_id, ITERATIONS));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "EDITOR585_GRID_MARKER_SUFFIX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} control_id_bytes={} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        control_id.len(),
        csv(&legacy),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(25),
        "canonical grid marker suffix classification must improve P95 by at least 75%"
    );
}

fn measure(optimized: bool, control_id: &str, iterations: usize) -> u128 {
    let started = Instant::now();
    let mut classified = false;
    for _ in 0..iterations {
        classified ^= if optimized {
            is_major_grid_line(black_box(control_id))
        } else {
            legacy_is_major_grid_line(black_box(control_id))
        };
    }
    black_box(classified);
    started.elapsed().as_nanos().max(1)
}

fn legacy_is_major_grid_line(control_id: &str) -> bool {
    control_id.contains("H2")
        || control_id.contains("H4")
        || control_id.contains("V2")
        || control_id.contains("V5")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
