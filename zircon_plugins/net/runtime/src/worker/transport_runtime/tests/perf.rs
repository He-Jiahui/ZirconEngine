use std::hint::black_box;
use std::time::Instant;

use super::udp_receive_buffer;

const BUFFER_BYTES: usize = u16::MAX as usize;
const POLLS_PER_SAMPLE: usize = 4_096;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED_IMPROVEMENT_PERCENT: u128 = 90;

#[test]
fn optimization_batch_20260830db_udp_receive_buffer_covers_full_datagram() {
    let buffer = udp_receive_buffer();

    assert_eq!(buffer.len(), BUFFER_BYTES);
    assert!(buffer.iter().all(|byte| *byte == 0));
}

#[test]
#[ignore = "release performance contract"]
fn optimization_batch_20260830db_udp_receive_buffer_is_reused_across_empty_polls() {
    let (legacy_p95, optimized_p95) = paired_p95(
        || {
            for _ in 0..POLLS_PER_SAMPLE {
                black_box(vec![0u8; BUFFER_BYTES]);
            }
        },
        || {
            let mut buffer = udp_receive_buffer();
            for _ in 0..POLLS_PER_SAMPLE {
                black_box(buffer.as_mut_slice());
            }
        },
    );
    let improvement =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);

    println!(
        "PERF_RESULT task=runtime173_udp_receive_buffer_reuse polls={POLLS_PER_SAMPLE} buffer_bytes={BUFFER_BYTES} sample_pairs={SAMPLE_PAIRS} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}"
    );
    assert!(
        improvement >= REQUIRED_IMPROVEMENT_PERCENT,
        "UDP receive-buffer reuse must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn paired_p95(mut legacy: impl FnMut(), mut optimized: impl FnMut()) -> (u128, u128) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    (
        legacy_samples[SAMPLE_PAIRS * 95 / 100],
        optimized_samples[SAMPLE_PAIRS * 95 / 100],
    )
}

fn measure(operation: &mut impl FnMut()) -> u128 {
    let started = Instant::now();
    operation();
    started.elapsed().as_nanos()
}
