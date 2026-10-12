use super::base_node_id;
use std::hint::black_box;
use std::time::Instant;

#[test]
fn single_buffer_normalization_matches_the_legacy_identifier_contract() {
    for (label, expected) in [
        ("", "node"),
        ("---", "node"),
        (" Button ", "button"),
        ("Scene  View", "scene__view"),
        ("HTTP2-Panel", "http2_panel"),
        ("_already__split_", "already__split"),
        ("Cafe\u{301}", "cafe"),
        ("\u{754c}Button\u{754c}", "button"),
    ] {
        assert_eq!(base_node_id(label), expected, "{label:?}");
    }
}

#[test]
#[ignore = "release-only palette base-node-id benchmark"]
fn palette_base_node_id_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const IDS_PER_SAMPLE: usize = 100_000;
    let label = "  Primary Scene View / Inspector Row  ";

    fn legacy(label: &str) -> String {
        let normalized = label
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_ascii_lowercase();
        if normalized.is_empty() {
            "node".to_string()
        } else {
            normalized
        }
    }

    fn measure(label: &str, normalize: fn(&str) -> String) -> u128 {
        let started = Instant::now();
        for _ in 0..IDS_PER_SAMPLE {
            black_box(normalize(black_box(label)));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(label, legacy));
            optimized_samples.push(measure(label, base_node_id));
        } else {
            optimized_samples.push(measure(label, base_node_id));
            legacy_samples.push(measure(label, legacy));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "RUNTIME75_PALETTE_BASE_NODE_ID_BENCH_V1 sample_pairs={SAMPLE_PAIRS} ids_per_sample={IDS_PER_SAMPLE} legacy_allocations_per_id=2 optimized_allocations_per_id=1 legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={legacy_samples:?} optimized_raw_ns={optimized_samples:?}"
    );

    assert_eq!(base_node_id(label), legacy(label));
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
        "single-buffer base node IDs must reduce P95 by at least 20%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
