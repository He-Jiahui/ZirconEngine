use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::metrics::workbench_field_metrics;

const SEARCH_FIELD_ICON: &str = "search";
const SEARCH_RING_CENTER_FACTOR: f32 = 0.5;
const SEARCH_HANDLE_SEGMENT_SCALE: f32 = 2.0;
const SEARCH_HANDLE_SEGMENT_STEPS: [f32; 3] = [0.0, 1.0, 2.0];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_search_field_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    color: [u8; 4],
) {
    if !super::is_search_field(node) {
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
        x: icon.x + ((icon.width - ring_size) * SEARCH_RING_CENTER_FACTOR).round(),
        y: icon.y + ((icon.height - ring_size) * SEARCH_RING_CENTER_FACTOR).round(),
        width: ring_size,
        height: ring_size,
    }
}

fn search_handle_segments(ring: &FrameRect, border_width: f32) -> [FrameRect; 3] {
    let segment_size = border_width * SEARCH_HANDLE_SEGMENT_SCALE;
    SEARCH_HANDLE_SEGMENT_STEPS
        .map(|step| search_handle_segment(ring, border_width, segment_size, segment_size * step))
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
