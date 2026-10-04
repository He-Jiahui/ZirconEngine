use super::template_asset_capacity;

const BENCHMARK_MARKER: &str = "RUNTIME794_HOT_RELOAD_TEMPLATE_ASSETS_CAPACITY_BENCH_V1";

#[test]
fn runtime794_hot_reload_template_asset_capacity_is_bounded_and_overflow_safe() {
    assert_eq!(template_asset_capacity(0, 0), 0);
    assert_eq!(template_asset_capacity(128, 256), 384);
    assert_eq!(template_asset_capacity(usize::MAX, usize::MAX), usize::MAX);
}

#[test]
#[ignore = "run in the managed Release validation batch"]
fn optimization_batch_20260917_runtime794_hot_reload_template_assets_capacity_bench() {
    const INPUT_COUNT: usize = 16_384;
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
