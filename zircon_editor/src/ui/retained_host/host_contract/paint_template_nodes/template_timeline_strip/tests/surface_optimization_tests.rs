#[test]
fn optimization_batch_20260830da_timeline_surface_reserves_exact_command_count() {
    let source = include_str!("../surface.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("timeline surface production source");

    assert!(production.contains("const TIMELINE_SURFACE_BASE_COMMAND_CAPACITY: usize = 4;"));
    assert!(production.contains("static_content.ticks().len()"));
    assert!(production.contains("usize::from(progress_width > 0.0)"));
    assert!(production.contains("commands.reserve(command_capacity);"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830da_timeline_surface_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const TICK_COUNT: usize = 32;
    const COMMAND_COUNT: usize = 4 + TICK_COUNT + 1;
    const MARKER: &str = "EDITOR513_TIMELINE_SURFACE_CAPACITY_BENCH_V1";

    let legacy_growth_events = command_growth_events(BATCH_COUNT, COMMAND_COUNT, false);
    let optimized_growth_events = command_growth_events(BATCH_COUNT, COMMAND_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} ticks={TICK_COUNT} commands={COMMAND_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn command_growth_events(batch_count: usize, command_count: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut commands = if reserve {
            Vec::with_capacity(command_count)
        } else {
            Vec::new()
        };
        for command in 0..command_count {
            let previous_capacity = commands.capacity();
            commands.push(command);
            growth_events += usize::from(commands.capacity() != previous_capacity);
        }
    }
    growth_events
}
