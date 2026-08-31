use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const ROWS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_20260826gk_runtime231_target_mode_validation_and_dedup_preserve_results() {
    let mut target_modes = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
        RuntimeTargetMode::EditorHost,
    ];
    let mut diagnostics = Vec::new();

    validate_project_target_modes(
        format_args!("project target_modes"),
        &target_modes,
        &mut diagnostics,
    );
    deduplicate_target_modes(&mut target_modes);

    assert_eq!(diagnostics.len(), 3);
    assert!(diagnostics[0].contains("ClientRuntime"));
    assert!(diagnostics[1].contains("ClientRuntime"));
    assert!(diagnostics[2].contains("EditorHost"));
    assert_eq!(
        target_modes,
        [
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::EditorHost,
        ]
    );
}

#[test]
fn deferred_target_mode_diagnostic_context_preserves_contract() {
    let target_modes = [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ClientRuntime,
    ];
    let mut diagnostics = Vec::new();

    validate_project_target_modes(
        format_args!("project plugin feature rendering.shadow target_modes"),
        &target_modes,
        &mut diagnostics,
    );

    assert_eq!(
        diagnostics,
        ["project plugin feature rendering.shadow target_modes must not repeat target mode ClientRuntime"]
    );
}

#[test]
fn optimization_batch_20260826gk_runtime231_target_modes_use_prefix_and_bitset_scratch() {
    let source = include_str!("../target_modes.rs");

    assert!(source.contains("target_modes[..index].contains(&target_mode)"));
    assert!(source.contains("let mut seen_target_modes = 0_u8;"));
    assert!(source.contains("fn project_target_mode_bit("));
    assert!(!source.contains("let mut seen = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gk_runtime231_project_target_mode_scratch_allocation_bench() {
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
        "RUNTIME231_PROJECT_TARGET_MODE_SCRATCH_ALLOCATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
rows_per_sample={ROWS_PER_SAMPLE} modes_per_row=6 legacy_scratch_vectors_per_row=2 \
optimized_scratch_vectors_per_row=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(use_bitset: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for row in 0..ROWS_PER_SAMPLE {
        let modes = black_box([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
            RuntimeTargetMode::EditorHost,
        ]);
        let (duplicates, retained) = if use_bitset {
            optimized_row(&modes)
        } else {
            legacy_row(&modes)
        };
        checksum ^= black_box(duplicates ^ retained.len() ^ retained.capacity() ^ row);
        black_box(retained);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_row(modes: &[RuntimeTargetMode]) -> (usize, Vec<RuntimeTargetMode>) {
    let mut seen = Vec::new();
    let mut duplicates = 0;
    for mode in modes.iter().copied() {
        if seen.contains(&mode) {
            duplicates += 1;
        } else {
            seen.push(mode);
        }
    }
    black_box(&seen);

    let mut retained = modes.to_vec();
    let mut dedup_seen = Vec::new();
    retained.retain(|mode| {
        if dedup_seen.contains(mode) {
            false
        } else {
            dedup_seen.push(*mode);
            true
        }
    });
    black_box(&dedup_seen);
    (duplicates, retained)
}

fn optimized_row(modes: &[RuntimeTargetMode]) -> (usize, Vec<RuntimeTargetMode>) {
    let duplicates = modes
        .iter()
        .enumerate()
        .filter(|(index, mode)| modes[..*index].contains(mode))
        .count();
    let mut retained = modes.to_vec();
    let mut seen_target_modes = 0_u8;
    retained.retain(|mode| {
        let mode_bit = project_target_mode_bit(*mode);
        let retain = seen_target_modes & mode_bit == 0;
        seen_target_modes |= mode_bit;
        retain
    });
    (duplicates, retained)
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
