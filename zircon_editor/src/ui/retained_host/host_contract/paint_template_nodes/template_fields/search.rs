use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::metrics::workbench_field_metrics;

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
    let metrics = workbench_field_metrics();
    if is_search_field(node) {
        metrics.search_text_left
    } else {
        metrics.input_pad_left
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_field_paint_rect(
    node: &TemplatePaneNodeData,
    rect: FrameRect,
) -> FrameRect {
    let metrics = workbench_field_metrics();
    let max_height = metrics.search_max_height;
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

    let metrics = workbench_field_metrics();
    let ring = search_icon_ring_rect(&icon, metrics.search_fallback_ring_size);
    commands.push(HostPaintCommand::quad(
        ring.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        metrics.border_width,
        metrics.search_fallback_radius,
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
    let metrics = workbench_field_metrics();
    let icon_left = (rect.x + metrics.input_pad_left).round();
    let icon_top = (rect.y + (rect.height - metrics.search_icon_size).max(0.0) * 0.5).round();
    FrameRect {
        x: icon_left,
        y: icon_top,
        width: metrics.search_icon_size,
        height: metrics.search_icon_size,
    }
}

fn search_icon_ring_rect(icon: &FrameRect, ring_size: f32) -> FrameRect {
    FrameRect {
        x: icon.x + ((icon.width - ring_size) * 0.5).round(),
        y: icon.y + ((icon.height - ring_size) * 0.5).round(),
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
