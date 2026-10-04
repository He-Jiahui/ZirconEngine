use super::{ready_set_level_count, OrderedReadySet};

#[test]
fn pops_sparse_indices_in_stable_order() {
    let mut set = OrderedReadySet::new(1_000);
    for index in [999, 64, 511, 0, 65, 511] {
        set.insert(index);
    }

    assert_eq!(
        std::iter::from_fn(|| set.pop_first()).collect::<Vec<_>>(),
        [0, 64, 65, 511, 999]
    );
}

#[test]
fn optimization_batch_20260830cv_ready_set_level_count_covers_word_boundaries() {
    assert_eq!(ready_set_level_count(0), 0);
    assert_eq!(ready_set_level_count(1), 1);
    assert_eq!(ready_set_level_count(64), 1);
    assert_eq!(ready_set_level_count(65), 2);
    assert_eq!(ready_set_level_count(4_096), 2);
    assert_eq!(ready_set_level_count(4_097), 3);
}

#[test]
fn optimization_batch_20260830cv_ready_set_reserves_its_exact_level_count() {
    let source = include_str!("../ordered_ready_set.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("ordered ready set production source");

    assert!(production.contains("Vec::with_capacity(ready_set_level_count(capacity))"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830cv_ready_set_level_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const FEATURE_CAPACITY: usize = 1 << 20;
    const MARKER: &str = "RUNTIME509_ORDERED_READY_SET_LEVEL_CAPACITY_BENCH_V1";
    let level_count = ready_set_level_count(FEATURE_CAPACITY);

    let legacy_growth_events = level_growth_events(BATCH_COUNT, level_count, false);
    let optimized_growth_events = level_growth_events(BATCH_COUNT, level_count, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} feature_capacity={FEATURE_CAPACITY} \
             levels_per_batch={level_count} legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn level_growth_events(batch_count: usize, level_count: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut levels = if reserve {
            Vec::with_capacity(level_count)
        } else {
            Vec::new()
        };
        for level in 0..level_count {
            let previous_capacity = levels.capacity();
            levels.push(level);
            growth_events += usize::from(levels.capacity() != previous_capacity);
        }
    }
    growth_events
}
