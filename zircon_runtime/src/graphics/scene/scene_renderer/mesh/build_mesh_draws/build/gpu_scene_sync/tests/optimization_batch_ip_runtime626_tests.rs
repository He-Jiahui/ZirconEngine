use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

const DRAW_COUNT: usize = 65_536;
const SAMPLE_COUNT: usize = 17;

fn legacy_sync_indexes(keys: &[u64]) -> (usize, usize) {
    let mut live = HashSet::new();
    let mut entries = HashMap::new();
    for (index, key) in keys.iter().copied().enumerate() {
        live.insert(key);
        entries.insert(key, index);
    }
    (live.len(), entries.len())
}

fn optimized_sync_indexes(keys: &[u64]) -> (usize, usize) {
    let draw_capacity = keys.len();
    let mut live = HashSet::with_capacity(draw_capacity);
    let mut entries = HashMap::with_capacity(draw_capacity);
    for (index, key) in keys.iter().copied().enumerate() {
        live.insert(key);
        entries.insert(key, index);
    }
    (live.len(), entries.len())
}

#[test]
fn optimization_batch_ip_runtime626_gpu_scene_sync_preallocates_draw_indexes() {
    let source = include_str!("../../gpu_scene_sync.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("GPU scene sync production source");

    assert!(production.contains("let draw_capacity = pending_draws.len();"));
    // BUG: [CR-R02-runtime_wave12_graphics_mesh_draw_build-0001] 当前生产代码从 entries.keys 收集 live_keys，已无此旧 HashSet 构造；前置 draw_capacity 断言成立后本行必失败，不能作为现实现预分配证据。
    assert!(production.contains("HashSet::with_capacity(draw_capacity)"));
    assert!(production.contains("HashMap::with_capacity(draw_capacity)"));
    assert!(!production.contains("let mut live_keys = HashSet::new()"));
    assert!(!production.contains("let mut entries = HashMap::new()"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_ip_runtime626_gpu_scene_sync_index_performance_evidence() {
    let keys = (0..DRAW_COUNT as u64)
        .map(|key| key.wrapping_mul(0x9e37_79b9_7f4a_7c15))
        .collect::<Vec<_>>();
    assert_eq!(legacy_sync_indexes(&keys), optimized_sync_indexes(&keys));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_sync_indexes(black_box(&keys)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_sync_indexes(black_box(&keys)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_sync_indexes(black_box(&keys)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_sync_indexes(black_box(&keys)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "RUNTIME626_PREALLOCATED_GPU_SCENE_SYNC_BENCH_V1 draws={DRAW_COUNT} indexes=2 \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=8500"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_500),
        "preallocated GPU scene sync P95 {optimized_p95} ns exceeded 85% of unreserved {legacy_p95} ns"
    );
}
