use std::hint::black_box;
use std::time::Instant;

use serde::Serialize;

use super::*;

#[derive(Serialize)]
struct ContractFixture<'a> {
    name: &'a str,
    payload: &'a str,
    enabled: bool,
}

#[test]
fn optimization_batch_ed_contract_serialization_appends_without_intermediate_string() {
    let source_code = include_str!("../fingerprint.rs");
    let production = source_code
        .split("#[cfg(test)]")
        .next()
        .expect("fingerprint production implementation");

    assert!(production.contains("let mut serializer_buffer = toml::ser::Buffer::new()"));
    assert!(production.contains("append_serializable_for_fingerprint("));
    assert!(production.contains("serializer_buffer.clear()"));
    assert!(production.contains("write!(source, \"{serializer_buffer}\")"));
    assert!(!production.contains("source.push_str(&serialize_for_fingerprint"));

    let fixture = ContractFixture {
        name: "Button",
        payload: "state = enabled",
        enabled: true,
    };
    let legacy = serialize_for_fingerprint(&fixture).expect("legacy fixture serialization");
    let mut appended = String::from("prefix\n");
    let mut serializer_buffer = toml::ser::Buffer::new();
    append_serializable_for_fingerprint(&mut appended, &mut serializer_buffer, &fixture)
        .expect("direct fixture serialization");

    assert_eq!(&appended["prefix\n".len()..], legacy);
}

#[test]
#[ignore = "release-only direct contract fingerprint append benchmark"]
fn optimization_batch_ed_direct_contract_fingerprint_append_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const CONTRACTS_PER_SAMPLE: usize = 64;
    const PAYLOAD_BYTES: usize = 4_096;

    fn measure_legacy(fixture: &ContractFixture<'_>) -> u128 {
        let mut source = String::with_capacity(CONTRACTS_PER_SAMPLE * (PAYLOAD_BYTES + 128));
        let started = Instant::now();
        for _ in 0..CONTRACTS_PER_SAMPLE {
            let serialized = serialize_for_fingerprint(black_box(fixture))
                .expect("legacy contract serialization");
            source.push_str(&serialized);
            source.push('\n');
        }
        black_box(source);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(fixture: &ContractFixture<'_>) -> u128 {
        let mut source = String::with_capacity(CONTRACTS_PER_SAMPLE * (PAYLOAD_BYTES + 128));
        let mut serializer_buffer = toml::ser::Buffer::new();
        let started = Instant::now();
        for _ in 0..CONTRACTS_PER_SAMPLE {
            append_serializable_for_fingerprint(
                &mut source,
                &mut serializer_buffer,
                black_box(fixture),
            )
            .expect("direct contract serialization");
            source.push('\n');
        }
        black_box(source);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let payload = "x".repeat(PAYLOAD_BYTES);
    let fixture = ContractFixture {
        name: "LargeWidgetContract",
        payload: &payload,
        enabled: true,
    };

    for _ in 0..4 {
        black_box(measure_legacy(&fixture));
        black_box(measure_optimized(&fixture));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&fixture));
            optimized_samples.push(measure_optimized(&fixture));
        } else {
            optimized_samples.push(measure_optimized(&fixture));
            legacy_samples.push(measure_legacy(&fixture));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "RUNTIME438_DIRECT_CONTRACT_FINGERPRINT_APPEND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
contracts_per_sample={CONTRACTS_PER_SAMPLE} payload_bytes={PAYLOAD_BYTES} \
pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 \
legacy_intermediate_output_strings_per_sample={CONTRACTS_PER_SAMPLE} \
optimized_intermediate_output_strings_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
            "direct contract serialization must reduce P95 by at least 20%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
}
