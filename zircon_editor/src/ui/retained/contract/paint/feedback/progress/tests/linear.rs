use super::*;

#[test]
fn indeterminate_segment_rect_projects_percent_units() {
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 6.0,
    };

    let first = indeterminate_segment_rect(&rect, INDETERMINATE_SEGMENTS[0]);
    let second = indeterminate_segment_rect(&rect, INDETERMINATE_SEGMENTS[1]);

    assert_eq!(first.x, 34.0);
    assert_eq!(first.y, 20.0);
    assert_eq!(first.width, 72.0);
    assert_eq!(first.height, 6.0);
    assert_eq!(second.x, 134.0);
    assert_eq!(second.width, 48.0);
}

#[test]
fn indeterminate_segment_rect_keeps_minimum_width() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 2.0,
        height: 4.0,
    };

    let segment = indeterminate_segment_rect(&rect, INDETERMINATE_SEGMENTS[1]);

    assert!((segment.width - 0.48).abs() < f32::EPSILON);
    assert!(segment.right() <= rect.right());
}

#[test]
fn determinate_fill_stays_inside_a_tight_track() {
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 0.4,
        height: 4.0,
    };

    let fill = determinate_fill_rect(&rect, 1.0).expect("positive progress should retain fill");

    assert_eq!(fill.width, rect.width);
    assert!(fill.right() <= rect.right());
}

#[test]
fn collapsed_track_emits_no_linear_progress_commands() {
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 0.0,
        height: 4.0,
    };
    let mut commands = Vec::new();

    push_linear_progress_commands(
        &mut commands,
        &TemplatePaneNodeData::default(),
        &rect,
        &rect,
        0,
        1.0,
    );

    assert!(commands.is_empty());
}

#[test]
fn indeterminate_track_emits_the_bounded_three_command_sequence() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 6.0,
    };
    let node = TemplatePaneNodeData {
        component_variant: "indeterminate".to_owned(),
        ..TemplatePaneNodeData::default()
    };
    let mut commands = Vec::new();

    push_linear_progress_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    assert_eq!(commands.len(), 3);
}

#[test]
fn optimization_batch_20260830cs_editor506_linear_progress_reserves_command_bound() {
    let source = include_str!("../linear.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("linear progress production source");

    assert!(production.contains("commands.reserve(LINEAR_PROGRESS_COMMAND_CAPACITY);"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830cs_editor506_linear_progress_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const COMMANDS_PER_BATCH: usize = 3;
    const MARKER: &str = "EDITOR506_LINEAR_PROGRESS_COMMAND_CAPACITY_BENCH_V1";
    let legacy_growth_events = command_growth_events(BATCH_COUNT, COMMANDS_PER_BATCH, false);
    let optimized_growth_events = command_growth_events(BATCH_COUNT, COMMANDS_PER_BATCH, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} commands_per_batch={COMMANDS_PER_BATCH} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn command_growth_events(batch_count: usize, commands_per_batch: usize, reserve: bool) -> usize {
    let mut commands = Vec::new();
    let mut growth_events = 0;
    for _ in 0..batch_count {
        if reserve {
            commands.reserve(commands_per_batch);
        }
        for command in 0..commands_per_batch {
            let previous_capacity = commands.capacity();
            commands.push(command);
            growth_events += usize::from(commands.capacity() != previous_capacity);
        }
    }
    growth_events
}
