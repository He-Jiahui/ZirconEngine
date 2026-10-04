use std::ops::Range;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    PaneContentSize, PaneData, PerformanceTimelinePaneViewData,
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
    for (row, control) in data.capture_controls.iter().enumerate() {
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
        node.action_id = control.action_id.clone();
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
    let visible_rows = visible_row_range(data.frame_rows.row_count(), list_frame.y, list_frame);
    let mut nodes = Vec::with_capacity(visible_rows.len().saturating_mul(4));
    for row in visible_rows {
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
    let visible_rows = visible_row_range(data.span_rows.row_count(), start_y, list_frame);
    let mut nodes = Vec::with_capacity(visible_rows.len());
    for row in visible_rows {
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
    let visible_rows = visible_row_range(data.hotspot_rows.row_count(), start_y, list_frame);
    let mut nodes = Vec::with_capacity(visible_rows.len());
    for row in visible_rows {
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

fn visible_row_range(
    row_count: usize,
    start_y: f32,
    clip: &host_contract::TemplateNodeFrameData,
) -> Range<usize> {
    let row_stride = ROW_HEIGHT + ROW_GAP;
    let clip_start = clip.y;
    let clip_end = clip.y + clip.height.max(0.0);
    if row_count == 0
        || row_stride <= 0.0
        || !start_y.is_finite()
        || !clip_start.is_finite()
        || !clip_end.is_finite()
        || clip_end <= clip_start
    {
        return 0..0;
    }

    // Row i intersects the clip when row_end > clip_start and row_start < clip_end.
    let first =
        (((clip_start - start_y - ROW_HEIGHT) / row_stride).floor() as isize + 1).max(0) as usize;
    let end = (((clip_end - start_y) / row_stride).ceil() as isize).max(0) as usize;
    let first = first.min(row_count);
    first..end.min(row_count).max(first)
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
#[path = "tests/performance_timeline.rs"]
mod tests;
