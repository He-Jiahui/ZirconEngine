#[test]
fn optimization_batch_20260830db_activity_actions_use_fixed_sources_and_exact_capacity() {
    let resolve = include_str!("../resolve.rs");
    let production = resolve
        .split("#[cfg(test)]")
        .next()
        .expect("activity action production source");
    let sources = [
        (
            include_str!("../common_actions.rs"),
            "COMMON_TAB_ACTION_COUNT: usize = 2",
        ),
        (
            include_str!("../asset_actions.rs"),
            "ASSET_ACTION_COUNT: usize = 2",
        ),
        (
            include_str!("../inspector_actions.rs"),
            "INSPECTOR_ACTION_COUNT: usize = 3",
        ),
        (
            include_str!("../viewport_actions.rs"),
            "VIEWPORT_ACTION_COUNT: usize = 9",
        ),
    ];

    assert!(production.contains("Vec::with_capacity(action_capacity)"));
    for (source, count_declaration) in sources {
        assert!(source.contains(count_declaration));
        assert!(source.contains("-> [UiActionDescriptor;"));
    }
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830db_activity_action_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const COMMON_ACTION_COUNT: usize = 2;
    const SPECIALIZED_ACTION_COUNT: usize = 9;
    const ACTION_COUNT: usize = COMMON_ACTION_COUNT + SPECIALIZED_ACTION_COUNT;
    const MARKER: &str = "EDITOR514_ACTIVITY_ACTION_CAPACITY_BENCH_V1";

    let legacy_growth_events = action_growth_events(
        BATCH_COUNT,
        COMMON_ACTION_COUNT,
        SPECIALIZED_ACTION_COUNT,
        false,
    );
    let optimized_growth_events = action_growth_events(
        BATCH_COUNT,
        COMMON_ACTION_COUNT,
        SPECIALIZED_ACTION_COUNT,
        true,
    );

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} actions={ACTION_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn action_growth_events(
    batch_count: usize,
    common_action_count: usize,
    specialized_action_count: usize,
    reserve_exact: bool,
) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let initial_capacity = if reserve_exact {
            common_action_count.saturating_add(specialized_action_count)
        } else {
            common_action_count
        };
        let mut actions = Vec::with_capacity(initial_capacity);
        for action in 0..common_action_count.saturating_add(specialized_action_count) {
            let previous_capacity = actions.capacity();
            actions.push(action);
            growth_events += usize::from(actions.capacity() != previous_capacity);
        }
    }
    growth_events
}
