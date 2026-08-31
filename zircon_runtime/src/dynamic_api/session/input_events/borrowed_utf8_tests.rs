use std::hint::black_box;
use std::time::Instant;

use super::*;

const PAYLOAD_BYTES: usize = 32 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 512;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hk_runtime257_preserves_ime_utf8_validation() {
    assert_eq!(
        owned_utf8_payload("Zircon \u{8f93}\u{5165}".as_bytes()).as_deref(),
        Some("Zircon \u{8f93}\u{5165}")
    );
    assert!(owned_utf8_payload(b"invalid\xffpayload").is_none());
}

#[test]
fn optimization_batch_20260826hk_runtime257_validates_ime_utf8_before_allocation() {
    let source = include_str!("../input_events.rs");
    let start = source
        .find("fn owned_utf8_payload(")
        .expect("owned_utf8_payload function");
    let end = source[start..]
        .find("\npub(in crate::dynamic_api::session) fn ime_surrounding_text")
        .map(|offset| start + offset)
        .expect("ime_surrounding_text boundary");
    let body = &source[start..end];

    assert!(body.contains("std::str::from_utf8(payload)"));
    assert!(body.contains(".map(str::to_owned)"));
    assert!(!source.contains("String::from_utf8(payload.to_vec())"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hk_runtime257_borrowed_ime_utf8_release_benchmark() {
    let mut payload = vec![b'a'; PAYLOAD_BYTES];
    payload[PAYLOAD_BYTES - 1] = 0xff;

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_owned_utf8_payload(black_box(&payload)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(owned_utf8_payload(black_box(&payload)));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME257_BORROWED_IME_UTF8_VALIDATION_BENCH_V1 \
         payload_bytes={PAYLOAD_BYTES} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_owned_utf8_payload(payload: &[u8]) -> Option<String> {
    String::from_utf8(payload.to_vec()).ok()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
