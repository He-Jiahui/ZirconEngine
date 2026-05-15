use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    PaneContentSize, PaneData, PerformanceTimelineCaptureControlViewData,
    PerformanceTimelineFrameRowViewData, PerformanceTimelineHotspotRowViewData,
    PerformanceTimelinePaneViewData, PerformanceTimelineSpanRowViewData,
};
use crate::ui::retained_host as host_contract;

const ROW_HEIGHT: f32 = 24.0;
const ROW_GAP: f32 = 4.0;
const ROW_PADDING: f32 = 8.0;
const SECTION_GAP: f32 = 8.0;
const CONTROL_BUTTON_WIDTH: f32 = 112.0;
const CONTROL_BUTTON_HEIGHT: f32 = 24.0;
const CONTROL_BUTTON_GAP: f32 = 6.0;
const BUDGET_MARKER_WIDTH: f32 = 2.0;

pub(crate) fn to_host_contract_performance_timeline_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::PerformanceTimelinePaneData {
    let native = &data.native_body.performance_timeline;
    let mut nodes =
        performance_timeline_template_projection(data, content_size).unwrap_or_default();
    apply_template_text_nodes(native, &mut nodes);
    nodes.extend(performance_timeline_nodes(native, &nodes, content_size));

    host_contract::PerformanceTimelinePaneData {
        nodes: model_rc(nodes),
        frame_rows: super::map_model_rc(&native.frame_rows, to_host_contract_frame_row),
        span_rows: super::map_model_rc(&native.span_rows, to_host_contract_span_row),
        hotspot_rows: super::map_model_rc(&native.hotspot_rows, to_host_contract_hotspot_row),
        capture_controls: super::map_model_rc(
            &native.capture_controls,
            to_host_contract_capture_control,
        ),
        summary: native.summary.clone(),
        session_label: native.session_label.clone(),
        output_label: native.output_label.clone(),
    }
}

fn to_host_contract_frame_row(
    row: PerformanceTimelineFrameRowViewData,
) -> host_contract::PerformanceTimelineFrameRowData {
    host_contract::PerformanceTimelineFrameRowData {
        stream: row.stream,
        name: row.name,
        frame_index: row.frame_index,
        duration_label: row.duration_label,
        budget_label: row.budget_label,
        budget_usage_label: row.budget_usage_label,
        duration_ratio: row.duration_ratio,
        bar_fill_ratio: row.bar_fill_ratio,
        budget_marker_ratio: row.budget_marker_ratio,
        over_budget: row.over_budget,
    }
}

fn to_host_contract_span_row(
    row: PerformanceTimelineSpanRowViewData,
) -> host_contract::PerformanceTimelineSpanRowData {
    host_contract::PerformanceTimelineSpanRowData {
        stream: row.stream,
        category: row.category,
        name: row.name,
        path: row.path,
        duration_label: row.duration_label,
        depth: row.depth,
    }
}

fn to_host_contract_hotspot_row(
    row: PerformanceTimelineHotspotRowViewData,
) -> host_contract::PerformanceTimelineHotspotRowData {
    host_contract::PerformanceTimelineHotspotRowData {
        stream: row.stream,
        category: row.category,
        name: row.name,
        path: row.path,
        total_label: row.total_label,
        average_label: row.average_label,
        count_label: row.count_label,
    }
}

fn to_host_contract_capture_control(
    control: PerformanceTimelineCaptureControlViewData,
) -> host_contract::PerformanceTimelineCaptureControlData {
    host_contract::PerformanceTimelineCaptureControlData {
        label: control.label,
        action_id: control.action_id,
        enabled: control.enabled,
    }
}

fn performance_timeline_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        crate::ui::layouts::windows::workbench_host_window::PanePayload::PerformanceTimelineV1(_)
    ) {
        return None;
    }

    super::project_pane_template_nodes(&presentation.body, content_size)
}

fn performance_timeline_nodes(
    data: &PerformanceTimelinePaneViewData,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let list_frame = template_nodes
        .iter()
        .find(|node| {
            matches!(
                node.control_id.as_str(),
                "PerformanceTimelineFrameListSlotAnchor" | "PerformanceTimelineFrameList"
            )
        })
        .map(|node| node.frame.clone())
        .unwrap_or_else(|| host_contract::TemplateNodeFrameData {
            x: 0.0,
            y: 82.0,
            width: content_size.width.max(0.0),
            height: (content_size.height - 82.0).max(0.0),
        });
    let list_width = list_frame.width.max(content_size.width).max(0.0);
    let mut nodes = Vec::new();
    nodes.extend(control_button_nodes(data, template_nodes, list_width));
    nodes.extend(frame_row_nodes(data, &list_frame, list_width));

    let span_start_y =
        list_frame.y + data.frame_rows.row_count() as f32 * (ROW_HEIGHT + ROW_GAP) + SECTION_GAP;
    nodes.extend(span_row_nodes(data, &list_frame, span_start_y, list_width));

    let hotspot_start_y =
        span_start_y + data.span_rows.row_count() as f32 * (ROW_HEIGHT + ROW_GAP) + SECTION_GAP;
    nodes.extend(hotspot_row_nodes(
        data,
        &list_frame,
        hotspot_start_y,
        list_width,
    ));
    nodes
}

fn apply_template_text_nodes(
    data: &PerformanceTimelinePaneViewData,
    template_nodes: &mut [host_contract::TemplatePaneNodeData],
) {
    for node in template_nodes {
        match node.control_id.as_str() {
            "PerformanceTimelineSummary" => node.text = data.summary.clone(),
            "PerformanceTimelineSession" => node.text = data.session_label.clone(),
            "PerformanceTimelineOutput" => {
                node.text = data.output_label.clone();
                node.text_tone = "muted".into();
            }
            _ => {}
        }
    }
}

fn control_button_nodes(
    data: &PerformanceTimelinePaneViewData,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    list_width: f32,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let controls_frame = template_nodes
        .iter()
        .find(|node| node.control_id.as_str() == "PerformanceTimelineCaptureControls")
        .map(|node| node.frame.clone())
        .unwrap_or_else(|| host_contract::TemplateNodeFrameData {
            x: 0.0,
            y: 28.0,
            width: list_width,
            height: CONTROL_BUTTON_HEIGHT,
        });
    let mut nodes = Vec::new();
    for row in 0..data.capture_controls.row_count() {
        let Some(control) = data.capture_controls.row_data(row) else {
            continue;
        };
        let mut node = timeline_node(
            format!("performance_timeline_control_{row}"),
            "PerformanceTimelineCaptureControl",
            "Button",
            control.label.to_string(),
            host_contract::TemplateNodeFrameData {
                x: controls_frame.x + row as f32 * (CONTROL_BUTTON_WIDTH + CONTROL_BUTTON_GAP),
                y: controls_frame.y,
                width: CONTROL_BUTTON_WIDTH,
                height: CONTROL_BUTTON_HEIGHT,
            },
        );
        node.control_id = "PerformanceTimelineCaptureControl".into();
        node.action_id = control.action_id;
        node.disabled = !control.enabled;
        node.surface_variant = if control.enabled { "accent" } else { "inset" }.into();
        node.text_tone = if control.enabled {
            "default"
        } else {
            "disabled"
        }
        .into();
        node.corner_radius = 4.0;
        nodes.push(node);
    }
    nodes
}

fn frame_row_nodes(
    data: &PerformanceTimelinePaneViewData,
    list_frame: &host_contract::TemplateNodeFrameData,
    list_width: f32,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let mut nodes = Vec::new();
    for row in 0..data.frame_rows.row_count() {
        let Some(frame) = data.frame_rows.row_data(row) else {
            continue;
        };
        let y = list_frame.y + row as f32 * (ROW_HEIGHT + ROW_GAP);
        let mut track = timeline_node(
            format!("performance_timeline_frame_track_{row}"),
            format!("PerformanceTimelineFrameBarTrack.{row}"),
            "Panel",
            "",
            host_contract::TemplateNodeFrameData {
                x: list_frame.x,
                y,
                width: list_width,
                height: ROW_HEIGHT,
            },
        );
        track.surface_variant = "inset".into();
        track.corner_radius = 4.0;
        nodes.push(track);

        let mut fill = timeline_node(
            format!("performance_timeline_frame_fill_{row}"),
            format!("PerformanceTimelineFrameBarFill.{row}"),
            "Panel",
            "",
            host_contract::TemplateNodeFrameData {
                x: list_frame.x,
                y,
                width: frame_bar_width(list_width, frame.bar_fill_ratio),
                height: ROW_HEIGHT,
            },
        );
        fill.surface_variant = if frame.over_budget {
            "danger"
        } else {
            "accent"
        }
        .into();
        fill.corner_radius = 4.0;
        nodes.push(fill);

        let marker_x = budget_marker_x(list_frame.x, list_width, frame.budget_marker_ratio);
        let mut marker = timeline_node(
            format!("performance_timeline_frame_budget_{row}"),
            format!("PerformanceTimelineFrameBudgetMarker.{row}"),
            "Panel",
            "",
            host_contract::TemplateNodeFrameData {
                x: marker_x,
                y,
                width: BUDGET_MARKER_WIDTH.min(list_width.max(0.0)),
                height: ROW_HEIGHT,
            },
        );
        marker.validation_level = "warning".into();
        marker.corner_radius = 1.0;
        nodes.push(marker);

        let mut label = timeline_node(
            format!("performance_timeline_frame_label_{row}"),
            format!("PerformanceTimelineFrameLabel.{row}"),
            "Label",
            format!(
                "[{}] #{} {} - {} ({} / {})",
                frame.stream,
                frame.frame_index,
                frame.name,
                frame.duration_label,
                frame.budget_usage_label,
                frame.budget_label
            ),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + ROW_PADDING,
                y,
                width: (list_width - ROW_PADDING * 2.0).max(0.0),
                height: ROW_HEIGHT,
            },
        );
        label.text_tone = if frame.over_budget {
            "warning"
        } else {
            "default"
        }
        .into();
        nodes.push(label);
    }
    nodes
}

fn frame_bar_width(list_width: f32, ratio: f32) -> f32 {
    list_width.max(0.0) * ratio.clamp(0.0, 1.0)
}

fn budget_marker_x(list_x: f32, list_width: f32, ratio: f32) -> f32 {
    let marker_travel = (list_width - BUDGET_MARKER_WIDTH).max(0.0);
    list_x + marker_travel * ratio.clamp(0.0, 1.0)
}

fn span_row_nodes(
    data: &PerformanceTimelinePaneViewData,
    list_frame: &host_contract::TemplateNodeFrameData,
    start_y: f32,
    list_width: f32,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let mut nodes = Vec::new();
    for row in 0..data.span_rows.row_count() {
        let Some(span) = data.span_rows.row_data(row) else {
            continue;
        };
        let y = start_y + row as f32 * (ROW_HEIGHT + ROW_GAP);
        let mut node = timeline_node(
            format!("performance_timeline_span_{row}"),
            format!("PerformanceTimelineSpan.{row}"),
            "Label",
            format!(
                "{}:{} {} - {}",
                span.stream, span.category, span.name, span.duration_label
            ),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + ROW_PADDING * f32::from(span.depth.min(4)),
                y,
                width: (list_width - ROW_PADDING * f32::from(span.depth.min(4))).max(0.0),
                height: ROW_HEIGHT,
            },
        );
        node.text_tone = "muted".into();
        nodes.push(node);
    }
    nodes
}

fn hotspot_row_nodes(
    data: &PerformanceTimelinePaneViewData,
    list_frame: &host_contract::TemplateNodeFrameData,
    start_y: f32,
    list_width: f32,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let mut nodes = Vec::new();
    for row in 0..data.hotspot_rows.row_count() {
        let Some(hotspot) = data.hotspot_rows.row_data(row) else {
            continue;
        };
        let y = start_y + row as f32 * (ROW_HEIGHT + ROW_GAP);
        let mut node = timeline_node(
            format!("performance_timeline_hotspot_{row}"),
            format!("PerformanceTimelineHotspot.{row}"),
            "Label",
            format!(
                "Hotspot {}:{} {} - {}, {}, {}",
                hotspot.stream,
                hotspot.category,
                hotspot.name,
                hotspot.total_label,
                hotspot.average_label,
                hotspot.count_label
            ),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x,
                y,
                width: list_width,
                height: ROW_HEIGHT,
            },
        );
        node.surface_variant = "inset".into();
        node.text_tone = "warning".into();
        node.corner_radius = 4.0;
        nodes.push(node);
    }
    nodes
}

fn timeline_node(
    node_id: impl Into<String>,
    control_id: impl Into<String>,
    role: impl Into<String>,
    text: impl Into<String>,
    frame: host_contract::TemplateNodeFrameData,
) -> host_contract::TemplatePaneNodeData {
    host_contract::TemplatePaneNodeData {
        node_id: node_id.into().into(),
        control_id: control_id.into().into(),
        role: role.into().into(),
        text: text.into().into(),
        frame,
        ..host_contract::TemplatePaneNodeData::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "PerformanceTimelineSummary"
                | "PerformanceTimelineSession"
                | "PerformanceTimelineOutput"
        )));
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
}
