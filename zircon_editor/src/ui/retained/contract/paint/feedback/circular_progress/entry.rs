use std::sync::Arc;

use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_geometry::intersect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::raster_size_from_frame;

use super::super::metrics::material_feedback_metrics;
use super::super::state::{
    progress_fill_color, progress_is_indeterminate, progress_percent, progress_track_color,
};
use super::cache::{
    cached_circular_progress_raster, store_circular_progress_raster, CachedCircularProgressRaster,
    CircularProgressRasterKey,
};
use super::geometry::circular_progress_rect;
use super::key::circular_progress_image_key_for_target;
use super::pixels::{circular_progress_pixels_for_target, normalized_circular_progress_percent};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_circular_progress_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let image_rect = circular_progress_rect(rect);
    if intersect(&image_rect, clip).is_none() {
        return;
    }
    let target_size = image_rect.width.min(image_rect.height);
    let Some((source_width, source_height)) = raster_size_from_frame(target_size, target_size)
    else {
        return;
    };
    let source_size = source_width.min(source_height);
    if source_size == 0 {
        return;
    }

    let progress = normalized_circular_progress_percent(if progress_is_indeterminate(node) {
        material_feedback_metrics().circular_indeterminate_percent
    } else {
        progress_percent(node)
    });
    let track = progress_track_color(node);
    let fill = progress_fill_color(node);
    let cache_key =
        CircularProgressRasterKey::with_target(source_size, target_size, progress, track, fill);
    let CachedCircularProgressRaster { resource_key, rgba } =
        cached_circular_progress_raster(cache_key).unwrap_or_else(|| {
            let rgba: Arc<[u8]> = circular_progress_pixels_for_target(
                source_size,
                target_size,
                progress,
                track,
                fill,
            )
            .into();
            let resource_key = circular_progress_image_key_for_target(
                source_size,
                target_size,
                progress,
                track,
                fill,
            );
            store_circular_progress_raster(cache_key, resource_key.clone(), Arc::clone(&rgba));
            CachedCircularProgressRaster { resource_key, rgba }
        });
    commands.push(HostPaintCommand::image_pixels(
        image_rect,
        Some(clip.clone()),
        order,
        resource_key,
        source_size,
        source_size,
        rgba,
        None,
        opacity,
    ));
}
