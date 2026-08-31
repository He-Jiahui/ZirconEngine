use std::hint::black_box;
use std::time::Instant;

use super::record_first_current_index;

const SAMPLE_PAIRS: usize = 21;
const SCANS_PER_SAMPLE: usize = 8_192;
const ROW_COUNT: usize = 256;

#[test]
fn optimization_batch_20260826ed_editor119_focus_index_preserves_first_collected_row() {
    let mut current_index = None;
    record_first_current_index(&mut current_index, 2, false, false);
    record_first_current_index(&mut current_index, 3, true, false);
    record_first_current_index(&mut current_index, 4, false, true);
    assert_eq!(current_index, Some(3));

    let mut selected_only = None;
    record_first_current_index(&mut selected_only, 0, false, true);
    assert_eq!(selected_only, Some(0));
}

#[test]
fn optimization_batch_20260826ed_editor119_focus_index_is_recorded_during_row_build() {
    let source = include_str!("../page_overflow.rs");
    let function_start = source
        .find("pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_keyboard_target_with_state")
        .unwrap();
    let function_end = source[function_start..]
        .find("#[cfg(test)]")
        .map(|offset| function_start + offset)
        .unwrap();
    let function_source = &source[function_start..function_end];
    assert_eq!(
        function_source
            .matches("record_first_current_index(")
            .count(),
        1
    );
    assert!(!function_source.contains("rows.iter().position"));
    assert!(!function_source.contains(".filter_map("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ed_editor119_page_overflow_focus_single_scan_bench() {
    let flags = fixture_flags();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&flags));
            optimized_samples.push(measure_optimized(&flags));
        } else {
            optimized_samples.push(measure_optimized(&flags));
            legacy_samples.push(measure_legacy(&flags));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR119_PAGE_OVERFLOW_FOCUS_SINGLE_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
scans_per_sample={SCANS_PER_SAMPLE} rows_per_scan={ROW_COUNT} legacy_passes=2 \
optimized_passes=1 legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single overflow focus scan P95 {optimized_p95_ns}ns must be at most 70% of build-plus-position P95 {legacy_p95_ns}ns"
    );
}

fn fixture_flags() -> Vec<(bool, bool)> {
    let mut flags = vec![(false, false); ROW_COUNT];
    flags[ROW_COUNT - 1] = (true, false);
    flags
}

fn measure_legacy(flags: &[(bool, bool)]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SCANS_PER_SAMPLE {
        for (focused, selected) in black_box(flags).iter().copied() {
            checksum = checksum.wrapping_add(focused as usize + selected as usize);
        }
        checksum ^= black_box(flags)
            .iter()
            .position(|(focused, selected)| *focused || *selected)
            .unwrap_or(0);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(flags: &[(bool, bool)]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SCANS_PER_SAMPLE {
        let mut current_index = None;
        for (index, (focused, selected)) in black_box(flags).iter().copied().enumerate() {
            checksum = checksum.wrapping_add(focused as usize + selected as usize);
            record_first_current_index(&mut current_index, index, focused, selected);
        }
        checksum ^= current_index.unwrap_or(0);
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
