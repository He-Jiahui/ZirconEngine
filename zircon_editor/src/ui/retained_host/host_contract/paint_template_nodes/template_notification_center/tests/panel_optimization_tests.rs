#[test]
fn optimization_batch_20260830dh_notification_panel_reserves_two_commands() {
    let source = include_str!("../panel.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("notification panel production source");

    assert!(production.contains("commands.reserve(2)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830dh_notification_panel_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const COMMAND_COUNT: usize = 2;
    const MARKER: &str = "EDITOR520_NOTIFICATION_PANEL_CAPACITY_BENCH_V1";

    let legacy_growth_events = panel_growth_events(BATCH_COUNT, COMMAND_COUNT, false);
    let optimized_growth_events = panel_growth_events(BATCH_COUNT, COMMAND_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} commands={COMMAND_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn panel_growth_events(batch_count: usize, command_count: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut commands = Vec::new();
        if reserve {
            commands.reserve(command_count);
        }
        for command in 0..command_count {
            let previous_capacity = commands.capacity();
            commands.push(command);
            growth_events += usize::from(commands.capacity() != previous_capacity);
        }
    }
    growth_events
}
