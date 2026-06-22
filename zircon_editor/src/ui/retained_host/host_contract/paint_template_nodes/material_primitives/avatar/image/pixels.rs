use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::visual_assets::{
    raster_size_from_frame, template_image_pixels, HostPaintImagePixels,
};
use super::super::mask::apply_rounded_alpha_mask;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_image_pixels(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    corner_radius: f32,
) -> Option<HostPaintImagePixels> {
    if !node.has_preview_image && node.media_source.is_empty() {
        return None;
    }
    let (target_width, target_height) = raster_size_from_frame(rect.width, rect.height)?;
    let mut image = template_image_pixels(
        &node.preview_image,
        node.media_source.as_str(),
        "",
        target_width,
        target_height,
        None,
        true,
    )?;
    apply_rounded_alpha_mask(&mut image, corner_radius, rect);
    Some(image)
}
