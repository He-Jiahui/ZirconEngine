use std::hint::black_box;
use std::time::Instant;

const DIAGNOSTIC_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_ji_editor648_preallocates_unsafe_guidance_items() {
    let source = include_str!("../../runtime_report_state.rs");
    let guidance = source
        .split("fn unsafe_action_guidance_items")
        .nth(1)
        .expect("unsafe action guidance remains present")
        .split("#[cfg(test)]")
        .next()
        .expect("unsafe action guidance remains bounded");

    assert!(guidance.contains("let guidance_capacity = runtime_report"));
    assert!(guidance.contains("Vec::with_capacity(guidance_capacity)"));
    assert!(!guidance.contains("let mut items = Vec::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ji_editor648_unsafe_guidance_capacity_benchmark() {
    for _ in 0..4 {
        black_box(measure_guidance(false));
        black_box(measure_guidance(true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_guidance(false));
            reserved_samples.push(measure_guidance(true));
        } else {
            reserved_samples.push(measure_guidance(true));
            unreserved_samples.push(measure_guidance(false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let reserved_p95 = percentile(&reserved_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(reserved_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR648_UNSAFE_GUIDANCE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} diagnostic_count={DIAGNOSTIC_COUNT} unreserved_ns={} reserved_ns={} unreserved_p95_ns={unreserved_p95} reserved_p95_ns={reserved_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&reserved_samples),
    );
    assert!(reserved_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_guidance(reserved: bool) -> u128 {
    let started = Instant::now();
    let mut items = if reserved {
        Vec::with_capacity(DIAGNOSTIC_COUNT + 1)
    } else {
        Vec::new()
    };
    for index in 0..DIAGNOSTIC_COUNT {
        items.push(black_box(index));
    }
    black_box(items);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
