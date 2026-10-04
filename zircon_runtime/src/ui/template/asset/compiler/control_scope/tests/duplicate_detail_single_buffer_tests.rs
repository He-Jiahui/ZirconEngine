use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::duplicate_control_id_detail;

const SAMPLE_PAIRS: usize = 101;
const DETAILS_PER_SAMPLE: usize = 4_096;
const DUPLICATE_IDS_PER_DETAIL: usize = 64;

#[test]
fn runtime873_duplicate_control_detail_single_buffer_preserves_exact_text() {
    for duplicates in [
        BTreeMap::new(),
        BTreeMap::from([("Only".to_string(), 1)]),
        BTreeMap::from([
            ("Zulu".to_string(), 3),
            ("Alpha".to_string(), 1),
            ("中控".to_string(), 2),
        ]),
        BTreeMap::from([("".to_string(), 4), ("Visible".to_string(), 1)]),
    ] {
        assert_eq!(
            duplicate_control_id_detail(&duplicates),
            legacy_duplicate_control_id_detail(&duplicates)
        );
    }
}

#[test]
fn runtime873_duplicate_control_detail_single_buffer_uses_exact_capacity() {
    let duplicates = BTreeMap::from([
        ("a".to_string(), 1),
        ("a-substantially-longer-control-id".to_string(), 2),
        ("中控".to_string(), 3),
    ]);
    let detail = duplicate_control_id_detail(&duplicates);

    assert_eq!(detail.capacity(), detail.len());
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn runtime873_duplicate_control_detail_single_buffer_release_percentiles() {
    let duplicates = (0..DUPLICATE_IDS_PER_DETAIL)
        .map(|index| (format!("Control{index:02}_{}", "x".repeat(24)), index + 1))
        .collect::<BTreeMap<_, _>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(|| {
                render_batch(&duplicates, legacy_duplicate_control_id_detail)
            }));
            optimized_samples.push(measure(|| {
                render_batch(&duplicates, duplicate_control_id_detail)
            }));
        } else {
            optimized_samples.push(measure(|| {
                render_batch(&duplicates, duplicate_control_id_detail)
            }));
            legacy_samples.push(measure(|| {
                render_batch(&duplicates, legacy_duplicate_control_id_detail)
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples.clone(), 95);
    let legacy_p99_ns = percentile_ns(&mut legacy_samples, 99);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples.clone(), 95);
    let optimized_p99_ns = percentile_ns(&mut optimized_samples, 99);
    println!(
        "RUNTIME873_DUPLICATE_CONTROL_DETAIL_SINGLE_BUFFER_BENCH_V1 \
legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} \
optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(
        optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100),
        "single-buffer p95 regressed beyond 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_duplicate_control_id_detail(duplicates: &BTreeMap<String, usize>) -> String {
    format!(
        "compiled template contains duplicate control ids: {}",
        duplicates.keys().cloned().collect::<Vec<_>>().join(", ")
    )
}

fn render_batch(
    duplicates: &BTreeMap<String, usize>,
    render: fn(&BTreeMap<String, usize>) -> String,
) -> usize {
    (0..DETAILS_PER_SAMPLE)
        .map(|_| black_box(render(black_box(duplicates))).len())
        .sum()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
