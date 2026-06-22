use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::super::visual_assets::HostPaintImagePixels;

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
