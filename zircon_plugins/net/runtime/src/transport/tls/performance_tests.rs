use std::hint::black_box;
use std::time::Instant;

use super::hex_encode;

const DIGEST_BYTES: usize = 32;
const ITERATIONS: usize = 16_384;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED_IMPROVEMENT_PERCENT: u128 = 80;

#[test]
fn optimization_batch_20260830db_tls_hex_preserves_lowercase_and_leading_zeroes() {
    assert_eq!(
        hex_encode(&[0x00, 0x01, 0x0f, 0x10, 0xab, 0xff]),
        "00010f10abff"
    );
}

#[test]
#[ignore = "release performance contract"]
fn optimization_batch_20260830db_tls_hex_avoids_per_byte_format_allocations() {
    let digest = (0..DIGEST_BYTES)
        .map(|index| (index * 37 + 11) as u8)
        .collect::<Vec<_>>();
    let (legacy_p95, optimized_p95) = paired_p95(
        || {
            for _ in 0..ITERATIONS {
                black_box(legacy_hex_encode(black_box(&digest)));
            }
        },
        || {
            for _ in 0..ITERATIONS {
                black_box(hex_encode(black_box(&digest)));
            }
        },
    );
    let improvement =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);

    println!(
        "PERF_RESULT task=runtime173_tls_hex_encoding digest_bytes={DIGEST_BYTES} iterations={ITERATIONS} sample_pairs={SAMPLE_PAIRS} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}"
    );
    assert!(
        improvement >= REQUIRED_IMPROVEMENT_PERCENT,
        "TLS hex encoding must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn legacy_hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
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
