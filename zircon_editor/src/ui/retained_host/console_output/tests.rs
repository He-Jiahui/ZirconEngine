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
