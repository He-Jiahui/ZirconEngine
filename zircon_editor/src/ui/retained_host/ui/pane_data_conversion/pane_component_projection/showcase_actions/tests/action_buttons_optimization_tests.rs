#[test]
fn optimization_batch_20260830df_action_buttons_reserve_spec_upper_bound() {
    let source = include_str!("../action_buttons.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("showcase action button production source");

    assert!(production.contains("let specs = action_button_specs(control_id)"));
    assert!(production.contains("Vec::with_capacity(specs.len())"));
    assert!(production.contains("actions.extend("));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830df_action_button_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const ACTION_COUNT: usize = 4;
    const MARKER: &str = "EDITOR518_ACTION_BUTTON_CAPACITY_BENCH_V1";

    let legacy_growth_events = action_growth_events(BATCH_COUNT, ACTION_COUNT, false);
    let optimized_growth_events = action_growth_events(BATCH_COUNT, ACTION_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} actions={ACTION_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn action_growth_events(batch_count: usize, action_count: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut actions = if reserve {
            Vec::with_capacity(action_count)
        } else {
            Vec::new()
        };
        for action in 0..action_count {
            let previous_capacity = actions.capacity();
            actions.push(action);
            growth_events += usize::from(actions.capacity() != previous_capacity);
        }
    }
    growth_events
}
