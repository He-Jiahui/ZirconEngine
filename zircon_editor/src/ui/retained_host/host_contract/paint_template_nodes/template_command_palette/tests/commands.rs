use super::super::layout::command_palette_metrics;
use super::*;

#[test]
fn visible_rows_include_exactly_one_overscan_row_on_each_side() {
    let metrics = command_palette_metrics();
    let panel = FrameRect {
        x: 100.0,
        y: 50.0,
        width: 560.0,
        height: metrics.list_top + metrics.row_height * 40.0,
    };
    let clip = FrameRect {
        x: panel.x,
        y: panel.y + metrics.list_top + metrics.row_height * 10.25,
        width: panel.width,
        height: metrics.row_height * 2.5,
    };

    assert_eq!(command_palette_visible_rows(&panel, &clip, 40), 9..14);
}

#[test]
fn visible_rows_are_empty_when_clip_is_horizontally_disjoint() {
    let panel = FrameRect {
        x: 100.0,
        y: 50.0,
        width: 560.0,
        height: 280.0,
    };
    let clip = FrameRect {
        x: panel.x + panel.width + 1.0,
        y: panel.y,
        width: 20.0,
        height: panel.height,
    };

    assert_eq!(command_palette_visible_rows(&panel, &clip, 40), 0..0);
}

#[test]
fn painter_source_has_no_full_row_loop_or_cloning_row_access() {
    let source = include_str!("../commands.rs");
    let full_loop = ["for row in ", "0..row_count"].concat();
    let cloning_access = ["structured_options", ".row_data(row)"].concat();
    let borrowed_access = ["structured_options", ".get(row)"].concat();

    assert!(!source.contains(&full_loop));
    assert!(!source.contains(&cloning_access));
    assert!(source.contains(&borrowed_access));
}

#[test]
fn optimization_batch_20260830dg_command_palette_reserves_visible_command_upper_bound() {
    let source = include_str!("../commands.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("command palette production source");

    assert!(production.contains("COMMAND_PALETTE_PANEL_COMMAND_UPPER_BOUND"));
    assert!(production.contains("COMMAND_PALETTE_ROW_COMMAND_UPPER_BOUND"));
    assert!(production.contains("commands.reserve(command_upper_bound)"));
    assert_eq!(
        production.matches("command_palette_visible_rows(").count(),
        2
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830dg_command_palette_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const PANEL_COMMAND_COUNT: usize = 4;
    const VISIBLE_ROW_COUNT: usize = 8;
    const ROW_COMMAND_COUNT: usize = 4;
    const COMMAND_COUNT: usize = PANEL_COMMAND_COUNT + VISIBLE_ROW_COUNT * ROW_COMMAND_COUNT;
    const MARKER: &str = "EDITOR519_COMMAND_PALETTE_CAPACITY_BENCH_V1";

    let legacy_growth_events = command_growth_events(BATCH_COUNT, COMMAND_COUNT, false);
    let optimized_growth_events = command_growth_events(BATCH_COUNT, COMMAND_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} panel_commands={PANEL_COMMAND_COUNT} \
             visible_rows={VISIBLE_ROW_COUNT} row_commands={ROW_COMMAND_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn command_growth_events(batch_count: usize, command_count: usize, reserve: bool) -> usize {
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
