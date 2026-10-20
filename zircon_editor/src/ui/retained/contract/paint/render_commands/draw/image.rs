#[path = "image/entry.rs"]
mod entry;
#[path = "../../draw/image/pixels.rs"]
mod pixels;
#[path = "../../draw/image/placeholder.rs"]
mod placeholder;

use super::super::super::evidence::{
    record_media_not_ready, record_media_use, register_media_source,
};
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
            Some(image) => {
                if super::super::super::evidence::capture_active() {
                    if super::super::super::visual_assets::visual_asset_source_is_current(image) {
                        if let Some(path) = image.source_path.as_deref() {
                            register_media_source(&image.resource_key, path);
                        }
                    } else {
                        record_media_not_ready(Some(&image.resource_key));
                    }
                }
                record_media_use(&image.resource_key);
            }
            None => record_media_not_ready(command.image_key.as_deref()),
        }
    }
    drawn
}
