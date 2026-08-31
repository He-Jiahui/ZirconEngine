use super::*;

#[test]
fn console_output_model_preserves_logical_lines_and_empty_state() {
    assert_eq!(
        console_output_lines("compile started\r\ncache ready\n").collect::<Vec<_>>(),
        vec!["compile started", "cache ready", ""]
    );
    assert_eq!(
        console_output_lines("").collect::<Vec<_>>(),
        vec!["No output yet"]
    );
    assert_eq!(
        console_output_lines_with_presence("", true).collect::<Vec<_>>(),
        vec![""]
    );
    assert_eq!(console_output_line_count(""), 1);
    assert_eq!(console_output_line_count("one"), 1);
    assert_eq!(console_output_line_count("one\ntwo\n"), 3);
    assert_eq!(console_content_extent("one\ntwo\nthree"), 54.0);
    assert_eq!(
        console_snapshot_content_extent(&ConsoleOutputSnapshot::from("one\ntwo\nthree")),
        54.0
    );
}

#[test]
fn console_output_metadata_visits_only_visible_lines_and_fixed_nodes() {
    let metadata = ConsoleOutputPaintMetadata::new(
        ConsoleOutputViewport {
            x: 4.0,
            y: 20.0,
            width: 200.0,
            height: 36.0,
        },
        20.0,
        2,
        5,
    )
    .expect("valid console output metadata");

    assert_eq!(metadata.visible_node_rows(8, 18.0), vec![0, 1, 3, 4, 7]);
}

#[test]
fn console_output_metadata_visits_every_node_owned_by_visible_logical_lines() {
    let metadata = ConsoleOutputPaintMetadata::new_with_nodes_per_line(
        ConsoleOutputViewport {
            x: 4.0,
            y: 20.0,
            width: 200.0,
            height: 36.0,
        },
        20.0,
        2,
        5,
        2,
    )
    .expect("valid composite console output metadata");

    assert_eq!(metadata.content_extent(), 90.0);
    assert_eq!(
        metadata.visible_node_rows(13, 18.0),
        vec![0, 1, 4, 5, 6, 7, 12]
    );
}

#[test]
fn console_output_metadata_static_rows_skip_the_entire_materialized_line_window() {
    let metadata = ConsoleOutputPaintMetadata::new_with_nodes_per_line(
        ConsoleOutputViewport {
            x: 4.0,
            y: 20.0,
            width: 200.0,
            height: 36.0,
        },
        20.0,
        2,
        256,
        2,
    )
    .expect("valid composite console output metadata");

    assert_eq!(
        metadata.static_node_rows(515).collect::<Vec<_>>(),
        vec![0, 1, 514]
    );
    assert_eq!(
        metadata.static_node_rows(515).rev().collect::<Vec<_>>(),
        vec![514, 1, 0]
    );
}

#[test]
fn console_output_metadata_rejects_zero_nodes_per_line_and_clamps_truncated_models() {
    let viewport = ConsoleOutputViewport {
        x: 4.0,
        y: 20.0,
        width: 200.0,
        height: 36.0,
    };
    assert!(ConsoleOutputPaintMetadata::new_with_nodes_per_line(viewport, 20.0, 2, 5, 0).is_none());

    let metadata = ConsoleOutputPaintMetadata::new_with_nodes_per_line(viewport, 20.0, 2, 5, 2)
        .expect("valid composite console output metadata");
    assert_eq!(metadata.visible_node_rows(5, 18.0), vec![0, 1]);
}

#[test]
fn console_output_virtualized_metadata_bounds_slots_independently_of_logical_rows() {
    let metadata = ConsoleOutputPaintMetadata::new_virtualized(
        ConsoleOutputViewport {
            x: 4.0,
            y: 20.0,
            width: 200.0,
            height: 36.0,
        },
        20.0,
        2,
        logical_lines(8_000),
        2,
        CONSOLE_OUTPUT_OVERSCAN_LINES,
    )
    .expect("valid virtualized console output metadata");

    assert_eq!(metadata.logical_line_count(), 8_000);
    assert_eq!(metadata.materialized_line_count(), 7);
    assert_eq!(metadata.materialized_node_count(), 14);
    assert_eq!(metadata.overscan_line_count(), 2);
    assert_eq!(metadata.visible_logical_line_count(1_800.0), 2);
    assert_eq!(
        metadata.visible_node_rows(17, 1_800.0),
        vec![0, 1, 6, 7, 8, 9, 16]
    );

    let severity = metadata
        .logical_line_for_node_row(6, 17, 1_800.0)
        .expect("logical line 100 severity binding");
    assert_eq!(severity.logical_index, 100);
    assert_eq!(severity.kind, ConsoleOutputSlotKind::Severity);
    let message = metadata
        .logical_line_for_node_row(7, 17, 1_800.0)
        .expect("logical line 100 message binding");
    assert_eq!(message.logical_index, 100);
    assert_eq!(message.kind, ConsoleOutputSlotKind::Message);
    assert_eq!(message.line.text(), "line-0100");
}

#[test]
fn console_output_virtualized_metadata_rebinds_only_the_entered_ring_slot() {
    let metadata = ConsoleOutputPaintMetadata::new_virtualized(
        ConsoleOutputViewport {
            x: 4.0,
            y: 20.0,
            width: 200.0,
            height: 36.0,
        },
        20.0,
        2,
        logical_lines(8_000),
        2,
        CONSOLE_OUTPUT_OVERSCAN_LINES,
    )
    .expect("valid virtualized console output metadata");

    let (before_index, before_line) = metadata
        .logical_line_for_slot(4, 1_800.0)
        .expect("slot four before scroll");
    let (after_index, after_line) = metadata
        .logical_line_for_slot(4, 1_818.0)
        .expect("slot four after scroll");
    assert_eq!(before_index, 102);
    assert_eq!(after_index, 102);
    assert!(std::ptr::eq(before_line, after_line));

    assert_eq!(
        metadata
            .logical_line_for_slot(0, 1_800.0)
            .map(|(index, _)| index),
        Some(98)
    );
    assert_eq!(
        metadata
            .logical_line_for_slot(0, 1_818.0)
            .map(|(index, _)| index),
        Some(105)
    );
}

fn logical_lines(count: usize) -> Vec<ConsoleOutputLogicalLine> {
    (0..count)
        .map(|index| {
            ConsoleOutputLogicalLine::new(format!("line-{index:04}"), "secondary".into())
                .with_severity("[Info]".into(), "secondary".into())
        })
        .collect()
}
