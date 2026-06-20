use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::visual_assets::{
    raster_size_from_frame, template_image_pixels, HostPaintImagePixels,
};
use super::geometry::avatar_fallback_child_frame;
use super::mask::apply_rounded_alpha_mask;

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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_avatar_image(
    commands: &mut Vec<HostPaintCommand>,
    image: HostPaintImagePixels,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::image_pixels(
        frame,
        Some(clip.clone()),
        order,
        image.resource_key,
        image.width,
        image.height,
        image.rgba,
        image.atlas,
        opacity,
    ));
}
