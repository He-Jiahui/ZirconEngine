use std::hint::black_box;
use std::time::Instant;

use super::{migrated_v1_link_id, validate_v1_link_count};

#[test]
fn optimization_batch_gq_runtime499_v1_link_ids_preserve_dense_checked_numbering() {
    assert_eq!(validate_v1_link_count(0), Ok(()));
    assert_eq!(validate_v1_link_count(u32::MAX as usize), Ok(()));
    assert_eq!(migrated_v1_link_id(0), 1);
    assert_eq!(migrated_v1_link_id(41), 42);
    assert_eq!(migrated_v1_link_id(u32::MAX as usize - 1), u32::MAX);

    #[cfg(target_pointer_width = "64")]
    assert_eq!(
        validate_v1_link_count(u32::MAX as usize + 1),
        Err(u32::MAX as usize + 1)
    );
}

#[test]
#[ignore = "release benchmark submitted to the validation coordinator"]
fn optimization_batch_gq_runtime499_v1_link_id_preflight_benchmark() {
    const MARKER: &str = "RUNTIME499_V1_LINK_ID_PREFLIGHT_BENCH_V1";
    const SAMPLES: usize = 31;
    const ITERATIONS: usize = 100_000;

    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for index in 0..ITERATIONS {
            checksum = checksum.wrapping_add(u64::from(migrated_v1_link_id(black_box(index))));
        }
        black_box(checksum);
        optimized_samples.push(started_at.elapsed().as_nanos() / ITERATIONS as u128);

        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for index in 0..ITERATIONS {
            let id = u32::try_from(black_box(index) + 1).expect("fixture link id");
            checksum = checksum.wrapping_add(u64::from(id));
        }
        black_box(checksum);
        legacy_samples.push(started_at.elapsed().as_nanos() / ITERATIONS as u128);
    }

    let optimized_p95_ns = p95(&mut optimized_samples);
    let legacy_p95_ns = p95(&mut legacy_samples);
    eprintln!(
        "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[samples.len().saturating_mul(95).div_ceil(100) - 1]
}
