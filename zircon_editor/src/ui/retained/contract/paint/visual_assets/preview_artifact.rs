use super::candidates::preview_artifact_candidates;
use super::loading::load_pixels_from_candidates;
use super::pixels::HostPaintImagePixels;
use super::retained::retained_image_pixels;
use super::target::RasterTargetSize;
use crate::ui::retained_host::host_contract::data::FrameRect;

const PREVIEW_RASTER_BUCKET_EDGE: u32 = 8;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn preview_artifact_image_pixels(
    preview_image: &crate::ui::retained_host::primitives::Image,
    source: &str,
    target_width: u32,
    target_height: u32,
    damage_frame: Option<FrameRect>,
) -> Option<HostPaintImagePixels> {
    let target = RasterTargetSize::new(target_width, target_height)
        .map(|target| target.quantized_up(PREVIEW_RASTER_BUCKET_EDGE));
    let key = format!("preview-artifact:{}", source.trim());
    load_pixels_from_candidates(
        || preview_artifact_candidates(source),
        &key,
        target,
        None,
        damage_frame,
    )
    .or_else(|| retained_image_pixels(preview_image, None))
}
