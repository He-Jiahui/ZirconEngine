use super::*;
use std::sync::Arc;

use crate::ui::workbench::snapshot::{
    ConsoleOutputLineDelta, ConsoleOutputLineGeneration, ConsoleOutputLineSnapshot,
};

#[test]
fn console_projection_bounds_product_input_then_materializes_only_viewport_slots() {
    let status_text = (0..8_000)
        .map(|index| format!("line-{index:04}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut nodes = vec![
        node(CONSOLE_OUTPUT_BODY_CONTROL_ID, 20.0, 36.0),
        node(CONSOLE_OUTPUT_PROTOTYPE_CONTROL_ID, 20.0, 18.0),
        node("ConsoleFooter", 56.0, 18.0),
    ];

    let output = ConsoleOutputSnapshot::from(status_text);
    let metadata = project_console_output_lines(&mut nodes, &output)
        .expect("console output projection metadata");

    assert_eq!(metadata.logical_line_count(), 256);
    assert_eq!(metadata.materialized_line_count(), 7);
    assert_eq!(nodes.len(), 9);
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "ConsoleOutputLine0006"));
    assert!(!nodes
        .iter()
        .any(|node| node.control_id == "ConsoleOutputLine0007"));
}

#[test]
fn console_projection_keeps_severity_and_message_in_one_bounded_slot_pair() {
    let status_text = (0..256)
        .map(|index| format!("line-{index:04}"))
        .collect::<Vec<_>>()
        .join("\n");
    let levels = vec![EditorConsoleMessageLevel::Warning; 256];
    let output = ConsoleOutputSnapshot::new(status_text.into(), levels.into());
    let mut nodes = vec![
        node(CONSOLE_OUTPUT_BODY_CONTROL_ID, 20.0, 36.0),
        node(CONSOLE_OUTPUT_PROTOTYPE_CONTROL_ID, 20.0, 18.0),
        node("ConsoleFooter", 56.0, 18.0),
    ];

    let metadata = project_console_output_lines(&mut nodes, &output)
        .expect("composite console output projection metadata");

    assert_eq!(metadata.materialized_line_count(), 7);
    assert_eq!(nodes.len(), 16);
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "ConsoleOutputSeverity0006"));
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "ConsoleOutputLine0006"));
}

#[test]
fn console_projection_keeps_nodes_when_the_required_prototype_is_missing() {
    let mut nodes = vec![
        node(CONSOLE_OUTPUT_BODY_CONTROL_ID, 20.0, 36.0),
        node("ConsoleFooter", 56.0, 18.0),
    ];
    let original_control_ids = nodes
        .iter()
        .map(|node| node.control_id.clone())
        .collect::<Vec<_>>();

    let output = ConsoleOutputSnapshot::from("first\nsecond");
    assert!(project_console_output_lines(&mut nodes, &output).is_none());
    assert_eq!(
        nodes
            .iter()
            .map(|node| node.control_id.clone())
            .collect::<Vec<_>>(),
        original_control_ids
    );
}

#[test]
fn console_append_reuses_retained_projection_rows_and_rebinds_only_the_expired_slot() {
    let before_lines = (0..256)
        .map(|index| console_line(index, format!("line-{index:04}")))
        .collect::<Vec<_>>();
    let before_generation = Arc::new(ConsoleOutputLineGeneration::from_lines(before_lines));
    let before_output = ConsoleOutputSnapshot::from_line_generation(
        Arc::clone(&before_generation),
        ConsoleOutputLevelCounts {
            info: 256,
            ..ConsoleOutputLevelCounts::default()
        },
        ConsoleMessageFilter::All,
        ConsoleSourceFilter::All,
        ConsoleOutputLineDelta {
            entered: 256,
            expired: 0,
            retained: 0,
        },
    );
    let mut nodes = vec![
        node(CONSOLE_OUTPUT_BODY_CONTROL_ID, 20.0, 36.0),
        node(CONSOLE_OUTPUT_PROTOTYPE_CONTROL_ID, 20.0, 18.0),
        node("ConsoleFooter", 56.0, 18.0),
    ];
    let before_metadata = project_console_output_lines(&mut nodes, &before_output)
        .expect("initial console output projection metadata");
    let cached = host_contract::ConsolePaneData {
        nodes: ModelRc::with_metadata(nodes, before_metadata),
        output: before_output,
    };
    let (after_generation, delta) =
        before_generation.append_bounded(vec![console_line(256, "line-0256".to_string())], 256);
    let after_output = ConsoleOutputSnapshot::from_line_generation(
        Arc::new(after_generation),
        ConsoleOutputLevelCounts {
            info: 256,
            ..ConsoleOutputLevelCounts::default()
        },
        ConsoleMessageFilter::All,
        ConsoleSourceFilter::All,
        delta,
    );

    let reused = reuse_console_projection(&cached, &after_output)
        .expect("same-shape append must reuse the retained projection");
    let metadata = reused
        .nodes
        .metadata::<ConsoleOutputPaintMetadata>()
        .expect("reused projection metadata");

    assert_eq!(delta.entered, 1);
    assert_eq!(delta.expired, 1);
    assert_eq!(delta.retained, 255);
    assert!(!cached.nodes.shares_row_with(&reused.nodes, 1));
    assert!(!cached.nodes.shares_row_with(&reused.nodes, 2));
    for row in 3..cached.nodes.row_count() {
        assert!(
            cached.nodes.shares_row_with(&reused.nodes, row),
            "row {row}"
        );
    }
    assert_eq!(
        reused.nodes.get(2).map(|node| node.text.as_str()),
        Some("line-0007")
    );
    assert_eq!(metadata.slot_source_id(0, 0.0), Some(7));
    assert_eq!(metadata.slot_source_id(1, 0.0), Some(1));
    assert_eq!(
        metadata
            .logical_line_for_slot(4, metadata.content_extent())
            .map(|(_, line)| line.text()),
        Some("line-0256")
    );

    let repeated = reuse_console_projection(&reused, &after_output)
        .expect("the same generation must reuse every retained projection row");
    for row in 0..reused.nodes.row_count() {
        assert!(
            reused.nodes.shares_row_with(&repeated.nodes, row),
            "row {row}"
        );
    }
}

fn console_line(source_id: u64, text: String) -> ConsoleOutputLineSnapshot {
    ConsoleOutputLineSnapshot::new(
        source_id,
        text.into(),
        EditorConsoleMessageLevel::Info,
        None,
        None,
    )
}

fn node(control_id: &str, y: f32, height: f32) -> host_contract::TemplatePaneNodeData {
    host_contract::TemplatePaneNodeData {
        node_id: control_id.into(),
        control_id: control_id.into(),
        frame: host_contract::TemplateNodeFrameData {
            x: 8.0,
            y,
            width: 240.0,
            height,
        },
        ..host_contract::TemplatePaneNodeData::default()
    }
}
