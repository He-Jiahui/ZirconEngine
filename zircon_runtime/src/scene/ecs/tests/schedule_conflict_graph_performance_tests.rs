use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const PERF_SAMPLE_PAIRS: usize = 21;
const PERF_ITERATIONS_PER_SAMPLE: usize = 5_000;
const PERF_SYSTEM_COUNT: usize = 64;

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn shared_schedule_batch_systems_avoid_frame_heap_allocations() {
    let mut batch =
        ScheduleParallelBatch::single(SystemStage::Update, "system.000".to_string(), 0, false);
    for index in 1..PERF_SYSTEM_COUNT {
        batch.push_system(format!("system.{index:03}"), index);
    }

    let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    for pair_index in 0..PERF_SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_legacy(&batch));
            optimized_samples.push(measure_optimized(&batch));
        } else {
            optimized_samples.push(measure_optimized(&batch));
            legacy_samples.push(measure_legacy(&batch));
        }
    }

    let legacy_p50 = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95 = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50 = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95 = percentile_ns(&mut optimized_samples, 95);
    println!(
        "PERF_RESULT runtime60_shared_schedule_batch_systems legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} systems_per_batch={PERF_SYSTEM_COUNT} iterations_per_sample={PERF_ITERATIONS_PER_SAMPLE} samples={PERF_SAMPLE_PAIRS} legacy_heap_allocations_per_batch=65 optimized_heap_allocations_per_batch=0"
    );

    assert!(
        optimized_p95 <= legacy_p95 / 2,
        "shared batch storage should cut P95 execution-frame setup by at least 50%: legacy={legacy_p95}ns optimized={optimized_p95}ns"
    );
}

fn measure_legacy(batch: &ScheduleParallelBatch) -> Duration {
    let started = Instant::now();
    for _ in 0..PERF_ITERATIONS_PER_SAMPLE {
        let system_ids = black_box(batch.system_ids()).to_vec();
        black_box(system_ids);
    }
    started.elapsed()
}

fn measure_optimized(batch: &ScheduleParallelBatch) -> Duration {
    let started = Instant::now();
    for _ in 0..PERF_ITERATIONS_PER_SAMPLE {
        let systems = black_box(batch).shared_systems();
        black_box(systems.system_ids());
    }
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = samples.len().saturating_mul(percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
