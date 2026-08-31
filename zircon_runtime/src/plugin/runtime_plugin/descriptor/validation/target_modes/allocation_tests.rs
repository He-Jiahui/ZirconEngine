use std::hint::black_box;
use std::time::Instant;

use super::validate_runtime_plugin_descriptor_target_modes;
use crate::core::framework::platform::RuntimeTargetMode;

const SAMPLE_PAIRS: usize = 21;
const VALIDATIONS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_20260826gf_runtime227_preserves_missing_and_duplicate_diagnostics() {
    let mut diagnostics = Vec::new();
    validate_runtime_plugin_descriptor_target_modes(&[], &mut diagnostics);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].contains("must declare at least one target mode"));

    diagnostics.clear();
    validate_runtime_plugin_descriptor_target_modes(
        &[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::ClientRuntime,
        ],
        &mut diagnostics,
    );
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.contains("must be unique")));
}

#[test]
fn optimization_batch_20260826gf_runtime227_checks_the_input_prefix_without_a_seen_vec() {
    let source = include_str!("../target_modes.rs");

    assert!(source.contains("for (index, target_mode) in target_modes.iter().copied().enumerate()"));
    assert!(source.contains("target_modes[..index].contains(&target_mode)"));
    assert!(!source.contains("let mut seen = Vec::new();"));
    assert!(!source.contains("seen.push(target_mode);"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gf_runtime227_descriptor_target_mode_allocation_bench() {
    let modes = [0_u8, 1, 2, 0];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&modes, false));
            optimized_samples.push(measure(&modes, true));
        } else {
            optimized_samples.push(measure(&modes, true));
            legacy_samples.push(measure(&modes, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME227_DESCRIPTOR_TARGET_MODE_ALLOCATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
validations_per_sample={VALIDATIONS_PER_SAMPLE} target_modes_per_validation={} \
legacy_temporary_vectors_per_validation=1 optimized_temporary_vectors_per_validation=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        modes.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(modes: &[u8], use_input_prefix: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for validation in 0..VALIDATIONS_PER_SAMPLE {
        let modes = black_box(modes);
        let mut duplicate_count = 0usize;
        if use_input_prefix {
            for (index, mode) in modes.iter().copied().enumerate() {
                duplicate_count += usize::from(modes[..index].contains(&mode));
            }
        } else {
            let mut seen = Vec::new();
            for mode in modes.iter().copied() {
                if seen.contains(&mode) {
                    duplicate_count += 1;
                } else {
                    seen.push(mode);
                }
            }
            black_box(&seen);
        }
        checksum ^= black_box(duplicate_count ^ validation);
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
