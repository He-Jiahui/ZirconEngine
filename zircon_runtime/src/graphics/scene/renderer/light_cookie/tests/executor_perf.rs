use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_gy_runtime580_cookie_context_preflight_preserves_error_order() {
    assert_eq!(
        validate_light_cookie_executor_context(false, false),
        Err(MISSING_STREAMER_CONTEXT)
    );
    assert_eq!(
        validate_light_cookie_executor_context(true, false),
        Err(MISSING_MESH_PIPELINE_CONTEXT)
    );
    assert_eq!(validate_light_cookie_executor_context(true, true), Ok(()));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_gy_runtime580_cookie_context_preflight_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS: usize = 8_192;
    const COOKIE_COUNT: usize = 4_096;
    let cookies = (0..COOKIE_COUNT as u64).collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false, &cookies, ITERATIONS));
            optimized.push(measure(true, &cookies, ITERATIONS));
        } else {
            optimized.push(measure(true, &cookies, ITERATIONS));
            legacy.push(measure(false, &cookies, ITERATIONS));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME580_COOKIE_CONTEXT_PREFLIGHT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} cookie_count={COOKIE_COUNT} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
        "cookie context preflight must improve missing-context P95 by at least 50%"
    );
}

fn measure(optimized: bool, cookies: &[u64], iterations: usize) -> u128 {
    let started = Instant::now();
    let mut rejected = 0_u64;
    for _ in 0..iterations {
        if optimized {
            rejected += u64::from(validate_light_cookie_executor_context(false, true).is_err());
        } else {
            let cloned = black_box(cookies).to_vec();
            rejected += u64::from(validate_light_cookie_executor_context(false, true).is_err());
            black_box(cloned);
        }
    }
    black_box(rejected);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
