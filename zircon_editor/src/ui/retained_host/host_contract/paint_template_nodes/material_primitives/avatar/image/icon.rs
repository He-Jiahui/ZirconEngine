use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::visual_assets::{
    raster_size_from_frame, template_image_pixels, HostPaintImagePixels,
};
use super::super::geometry::avatar_fallback_child_frame;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_icon_pixels(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    foreground: [u8; 4],
) -> Option<HostPaintImagePixels> {
    if node.icon_name.is_empty() {
        return None;
    }
    let icon_rect = avatar_fallback_child_frame(rect);
    let (target_width, target_height) = raster_size_from_frame(icon_rect.width, icon_rect.height)?;
    template_image_pixels(
        &node.preview_image,
        "",
        node.icon_name.as_str(),
        target_width,
        target_height,
        Some(foreground),
        false,
    )
}
