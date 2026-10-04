use std::collections::VecDeque;
use std::hint::black_box;
use std::time::Instant;

use super::process_lossless_queue;

#[test]
fn optimization_batch_di_direct_lifecycle_queue_matches_success_order() {
    let mut pending = VecDeque::new();
    let mut processed = Vec::new();

    process_lossless_queue(&mut pending, vec![1_u32, 2, 3, 4], |value| {
        processed.push(*value);
        Ok::<_, &'static str>(())
    })
    .expect("successful delivery batch");

    assert_eq!(processed, [1, 2, 3, 4]);
    assert!(pending.is_empty());
}

#[test]
fn optimization_batch_di_direct_lifecycle_queue_retains_failure_and_remainder() {
    let mut pending = VecDeque::new();
    let mut processed = Vec::new();

    let error = process_lossless_queue(&mut pending, vec![1_u32, 2, 3, 4], |value| {
        processed.push(*value);
        (*value != 3).then_some(()).ok_or("blocked")
    })
    .expect_err("the third delivery should be retained");

    assert_eq!(error, "blocked");
    assert_eq!(processed, [1, 2, 3]);
    assert_eq!(pending, VecDeque::from([3, 4]));

    process_lossless_queue(&mut pending, vec![5], |value| {
        processed.push(*value);
        Ok::<_, &'static str>(())
    })
    .expect("retained deliveries should retry before fresh work");
    assert_eq!(processed, [1, 2, 3, 3, 4, 5]);
    assert!(pending.is_empty());
}

#[test]
fn optimization_batch_di_preexisting_lifecycle_queue_keeps_priority() {
    let mut pending = VecDeque::from([7_u32, 8]);
    let mut processed = Vec::new();

    process_lossless_queue(&mut pending, vec![9, 10], |value| {
        processed.push(*value);
        Ok::<_, &'static str>(())
    })
    .expect("queued and fresh deliveries should drain");

    assert_eq!(processed, [7, 8, 9, 10]);
    assert!(pending.is_empty());
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_di_direct_lifecycle_queue_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const DELIVERIES_PER_SAMPLE: usize = 65_536;

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_queue_transfer(DELIVERIES_PER_SAMPLE, true));
            optimized_samples.push(measure_queue_transfer(DELIVERIES_PER_SAMPLE, false));
        } else {
            optimized_samples.push(measure_queue_transfer(DELIVERIES_PER_SAMPLE, false));
            legacy_samples.push(measure_queue_transfer(DELIVERIES_PER_SAMPLE, true));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "EDITOR345_DIRECT_LIFECYCLE_DELIVERY_QUEUE_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "direct lifecycle delivery p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn measure_queue_transfer(deliveries: usize, legacy: bool) -> u128 {
    let fresh = (0..deliveries as u64).collect::<Vec<_>>();
    let mut pending = VecDeque::new();
    let started_at = Instant::now();
    let mut checksum = 0_u64;
    if legacy {
        pending.extend(black_box(fresh));
        while let Some(value) = pending.pop_front() {
            checksum = checksum.wrapping_add(black_box(value));
        }
    } else {
        process_lossless_queue(&mut pending, black_box(fresh), |value| {
            checksum = checksum.wrapping_add(black_box(*value));
            Ok::<_, ()>(())
        })
        .expect("benchmark delivery processing succeeds");
    }
    black_box((checksum, pending));
    started_at.elapsed().as_nanos()
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let index = samples
        .len()
        .saturating_mul(95)
        .div_ceil(100)
        .saturating_sub(1);
    samples[index]
}
