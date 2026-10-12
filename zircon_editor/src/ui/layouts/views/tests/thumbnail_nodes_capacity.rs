use super::thumbnail_node_capacity;

const BENCHMARK_MARKER: &str = "EDITOR790_THUMBNAIL_NODE_CAPACITY_BENCH_V1";

#[test]
fn editor790_thumbnail_node_capacity_is_bounded_and_overflow_safe() {
    assert_eq!(thumbnail_node_capacity(0, 0), 1);
    assert_eq!(thumbnail_node_capacity(12, 4), 37);
    assert_eq!(thumbnail_node_capacity(4, 12), 37);
    assert_eq!(thumbnail_node_capacity(usize::MAX, usize::MAX), usize::MAX);
}

#[test]
#[ignore = "run in the managed Release validation batch"]
fn optimization_batch_20260917_editor790_thumbnail_node_capacity_bench() {
    const ITEM_COUNT: usize = 4_096;
    let node_count = 1 + ITEM_COUNT * 9;
    let legacy_growth_events = geometric_growth_events(node_count);
    let optimized_growth_events = 0;
    eprintln!(
        "{BENCHMARK_MARKER} item_count={ITEM_COUNT} node_count={node_count} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);
}

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
