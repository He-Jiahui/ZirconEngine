use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEARCH_ICON_SIZE: f32 =
    METRICS.font_large;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEARCH_TEXT_GAP: f32 =
    METRICS.gap_s;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEARCH_FIELD_MAX_HEIGHT: f32 =
    METRICS.row_height + (METRICS.border_width * 4.0);

const SEARCH_ICON_RING_SIZE: f32 = METRICS.gap_m;
const SEARCH_ICON_STROKE: f32 = METRICS.border_width;
const SEARCH_HANDLE_SEGMENT_SIZE: f32 = METRICS.border_width * 2.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_search_field(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(node.component_role.as_str(), "search-field")
        || matches!(node.role.as_str(), "SearchField")
        || search_identity_text(node.control_id.as_str())
        || search_identity_text(node.binding_id.as_str())
        || search_identity_text(node.action_id.as_str())
        || search_identity_text(node.edit_action_id.as_str())
        || search_identity_text(node.commit_action_id.as_str())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_field_text_left(
    node: &TemplatePaneNodeData,
) -> f32 {
    if is_search_field(node) {
        METRICS.input_pad[0] + SEARCH_ICON_SIZE + SEARCH_TEXT_GAP
    } else {
        METRICS.input_pad[0]
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_field_paint_rect(
    node: &TemplatePaneNodeData,
    rect: FrameRect,
) -> FrameRect {
    if !is_search_field(node) || rect.height <= SEARCH_FIELD_MAX_HEIGHT {
        return rect;
    }

    let height = SEARCH_FIELD_MAX_HEIGHT.round().max(1.0);
    FrameRect {
        y: (rect.y + ((rect.height - height) * 0.5)).round(),
        height,
        ..rect
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_field_label_is_placeholder(
    node: &TemplatePaneNodeData,
    label: &str,
) -> bool {
    is_search_field(node) && node.value_text.trim().is_empty() && !label.trim().is_empty()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_search_field_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    color: [u8; 4],
) {
    if !is_search_field(node) {
        return;
    }

    let ring = search_icon_ring_rect(rect);
    commands.push(HostPaintCommand::quad(
        ring.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        SEARCH_ICON_STROKE,
        SEARCH_ICON_RING_SIZE * 0.5,
        opacity,
    ));

    for segment in search_handle_segments(&ring) {
        commands.push(HostPaintCommand::quad(
            segment,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

fn search_identity_text(value: &str) -> bool {
    value.to_ascii_lowercase().contains("search")
}

fn search_icon_ring_rect(rect: &FrameRect) -> FrameRect {
    let icon_left = (rect.x + METRICS.input_pad[0]).round();
    let icon_top = (rect.y + (rect.height - SEARCH_ICON_SIZE).max(0.0) * 0.5).round();
    FrameRect {
        x: icon_left + METRICS.border_width,
        y: icon_top + METRICS.border_width,
        width: SEARCH_ICON_RING_SIZE,
        height: SEARCH_ICON_RING_SIZE,
    }
}

fn search_handle_segments(ring: &FrameRect) -> [FrameRect; 3] {
    [
        search_handle_segment(ring, 0.0),
        search_handle_segment(ring, SEARCH_HANDLE_SEGMENT_SIZE),
        search_handle_segment(ring, SEARCH_HANDLE_SEGMENT_SIZE * 2.0),
    ]
}

fn search_handle_segment(ring: &FrameRect, offset: f32) -> FrameRect {
    FrameRect {
        x: ring.x + ring.width - METRICS.border_width + offset,
        y: ring.y + ring.height - METRICS.border_width + offset,
        width: SEARCH_HANDLE_SEGMENT_SIZE,
        height: SEARCH_HANDLE_SEGMENT_SIZE,
    }
}
