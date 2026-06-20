use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::{
    draw_rgba_image_clipped_with_atlas, draw_rgba_image_clipped_with_resource_key,
};
use super::super::super::command::HostPaintCommand;
use super::pixels::image_pixels_with_opacity;
use super::placeholder::draw_image_placeholder;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_image_command(
    frame: &mut HostRgbaFrame,
    command: &HostPaintCommand,
) -> bool {
    if let Some(image) = command.image_pixels.as_ref() {
        if command.opacity >= 1.0 {
            if let Some(atlas) = image.atlas.as_ref().filter(|atlas| atlas.rgba.is_some()) {
                if draw_rgba_image_clipped_with_atlas(
                    frame,
                    command.frame.clone(),
                    command.clip_frame.as_ref(),
                    image.width,
                    image.height,
                    &image.rgba,
                    atlas,
                ) {
                    return true;
                }
            }
        }
        let rgba;
        let source = if command.opacity < 1.0 {
            rgba = image_pixels_with_opacity(image, command.opacity);
            &rgba
        } else {
            image.rgba.as_slice()
        };
        if draw_rgba_image_clipped_with_resource_key(
            frame,
            command.frame.clone(),
            command.clip_frame.as_ref(),
            &image.resource_key,
            image.width,
            image.height,
            source,
        ) {
            return true;
        }
    }

    draw_image_placeholder(frame, command)
}
