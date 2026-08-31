use std::hint::black_box;
use std::time::Instant;

const SLOT_COUNT: usize = 256;
const CHECKS_PER_SAMPLE: usize = 10_000;
const SAMPLE_PAIRS: usize = 31;

fn legacy_exact_unique_slots_cover_all(slots: &[usize], expected_count: usize) -> bool {
    if slots.len() != expected_count {
        return false;
    }
    let mut seen = vec![false; expected_count];
    for &slot in slots {
        let Some(seen_slot) = seen.get_mut(slot) else {
            return false;
        };
        if *seen_slot {
            return false;
        }
        *seen_slot = true;
    }
    seen.into_iter().all(|present| present)
}

fn optimized_exact_unique_slots_cover_all(slots: &[usize], expected_count: usize) -> bool {
    if slots.len() != expected_count {
        return false;
    }
    let mut seen = vec![false; expected_count];
    for &slot in slots {
        let Some(seen_slot) = seen.get_mut(slot) else {
            return false;
        };
        if *seen_slot {
            return false;
        }
        *seen_slot = true;
    }
    true
}

#[inline(never)]
fn legacy_finalize_coverage(seen: &[bool]) -> bool {
    black_box(seen).iter().copied().all(black_box)
}

#[inline(never)]
fn optimized_finalize_coverage(seen: &[bool]) -> bool {
    black_box(seen);
    true
}

fn measure(seen: &[bool], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0_usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        let complete = if optimized {
            optimized_finalize_coverage(seen)
        } else {
            legacy_finalize_coverage(seen)
        };
        evidence += black_box(complete) as usize;
    }
    black_box(evidence);
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

#[test]
fn optimization_batch_20260829bz_runtime353_exact_unique_slots_preserve_results() {
    for slots in [
        vec![0, 1, 2, 3],
        vec![3, 2, 1, 0],
        vec![0, 1, 1, 3],
        vec![0, 1, 2],
        vec![0, 1, 2, 4],
    ] {
        assert_eq!(
            optimized_exact_unique_slots_cover_all(&slots, 4),
            legacy_exact_unique_slots_cover_all(&slots, 4)
        );
    }
}

#[test]
fn optimization_batch_20260829bz_runtime353_production_elides_coverage_rescan() {
    let source = include_str!("../compressed.rs").replace("\r\n", "\n");
    let function = source
        .split_once("fn compressed_subresource_reason")
        .expect("compressed subresource validator")
        .1
        .split_once("\n#[derive(Clone, Copy)]")
        .expect("layout boundary")
        .0;
    assert!(!function.contains("seen.into_iter().all"));
    assert!(function.trim_end().ends_with("None\n}"));
}

#[test]
#[ignore = "managed performance gate"]
fn optimization_batch_20260829bz_runtime353_coverage_finalization_benchmark() {
    let seen = vec![true; SLOT_COUNT];
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&seen, false));
            candidate.push(measure(&seen, true));
        } else {
            candidate.push(measure(&seen, true));
            baseline.push(measure(&seen, false));
        }
    }
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME353_COMPRESSED_COVERAGE_RESCAN_BENCH_V1 baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_samples_ns={} candidate_samples_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns * 100 <= baseline_p95_ns * 70);
}
