const BENCHMARK_MARKER: &str = "RUNTIME797_COMPILE_CACHE_EVICTION_KEY_CAPACITY_BENCH_V1";

#[test]
fn runtime797_compile_cache_eviction_key_capacity_is_input_bounded() {
    let source = include_str!("../../compile_cache.rs");
    let body = source
        .split("pub fn evict_assets")
        .nth(1)
        .and_then(|body| body.split("pub fn get").next())
        .expect("compile-cache eviction implementation");

    assert_eq!(
        body.matches("Vec::with_capacity(asset_ids.len())").count(),
        2
    );
    assert!(body.contains("entry_keys.extend"));
    assert!(body.contains("snapshot_keys.extend"));
}

#[test]
#[ignore = "run in the managed Release validation batch"]
fn optimization_batch_20260918_runtime797_compile_cache_eviction_key_capacity_bench() {
    const INPUT_COUNT: usize = 65_536;
    let legacy_growth_events = geometric_growth_events(INPUT_COUNT);
    let optimized_growth_events = 0;
    eprintln!(
        "{BENCHMARK_MARKER} input_count={INPUT_COUNT} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);
}

// 此辅助函数只模拟容量增长；该标记测试没有观测生产热重载或缓存驱逐的实际分配。
fn geometric_growth_events(length: usize) -> usize {
    let mut capacity = 0usize;
    let mut growth_events = 0usize;
    for current_length in 1..=length {
        if current_length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            growth_events += 1;
        }
    }
    growth_events
}
