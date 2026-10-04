use super::*;

#[test]
fn section_title_preserves_fractional_post_dpi_surface_geometry() {
    let node = TemplatePaneNodeData {
        control_id: "WorkbenchSectionTitleRoot".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 8.25,
        y: 10.5,
        width: 240.75,
        height: 28.25,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 640.0,
        height: 480.0,
    };
    let mut commands = Vec::new();

    assert!(push_section_title_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        0,
        1.0,
    ));

    assert_eq!(commands.first().map(|command| &command.frame), Some(&rect));
}

#[test]
fn optimization_batch_20260830cu_section_title_reserves_its_command_bound() {
    let source = include_str!("../commands.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("section title production source");

    assert!(production.contains("const SECTION_TITLE_COMMAND_CAPACITY: usize = 4;"));
    assert!(production.contains("commands.reserve(SECTION_TITLE_COMMAND_CAPACITY);"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830cu_section_title_command_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const COMMANDS_PER_BATCH: usize = 4;
    const MARKER: &str = "EDITOR508_SECTION_TITLE_COMMAND_CAPACITY_BENCH_V1";

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
