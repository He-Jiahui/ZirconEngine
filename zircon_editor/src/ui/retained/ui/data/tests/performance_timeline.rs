use super::*;
use crate::ui::layouts::windows::workbench_host_window::PerformanceTimelineFrameRowViewData;

#[test]
fn frame_rows_project_budget_bar_nodes() {
    let data = PerformanceTimelinePaneViewData {
        frame_rows: model_rc(vec![PerformanceTimelineFrameRowViewData {
            stream: "editor".into(),
            name: "retained_host_tick".into(),
            frame_index: 7,
            duration_label: "20.00 ms".into(),
            budget_label: "16.67 ms budget".into(),
            budget_usage_label: "120% budget".into(),
            duration_ratio: 1.2,
            bar_fill_ratio: 1.0,
            budget_marker_ratio: 0.8335,
            over_budget: true,
        }]),
        span_rows: model_rc(Vec::new()),
        hotspot_rows: model_rc(Vec::new()),
        capture_controls: model_rc(Vec::new()),
        summary: "Profiling active".into(),
        session_label: "Session local".into(),
        output_label: "Output target/zircon-profiles/local".into(),
    };

    let nodes = performance_timeline_nodes(&data, &[], PaneContentSize::new(240.0, 160.0));

    let track = find_node(&nodes, "PerformanceTimelineFrameBarTrack.0");
    assert_eq!(track.role.as_str(), "Panel");
    assert_eq!(track.surface_variant.as_str(), "inset");
    assert_eq!(track.frame.width, 240.0);

    let fill = find_node(&nodes, "PerformanceTimelineFrameBarFill.0");
    assert_eq!(fill.role.as_str(), "Panel");
    assert_eq!(fill.surface_variant.as_str(), "danger");
    assert_eq!(fill.frame.width, 240.0);

    let marker = find_node(&nodes, "PerformanceTimelineFrameBudgetMarker.0");
    assert_eq!(marker.validation_level.as_str(), "warning");
    assert!((marker.frame.x - 198.373).abs() < 0.01);
    assert_eq!(marker.frame.width, BUDGET_MARKER_WIDTH);

    let label = find_node(&nodes, "PerformanceTimelineFrameLabel.0");
    assert_eq!(label.role.as_str(), "Label");
    assert_eq!(label.text_tone.as_str(), "warning");
    assert!(label
        .text
        .as_str()
        .contains("120% budget / 16.67 ms budget"));
}

#[test]
fn template_text_updates_do_not_duplicate_static_nodes() {
    let data = PerformanceTimelinePaneViewData {
        summary: "Profiling active".into(),
        session_label: "Session local".into(),
        output_label: "Output target/zircon-profiles/local".into(),
        ..PerformanceTimelinePaneViewData::default()
    };
    let mut static_nodes = vec![
        template_node("PerformanceTimelineSummary", "Profiling disabled"),
        template_node("PerformanceTimelineSession", "Session pending"),
        template_node("PerformanceTimelineOutput", "Output pending"),
    ];

    apply_template_text_nodes(&data, &mut static_nodes);
    let dynamic_nodes =
        performance_timeline_nodes(&data, &static_nodes, PaneContentSize::new(240.0, 160.0));

    assert_eq!(static_nodes.len(), 3);
    assert_eq!(static_nodes[0].text.as_str(), "Profiling active");
    assert_eq!(static_nodes[1].text.as_str(), "Session local");
    assert_eq!(
        static_nodes[2].text.as_str(),
        "Output target/zircon-profiles/local"
    );
    assert_eq!(static_nodes[2].text_tone.as_str(), "muted");
    assert!(!dynamic_nodes.iter().any(|node| matches!(
        node.control_id.as_str(),
        "PerformanceTimelineSummary" | "PerformanceTimelineSession" | "PerformanceTimelineOutput"
    )));
}

#[test]
fn large_timeline_materializes_only_rows_intersecting_the_list_clip() {
    const LOGICAL_ROWS: usize = 10_000;
    let frame_rows = (0..LOGICAL_ROWS)
        .map(|frame_index| PerformanceTimelineFrameRowViewData {
            stream: "editor".into(),
            name: "retained_host_tick".into(),
            frame_index: frame_index as u64,
            duration_label: "1.00 ms".into(),
            budget_label: "16.67 ms budget".into(),
            budget_usage_label: "6% budget".into(),
            duration_ratio: 0.06,
            bar_fill_ratio: 0.06,
            budget_marker_ratio: 1.0,
            over_budget: false,
        })
        .collect();
    let data = PerformanceTimelinePaneViewData {
        frame_rows: model_rc(frame_rows),
        ..PerformanceTimelinePaneViewData::default()
    };

    let nodes = performance_timeline_nodes(&data, &[], PaneContentSize::new(240.0, 160.0));

    assert_eq!(data.frame_rows.row_count(), LOGICAL_ROWS);
    assert!(
        nodes.len() < 100,
        "visible timeline nodes were {}",
        nodes.len()
    );
    assert!(find_node(&nodes, "PerformanceTimelineFrameLabel.0")
        .text
        .as_str()
        .contains("#0"));
    assert!(nodes.iter().all(|node| {
        !node
            .control_id
            .as_str()
            .starts_with("PerformanceTimelineFrameLabel.")
            || node.frame.y < 160.0
    }));
}

fn template_node(control_id: &str, text: &str) -> host_contract::TemplatePaneNodeData {
    host_contract::TemplatePaneNodeData {
        control_id: control_id.into(),
        text: text.into(),
        ..host_contract::TemplatePaneNodeData::default()
    }
}

fn find_node<'a>(
    nodes: &'a [host_contract::TemplatePaneNodeData],
    control_id: &str,
) -> &'a host_contract::TemplatePaneNodeData {
    nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} node should be projected"))
}
