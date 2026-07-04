use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::raster_size_from_frame;

use super::super::metrics::material_feedback_metrics;
use super::super::state::{
    progress_fill_color, progress_is_indeterminate, progress_percent, progress_track_color,
};
use super::geometry::circular_progress_rect;
use super::key::circular_progress_image_key;
use super::pixels::circular_progress_pixels;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_circular_progress_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let image_rect = circular_progress_rect(rect);
    let Some((width, height)) = raster_size_from_frame(image_rect.width, image_rect.height) else {
        return;
    };
    let size = width.min(height);
    if size == 0 {
        return;
    }

    let progress = if progress_is_indeterminate(node) {
        material_feedback_metrics().circular_indeterminate_percent
    } else {
        progress_percent(node)
    };
    let track = progress_track_color(node);
    let fill = progress_fill_color(node);
    let rgba = circular_progress_pixels(size, progress, track, fill);
    commands.push(HostPaintCommand::image_pixels(
        image_rect,
        Some(clip.clone()),
        order,
        circular_progress_image_key(size, progress_percent(node), track, fill),
        size,
        size,
        rgba,
        None,
        opacity,
    ));
}
