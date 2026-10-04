mod entry;
mod pixels;
mod placeholder;

use super::super::super::evidence::{record_media_not_ready, record_media_use};
use super::super::command::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_image_command(
    frame: &mut HostRgbaFrame,
    command: &HostPaintCommand,
) -> bool {
    let drawn = entry::draw_image_command(frame, command);
    if drawn {
        match command
            .image_pixels
            .as_ref()
            .filter(|image| image.is_valid())
        {
            Some(image) => record_media_use(&image.resource_key),
            None => record_media_not_ready(command.image_key.as_deref()),
        }
    }
    drawn
}
