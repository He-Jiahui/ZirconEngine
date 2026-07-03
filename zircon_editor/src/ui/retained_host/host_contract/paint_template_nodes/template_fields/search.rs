use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;
#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEARCH_ICON_SIZE: f32 =
    16.0;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEARCH_FIELD_MAX_HEIGHT: f32 =
    METRICS.row_height + (METRICS.border_width * 4.0);

const SEARCH_FIELD_ICON: &str = "search";

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
    let metrics = current_host_metrics();
    if is_search_field(node) {
        metrics.input_pad[0] + SEARCH_ICON_SIZE + metrics.gap_s
    } else {
        metrics.input_pad[0]
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_field_paint_rect(
    node: &TemplatePaneNodeData,
    rect: FrameRect,
) -> FrameRect {
    let metrics = current_host_metrics();
    let max_height = metrics.row_height + (metrics.border_width * 4.0);
    if !is_search_field(node) || rect.height <= max_height {
        return rect;
    }

    let height = max_height.round().max(1.0);
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

    let icon = search_icon_rect(rect);
    if push_icon_asset_pixels(
        commands,
        SEARCH_FIELD_ICON,
        &icon,
        clip,
        order,
        Some(color),
        opacity,
    ) {
        return;
    }

    let metrics = current_host_metrics();
    let ring = search_icon_ring_rect(&icon, metrics.gap_m);
    commands.push(HostPaintCommand::quad(
        ring.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        metrics.border_width,
        metrics.gap_m * 0.5,
        opacity,
    ));

    for segment in search_handle_segments(&ring, metrics.border_width) {
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

fn search_icon_rect(rect: &FrameRect) -> FrameRect {
    let metrics = current_host_metrics();
    let icon_left = (rect.x + metrics.input_pad[0]).round();
    let icon_top = (rect.y + (rect.height - SEARCH_ICON_SIZE).max(0.0) * 0.5).round();
    FrameRect {
        x: icon_left,
        y: icon_top,
        width: SEARCH_ICON_SIZE,
        height: SEARCH_ICON_SIZE,
    }
}

fn search_icon_ring_rect(icon: &FrameRect, ring_size: f32) -> FrameRect {
    FrameRect {
        x: icon.x + ((SEARCH_ICON_SIZE - ring_size) * 0.5).round(),
        y: icon.y + ((SEARCH_ICON_SIZE - ring_size) * 0.5).round(),
        width: ring_size,
        height: ring_size,
    }
}

fn search_handle_segments(ring: &FrameRect, border_width: f32) -> [FrameRect; 3] {
    let segment_size = border_width * 2.0;
    [
        search_handle_segment(ring, border_width, segment_size, 0.0),
        search_handle_segment(ring, border_width, segment_size, segment_size),
        search_handle_segment(ring, border_width, segment_size, segment_size * 2.0),
    ]
}

fn search_handle_segment(
    ring: &FrameRect,
    border_width: f32,
    segment_size: f32,
    offset: f32,
) -> FrameRect {
    FrameRect {
        x: ring.x + ring.width - border_width + offset,
        y: ring.y + ring.height - border_width + offset,
        width: segment_size,
        height: segment_size,
    }
}
