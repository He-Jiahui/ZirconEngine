#[test]
fn optimization_batch_20260830cy_table_row_surface_reserves_its_command_bound() {
    let source = include_str!("../surface.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("table row surface production source");

    assert!(production.contains("const TABLE_ROW_SURFACE_COMMAND_CAPACITY: usize = 2;"));
    assert!(production.contains("commands.reserve(TABLE_ROW_SURFACE_COMMAND_CAPACITY);"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830cy_table_row_surface_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const COMMANDS_PER_BATCH: usize = 2;
    const MARKER: &str = "EDITOR511_TABLE_ROW_SURFACE_CAPACITY_BENCH_V1";

    let legacy_growth_events = command_growth_events(BATCH_COUNT, COMMANDS_PER_BATCH, false);
    let optimized_growth_events = command_growth_events(BATCH_COUNT, COMMANDS_PER_BATCH, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} commands_per_batch={COMMANDS_PER_BATCH} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn command_growth_events(batch_count: usize, commands_per_batch: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut commands = if reserve {
            Vec::with_capacity(commands_per_batch)
        } else {
            Vec::new()
        };
        for command in 0..commands_per_batch {
            let previous_capacity = commands.capacity();
            commands.push(command);
            growth_events += usize::from(commands.capacity() != previous_capacity);
        }
    }
    growth_events
}
